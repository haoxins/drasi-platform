
using Microsoft.PowerPlatform.Dataverse.Client;
using Microsoft.Xrm.Sdk;
using Microsoft.Xrm.Sdk.Messages;
using Microsoft.Xrm.Sdk.Query;

namespace Reactivator.Services
{
    class SyncWorker(IChangePublisher changePublisher, IDeltaTokenStore checkpointStore, IEventMapper eventMapper, IOrganizationServiceAsync serviceClient, string entityName, int interval) : BackgroundService
    {
        private readonly IChangePublisher _changePublisher = changePublisher;
        private readonly IDeltaTokenStore _checkpointStore = checkpointStore;
        private readonly IEventMapper _eventMapper = eventMapper;
        private readonly IOrganizationServiceAsync _serviceClient = serviceClient;
        private readonly string _entityName = entityName;
        private readonly int _interval = interval;

    
        protected override async Task ExecuteAsync(CancellationToken stoppingToken)
        {
            var lastToken = await _checkpointStore.GetDeltaToken(_entityName);
            if (String.IsNullOrEmpty(lastToken))
            {
                lastToken = await GetCurrentDeltaToken();
            }

            while (!stoppingToken.IsCancellationRequested)
            {
                var delta = await GetChanges(lastToken);
                Console.WriteLine($"Got {delta.Item1.Count} changes for entity {_entityName}");
                foreach (var change in delta.Item1)
                {
                    var notification = await _eventMapper.MapEventAsync(change);
                    await _changePublisher.Publish([notification]);
                }

                await _checkpointStore.SetDeltaToken(_entityName, delta.Item2);
                lastToken = delta.Item2;
                await Task.Delay(_interval * 1000, stoppingToken);
            }
        }

        private async Task<(BusinessEntityChangesCollection, string)> GetChanges(string deltaToken)
        {
            var result = new BusinessEntityChangesCollection();
            
            RetrieveEntityChangesRequest req = new RetrieveEntityChangesRequest()
            {
                EntityName = _entityName,
                Columns = new ColumnSet(true),    
                DataVersion = deltaToken,
                PageInfo = new PagingInfo()
                { Count = 1000, PageNumber = 1, ReturnTotalRecordCount = false }
            };

            RetrieveEntityChangesResponse resp = (RetrieveEntityChangesResponse)_serviceClient.Execute(req);
            var moreData = true;

            while (moreData)
            {
                result.AddRange(resp.EntityChanges.Changes);                
                moreData = resp.EntityChanges.MoreRecords;
                if (moreData)
                {
                    resp = (RetrieveEntityChangesResponse)_serviceClient.Execute(new RetrieveEntityChangesRequest()
                    {
                        EntityName = _entityName,
                        Columns = new ColumnSet(true),
                        DataVersion = deltaToken,
                        PageInfo = new PagingInfo()
                        { PagingCookie = resp.EntityChanges.PagingCookie, Count = 1000 }
                    });
                }
            }

            return (result, resp.EntityChanges.DataToken);
        }

        private async Task<string> GetCurrentDeltaToken()
        {
            RetrieveEntityChangesRequest req = new RetrieveEntityChangesRequest()
            {
                EntityName = _entityName,
                Columns = new ColumnSet(true),    
                PageInfo = new PagingInfo()
                { Count = 1000, PageNumber = 1, ReturnTotalRecordCount = false }
            };

            RetrieveEntityChangesResponse resp = (RetrieveEntityChangesResponse)_serviceClient.Execute(req);
            var moreData = true;

            while (moreData)
            {
                moreData = resp.EntityChanges.MoreRecords;
                if (moreData)
                {
                    resp = (RetrieveEntityChangesResponse)_serviceClient.Execute(new RetrieveEntityChangesRequest()
                    {
                        EntityName = _entityName,
                        Columns = new ColumnSet(true),
                        PageInfo = new PagingInfo()
                        { PagingCookie = resp.EntityChanges.PagingCookie, Count = 1000 }
                    });
                }
            }

            return resp.EntityChanges.DataToken;
        }
    }
}