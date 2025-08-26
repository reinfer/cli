# \AlertsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**alert_issues**](AlertsApi.md#alert_issues) | **GET** /api/_private/alerts/{owner}/{raw_alert_name}/issues | Get issues for an alert
[**create_alert**](AlertsApi.md#create_alert) | **PUT** /api/_private/alerts/{owner}/{raw_alert_name} | Create an alert
[**delete_alert**](AlertsApi.md#delete_alert) | **DELETE** /api/_private/alerts/id:{raw_alert_id} | Delete a alert
[**delete_alert_subscription**](AlertsApi.md#delete_alert_subscription) | **DELETE** /api/_private/alerts/subscriptions/{raw_alert_id} | Unsubscribes the user from an alert
[**get_alert**](AlertsApi.md#get_alert) | **GET** /api/_private/alerts/{owner}/{raw_alert_name} | Get a alert by project and name
[**get_alert_subscriptions**](AlertsApi.md#get_alert_subscriptions) | **GET** /api/_private/alerts/subscriptions | Get all alerts that the user is subscribed to
[**get_all_alerts**](AlertsApi.md#get_all_alerts) | **GET** /api/_private/alerts | Get all alerts for a user
[**get_issue**](AlertsApi.md#get_issue) | **GET** /api/_private/issues/{hex_issue_id} | Get an issue
[**preview_alert**](AlertsApi.md#preview_alert) | **POST** /api/_private/alerts/{owner}/{raw_alert_name}/preview | Preview an alert
[**query_issues**](AlertsApi.md#query_issues) | **POST** /api/_private/issues/query | Query issues for the current user
[**update_alert**](AlertsApi.md#update_alert) | **POST** /api/_private/alerts/{owner}/{raw_alert_name} | Update a alert
[**update_issue_status**](AlertsApi.md#update_issue_status) | **POST** /api/_private/issues/{hex_issue_id}/status | Update the status of the issue



## alert_issues

> models::AlertIssuesResponse alert_issues(owner, raw_alert_name)
Get issues for an alert

Get issues for an alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**raw_alert_name** | **String** |  | [required] |

### Return type

[**models::AlertIssuesResponse**](AlertIssuesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_alert

> models::CreateAlertResponse create_alert(owner, raw_alert_name, create_alert_request)
Create an alert

Create an alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**raw_alert_name** | **String** |  | [required] |
**create_alert_request** | [**CreateAlertRequest**](CreateAlertRequest.md) |  | [required] |

### Return type

[**models::CreateAlertResponse**](CreateAlertResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_alert

> models::DeleteAlertResponse delete_alert(raw_alert_id)
Delete a alert

Delete a alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_alert_id** | **String** |  | [required] |

### Return type

[**models::DeleteAlertResponse**](DeleteAlertResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_alert_subscription

> models::DeleteAlertSubscriptionReponse delete_alert_subscription(raw_alert_id)
Unsubscribes the user from an alert

Unsubscribes the user from an alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_alert_id** | **String** |  | [required] |

### Return type

[**models::DeleteAlertSubscriptionReponse**](DeleteAlertSubscriptionReponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_alert

> models::GetAlertResponse get_alert(owner, raw_alert_name)
Get a alert by project and name

Get a alert by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** | The project this alert is in | [required] |
**raw_alert_name** | **String** |  | [required] |

### Return type

[**models::GetAlertResponse**](GetAlertResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_alert_subscriptions

> models::GetAlertSubscriptionsResponse get_alert_subscriptions()
Get all alerts that the user is subscribed to

Get all alerts that the user is subscribed to

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAlertSubscriptionsResponse**](GetAlertSubscriptionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_alerts

> models::GetAllAlertsResponse get_all_alerts()
Get all alerts for a user

Get all alerts for a user

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllAlertsResponse**](GetAllAlertsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_issue

> models::GetIssueResponse get_issue(hex_issue_id)
Get an issue

Get an issue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hex_issue_id** | **String** |  | [required] |

### Return type

[**models::GetIssueResponse**](GetIssueResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## preview_alert

> models::PreviewAlertResponse preview_alert(owner, raw_alert_name, preview_alert_request)
Preview an alert

Preview an alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**raw_alert_name** | **String** |  | [required] |
**preview_alert_request** | [**PreviewAlertRequest**](PreviewAlertRequest.md) |  | [required] |

### Return type

[**models::PreviewAlertResponse**](PreviewAlertResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## query_issues

> models::QueryIssuesResponse query_issues(query_issues_request)
Query issues for the current user

Query issues for the current user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_issues_request** | [**QueryIssuesRequest**](QueryIssuesRequest.md) |  | [required] |

### Return type

[**models::QueryIssuesResponse**](QueryIssuesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_alert

> models::UpdateAlertResponse update_alert(owner, raw_alert_name, update_alert_request)
Update a alert

Update a alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**raw_alert_name** | **String** |  | [required] |
**update_alert_request** | [**UpdateAlertRequest**](UpdateAlertRequest.md) |  | [required] |

### Return type

[**models::UpdateAlertResponse**](UpdateAlertResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_issue_status

> models::UpdateIssueStatusResponse update_issue_status(hex_issue_id, update_issue_status_request)
Update the status of the issue

Update the status of the issue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hex_issue_id** | **String** |  | [required] |
**update_issue_status_request** | [**UpdateIssueStatusRequest**](UpdateIssueStatusRequest.md) |  | [required] |

### Return type

[**models::UpdateIssueStatusResponse**](UpdateIssueStatusResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

