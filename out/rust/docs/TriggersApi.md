# \TriggersApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**advance_trigger**](TriggersApi.md#advance_trigger) | **POST** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name}/advance | Advance a trigger
[**create_trigger**](TriggersApi.md#create_trigger) | **PUT** /api/v1/datasets/{owner}/{dataset_name}/triggers | Create a trigger
[**delete_trigger**](TriggersApi.md#delete_trigger) | **DELETE** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name} | Delete a trigger
[**delete_trigger_exception**](TriggersApi.md#delete_trigger_exception) | **DELETE** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name}/exceptions | Delete trigger exception
[**get_all_triggers_in_dataset**](TriggersApi.md#get_all_triggers_in_dataset) | **GET** /api/v1/datasets/{owner}/{dataset_name}/triggers | Get the available triggers for a dataset
[**get_trigger**](TriggersApi.md#get_trigger) | **GET** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name} | Get a trigger
[**poll_trigger**](TriggersApi.md#poll_trigger) | **POST** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name}/fetch | Fetch new messages from a trigger
[**reset_trigger**](TriggersApi.md#reset_trigger) | **POST** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name}/reset | Reset a trigger
[**store_trigger_exception**](TriggersApi.md#store_trigger_exception) | **PUT** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name}/exceptions | Store trigger exception
[**update_trigger**](TriggersApi.md#update_trigger) | **POST** /api/v1/datasets/{owner}/{dataset_name}/triggers/{trigger_name} | Update a trigger



## advance_trigger

> models::AdvanceTriggerResponse advance_trigger(owner, dataset_name, trigger_name, advance_trigger_request)
Advance a trigger

Advance a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |
**advance_trigger_request** | [**AdvanceTriggerRequest**](AdvanceTriggerRequest.md) |  | [required] |

### Return type

[**models::AdvanceTriggerResponse**](AdvanceTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_trigger

> models::CreateTriggerResponse create_trigger(owner, dataset_name, create_trigger_request)
Create a trigger

Create a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**create_trigger_request** | [**CreateTriggerRequest**](CreateTriggerRequest.md) |  | [required] |

### Return type

[**models::CreateTriggerResponse**](CreateTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_trigger

> models::DeleteTriggerResponse delete_trigger(owner, dataset_name, trigger_name)
Delete a trigger

Delete a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |

### Return type

[**models::DeleteTriggerResponse**](DeleteTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_trigger_exception

> models::DeleteStreamExceptionResponse delete_trigger_exception(owner, dataset_name, trigger_name)
Delete trigger exception

Delete trigger exception

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |

### Return type

[**models::DeleteStreamExceptionResponse**](DeleteStreamExceptionResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_triggers_in_dataset

> models::GetAllTriggersInDatasetResponse get_all_triggers_in_dataset(owner, dataset_name)
Get the available triggers for a dataset

Get the available triggers for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetAllTriggersInDatasetResponse**](GetAllTriggersInDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_trigger

> models::GetTriggerResponse get_trigger(owner, dataset_name, trigger_name)
Get a trigger

Get a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |

### Return type

[**models::GetTriggerResponse**](GetTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## poll_trigger

> models::PollTriggerResponse poll_trigger(owner, dataset_name, trigger_name, poll_trigger_request)
Fetch new messages from a trigger

This operation polls a trigger and fetches any new messages

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |
**poll_trigger_request** | [**PollTriggerRequest**](PollTriggerRequest.md) |  | [required] |

### Return type

[**models::PollTriggerResponse**](PollTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_trigger

> models::ResetTriggerResponse reset_trigger(owner, dataset_name, trigger_name, reset_trigger_request)
Reset a trigger

Reset a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |
**reset_trigger_request** | [**ResetTriggerRequest**](ResetTriggerRequest.md) |  | [required] |

### Return type

[**models::ResetTriggerResponse**](ResetTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## store_trigger_exception

> models::StoreExceptionResponse store_trigger_exception(owner, dataset_name, trigger_name, store_exception_request)
Store trigger exception

Store trigger exception

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |
**store_exception_request** | [**StoreExceptionRequest**](StoreExceptionRequest.md) |  | [required] |

### Return type

[**models::StoreExceptionResponse**](StoreExceptionResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_trigger

> models::UpdateTriggerResponse update_trigger(owner, dataset_name, trigger_name, update_trigger_request)
Update a trigger

Update a trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**trigger_name** | **String** |  | [required] |
**update_trigger_request** | [**UpdateTriggerRequest**](UpdateTriggerRequest.md) |  | [required] |

### Return type

[**models::UpdateTriggerResponse**](UpdateTriggerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

