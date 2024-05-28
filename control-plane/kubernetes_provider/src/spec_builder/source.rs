use super::{
    super::models::{KubernetesSpec, RuntimeConfig},
    build_deployment_spec, SpecBuilder,
};
use k8s_openapi::{
    api::core::v1::{ServicePort, ServiceSpec},
    apimachinery::pkg::util::intstr::IntOrString,
};
use resource_provider_api::models::{ConfigValue, EndpointSetting, ResourceRequest, SourceSpec};
use std::collections::{BTreeMap, HashMap};

macro_rules! hashmap {
  ($( $key: expr => $val: expr ),*) => {{
       let mut map = ::std::collections::BTreeMap::new();
       $( map.insert($key.to_string(), $val); )*
       map
  }}
}

pub struct SourceSpecBuilder {}

impl SpecBuilder<SourceSpec> for SourceSpecBuilder {
    fn build(
        &self,
        source: ResourceRequest<SourceSpec>,
        runtime_config: &RuntimeConfig,
    ) -> Vec<KubernetesSpec> {
        let mut specs = Vec::new();

        specs.push(KubernetesSpec {
            resource_id: source.id.to_string(),
            service_name: "change-svc".to_string(),
            deployment: build_deployment_spec(
                runtime_config,
                "source",
                &source.id,
                "change-svc",
                "source-change-svc",
                1,
                Some(3000),
                hashmap![
                "SOURCE_ID" => ConfigValue::Inline { value: source.id.clone() }
                ],
                None,
                None,
                None,
                None,
            ),
            services: BTreeMap::new(),
            config_maps: BTreeMap::new(),
            volume_claims: BTreeMap::new(),
            pub_sub: None,
            removed: false,
        });

        specs.push(KubernetesSpec {
            resource_id: source.id.to_string(),
            service_name: "change-dispatcher".to_string(),
            deployment: build_deployment_spec(
                runtime_config,
                "source",
                &source.id,
                "change-dispatcher",
                "source-change-dispatcher",
                1,
                Some(3000),
                hashmap![
                "SOURCE_ID" => ConfigValue::Inline { value: source.id.clone() }
                ],
                None,
                None,
                None,
                None,
            ),
            services: BTreeMap::new(),
            config_maps: BTreeMap::new(),
            volume_claims: BTreeMap::new(),
            pub_sub: None,
            removed: false,
        });

        specs.push(KubernetesSpec {
            resource_id: source.id.to_string(),
            service_name: "query-api".to_string(),
            deployment: build_deployment_spec(
                runtime_config,
                "source",
                &source.id,
                "query-api",
                "source-query-api",
                1,
                Some(4001),
                hashmap![
                "SOURCE_ID" => ConfigValue::Inline { value: source.id.clone() }
                ],
                None,
                None,
                None,
                None,
            ),
            services: BTreeMap::new(),
            config_maps: BTreeMap::new(),
            volume_claims: BTreeMap::new(),
            pub_sub: None,
            removed: false,
        });

        let source_spec = source.spec;
        let services = match source_spec.services {
            Some(services) => services,
            None => HashMap::new(),
        }; // Maybe we need to make service as a required field
        let properties = match source_spec.properties {
            Some(props) => props,
            None => HashMap::new(),
        };

        let env_var_map: BTreeMap<String, ConfigValue> = properties.into_iter().collect();

        for (service_name, service_spec) in services {
            let app_port = match service_spec.dapr {
                Some(ref dapr) => match dapr.get("app-port") {
                    Some(port) => match port {
                        ConfigValue::Inline { value } => Some(value.parse::<u16>().unwrap()),
                        _ => None,
                    },
                    None => None,
                },
                None => None,
            };

            let app_protocol = match service_spec.dapr {
                Some(ref dapr) => match dapr.get("app-protocol") {
                    Some(protocol) => match protocol {
                        ConfigValue::Inline { value } => Some(value.clone()),
                        _ => None,
                    },
                    None => None,
                },
                None => None,
            };

            let replica = match service_spec.replica {
                Some(rep) => match rep.parse::<i32>() {
                    Ok(r) => r,
                    Err(_) => 1,
                },
                None => 1,
            };
            let mut env_var_map = env_var_map.clone();
            // combine this with the properties in service_spec
            env_var_map.insert(
                "SOURCE_ID".to_string(),
                ConfigValue::Inline {
                    value: source.id.clone(),
                },
            );
            if let Some(props) = service_spec.properties {
                for (key, value) in props {
                    env_var_map.insert(key, value);
                }
            }
            let mut k8s_services = BTreeMap::new();
            match service_spec.endpoints {
                Some(endpoints) => {
                    for (endpoint_name, endpoint) in endpoints {
                        match endpoint.setting {
                            EndpointSetting::Internal => {
                                let port = endpoint.target.parse::<i32>().unwrap();
                                let service_spec = ServiceSpec {
                                    type_: Some("ClusterIP".to_string()),
                                    selector: Some(hashmap![
                                        "drasi/type".to_string() => "source".to_string(),
                                        "drasi/resource".to_string() => source.id.clone(),
                                        "drasi/service".to_string() => service_name.clone()
                                    ]),
                                    ports: Some(vec![ServicePort {
                                        name: Some(endpoint_name.clone()),
                                        port: port,
                                        target_port: Some(IntOrString::Int(port)),
                                        ..Default::default()
                                    }]),
                                    ..Default::default()
                                };

                                k8s_services.insert(endpoint_name.clone(), service_spec);
                            }
                            EndpointSetting::External => {
                                unimplemented!();
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }
                }
                None => {}
            };

            let k8s_spec = KubernetesSpec {
                resource_id: source.id.to_string(),
                service_name: service_name.to_string(),
                deployment: build_deployment_spec(
                    runtime_config,
                    "source",
                    &source.id,
                    &service_name,
                    service_spec.image.as_str(),
                    replica,
                    app_port,
                    env_var_map.clone(),
                    None,
                    None,
                    None,
                    app_protocol,
                ),
                services: k8s_services,
                config_maps: BTreeMap::new(),
                volume_claims: BTreeMap::new(),
                pub_sub: None,
                removed: false,
            };
            specs.push(k8s_spec);
        }
        specs
    }
}