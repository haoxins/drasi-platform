use std::sync::Arc;

use change_service_config::ChangeServiceConfig;
use log::{debug, info};
use serde_json::{json, Value};
use subscribers::Subscriber;
use uuid::Uuid;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use drasi_comms_abstractions::comms::{Headers, Publisher};
use drasi_comms_dapr::comms::DaprHttpPublisher;

mod change_service_config;
mod subscriber_map;
mod subscribers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Source Change Service");

    let config = ChangeServiceConfig::from_env();

    let node_subscriber = Subscriber::new();
    let rel_subscriber = Subscriber::new();

    let addr = "https://127.0.0.1".to_string();
    let mut dapr_client = dapr::Client::<dapr::client::TonicClient>::connect(addr)
        .await
        .expect("Unable to connect to Dapr");

    let query_condition = json!({
        "filter": {
            "EQ": { "type": "SourceSubscription"}
        },
    });
    let response = match dapr_client
        .query_state_alpha1(config.clone().subscriber_store, query_condition, None)
        .await
    {
        Ok(response) => response.results,
        Err(e) => {
            println!("Error querying state: {:?}", e);
            vec![]
        }
    };

    for sub in response {
        let data: Value = serde_json::from_slice(&sub.data).unwrap();
        // if the data is corrupt for this subscription, this will cause a panic and stop loading all the others... we should probably log an error but continue to load the rest of the subscriptions
        let node_labels: Vec<&str> = match data["nodeLabels"].as_array() {
            Some(labels) => labels
                .iter()
                .map(|label| label.as_str().unwrap_or(""))
                .collect(),
            None => {
                log::error!("Error loading nodeLabels for subscription: {:?}", data);
                vec![]
            }
        };
        node_subscriber.add_labels(
            node_labels,
            data["queryNodeId"].as_str().unwrap(),
            data["queryId"].as_str().unwrap(),
        );
        let rel_labels: Vec<&str> = match data["relLabels"].as_array() {
            Some(labels) => labels
                .iter()
                .map(|label| label.as_str().unwrap_or(""))
                .collect(),
            None => {
                log::error!("Error loading relLabels for subscription: {:?}", data);
                vec![]
            }
        };
        rel_subscriber.add_labels(
            rel_labels,
            data["queryNodeId"].as_str().unwrap(),
            data["queryId"].as_str().unwrap(),
        );
    }

    info!(
        "Forwarding Node types: {:?}",
        node_subscriber.get_label_map()
    );
    info!(
        "Forwarding Relation types: {:?}",
        rel_subscriber.get_label_map()
    );

    let topic = format!("{}-dispatch", config.source_id.clone()).to_string();
    let dapr_port = match config.dapr_port.parse::<u16>() {
        Ok(port) => port,
        Err(_e) => {
            unreachable!()
        }
    };
    let publisher = DaprHttpPublisher::new(
        "127.0.0.1".to_string(),
        dapr_port,
        config.pubsub_name.clone(),
        topic,
    );

    let shared_state = Arc::new(AppState {
        node_subscriber,
        rel_subscriber,
        config: config.clone(),
        publisher,
    });
    let subscriber_server = Router::new()
        .route("/dapr/subscribe", get(subscribe))
        .route("/receive", post(receive))
        .with_state(shared_state);

    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, subscriber_server).await.unwrap();

    Ok(())
}

struct AppState {
    node_subscriber: Subscriber,
    rel_subscriber: Subscriber,
    config: ChangeServiceConfig,
    publisher: DaprHttpPublisher,
}

async fn subscribe() -> impl IntoResponse {
    let config = ChangeServiceConfig::from_env();
    // just do a json that is a list of subscriptions
    let subscriptions = vec![json! {
        {
            "pubsubname": config.pubsub_name.clone(),
            "topic": format!("{}-change", config.source_id),
            "route": "receive"
        }
    }];

    Json(subscriptions)
}

