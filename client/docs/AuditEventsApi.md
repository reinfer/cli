# \AuditEventsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query_audit_events**](AuditEventsApi.md#query_audit_events) | **POST** /api/v1/audit_events/query | Query audit events



## query_audit_events

> models::QueryAuditEventsResponse query_audit_events(query_audit_events_request)
Query audit events

Query audit events

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_audit_events_request** | [**QueryAuditEventsRequest**](QueryAuditEventsRequest.md) |  | [required] |

### Return type

[**models::QueryAuditEventsResponse**](QueryAuditEventsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

