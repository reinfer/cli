# QueryAuditEventsResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** |  | 
**audit_events** | [**Vec<models::AuditEvent>**](AuditEvent.md) |  | 
**tenants** | [**Vec<models::AuditEventsTenant>**](_AuditEventsTenant.md) |  | 
**users** | Option<[**Vec<models::AuditEventsUser>**](_AuditEventsUser.md)> |  | [optional]
**sources** | Option<[**Vec<models::AuditEventsSource>**](_AuditEventsSource.md)> |  | [optional]
**datasets** | Option<[**Vec<models::AuditEventsDataset>**](_AuditEventsDataset.md)> |  | [optional]
**triggers** | Option<[**Vec<models::AuditEventsTrigger>**](_AuditEventsTrigger.md)> |  | [optional]
**projects** | Option<[**Vec<models::AuditEventsProject>**](_AuditEventsProject.md)> |  | [optional]
**continuation** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