#[axum::debug_handler]
async fn receive(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let trace_parent = match headers.get("traceparent") {
        Some(trace_parent) => trace_parent.to_str().unwrap().to_string(),
        None => {
            return StatusCode::BAD_REQUEST;
        }
    };
    let config = state.config.clone();
    let node_subscriber = &state.node_subscriber;
    let rel_subscriber = &state.rel_subscriber;
    let json_data = body["data"].clone();

    let publisher = &state.publisher;
    match process_changes(
        publisher,
        json_data,
        config,
        &node_subscriber,
        &rel_subscriber,
        trace_parent,
    )
    .await
    {
        Ok(_) => {}
        Err(_e) => {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}

async fn process_changes(
    publisher: &DaprHttpPublisher,
    changes: Value,
    config: ChangeServiceConfig,
    node_subscriber: &Subscriber,
    rel_subscriber: &Subscriber,
    traceparent: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(changes) = changes.as_array() {
        for change in changes {
            let change_service_start = chrono::Utc::now().timestamp_millis();
            let change_id = Uuid::new_v4().to_string();

            info!(
                "Processing change - db:{}, type:{}, id:{}",
                change["payload"]["source"]["db"], change["payload"]["source"]["table"], change_id
            );
            debug!("ChangeEvent: {}", change);

            if change["payload"]["source"]["db"] == "ReactiveGraph" {
                if change["payload"]["source"]["table"] == "SourceSubscription" {
                    if change["op"] == "i" {
                        info!(
                            "Activating new SourceSubscription: id:{}",
                            change["payload"]["after"]["id"]
                        );
                        let node_labels: Vec<&str> = change["payload"]["after"]["nodeLabels"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|label| label.as_str().unwrap_or(""))
                            .collect();
                        node_subscriber.add_labels(
                            node_labels,
                            change["payload"]["after"]["queryNodeId"].as_str().unwrap(),
                            change["payload"]["after"]["queryId"].as_str().unwrap(),
                        );
                        let rel_labels: Vec<&str> = change["payload"]["after"]["relLabels"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|label| label.as_str().unwrap_or(""))
                            .collect();
                        rel_subscriber.add_labels(
                            rel_labels,
                            change["payload"]["after"]["queryNodeId"].as_str().unwrap(),
                            change["payload"]["after"]["queryId"].as_str().unwrap(),
                        );

                        // let mut dapr_client = dapr_client.clone();
                        let source_subscription_value = json!({
                            "type": "SourceSubscription",
                            "queryId": change["payload"]["after"]["queryId"],
                            "queryNodeId": change["payload"]["after"]["queryNodeId"],
                            "nodeLabels": change["payload"]["after"]["nodeLabels"],
                            "relLabels": change["payload"]["after"]["relLabels"]
                        });
                        let mut headers = std::collections::HashMap::new();
                        headers.insert("traceparent".to_string(), traceparent.clone());
                        let headers = Headers::new(headers);
                        match publisher.publish(source_subscription_value, headers).await {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        // TODO - supprt other ops on SourceSubscriptions
                    }
                } else {
                }
                return Ok(());
            }

            let mut subscriptions: Option<Vec<String>> = None;
            if change["payload"]["source"]["table"] == "node" {
                if change["op"] == "i" || change["op"] == "u" {
                    let labels: Vec<&str> = change["payload"]["after"]["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|label| label.as_str().unwrap_or(""))
                        .collect();
                    subscriptions = node_subscriber.get_subscribers_for_labels(labels);
                } else if change["op"] == "d" {
                    let labels: Vec<&str> = change["payload"]["before"]["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|label| label.as_str().unwrap_or(""))
                        .collect();
                    subscriptions = node_subscriber.get_subscribers_for_labels(labels);
                } else {
                }
            } else if change["payload"]["source"]["table"] == "rel" {
                if change["op"] == "i" || change["op"] == "u" {
                    let labels: Vec<&str> = change["payload"]["after"]["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|label| label.as_str().unwrap_or(""))
                        .collect();
                    subscriptions = rel_subscriber.get_subscribers_for_labels(labels);
                } else if change["op"] == "d" {
                    let labels: Vec<&str> = change["payload"]["before"]["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|label| label.as_str().unwrap_or(""))
                        .collect();
                    subscriptions = rel_subscriber.get_subscribers_for_labels(labels);
                } else {
                }
            } else {
            }

            match subscriptions {
                Some(subscriptions) => {
                    let subscriptions: Vec<Value> = subscriptions
                        .iter()
                        .map(|subscription| {
                            let parsed_subscription: Value =
                                serde_json::from_str(subscription).unwrap();
                            parsed_subscription
                        })
                        .collect();

                    let change_dispatch_event = json!([{
                        "id": change_id,
                        "sourceId": config.source_id,
                        "type": change["op"],
                        "elementType": change["payload"]["source"]["table"],
                        "subscriptions": subscriptions,
                        "time": {
                            "seq": change["payload"]["source"]["lsn"],
                            "ms": change["ts_ms"]
                        },
                        "before": change["payload"]["before"],
                        "after": change["payload"]["after"],
                        "metadata": {
                            "changeEvent": change,
                            "tracking": {
                                "source": {
                                    "seq": change["payload"]["source"]["lsn"],
                                    "reactivator_ms": change["ts_ms"],
                                    "changeSvcStart_ms": change_service_start,
                                    "changeSvcEnd_ms": chrono::Utc::now().timestamp_millis()
                                }
                            }
                        }
                    }]);

                    let publish_topic = format!("{}-dispatch", config.source_id);
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("traceparent".to_string(), traceparent.clone());
                    let headers = Headers::new(headers);
                    match publisher.publish(change_dispatch_event, headers).await {
                        Ok(_) => {
                            println!("published event to topic: {}", publish_topic);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                None => {
                    println!("No subscribers for change: {:?}", change);
                }
            }
        }
    }
    Ok(())
}