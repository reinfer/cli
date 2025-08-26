# \ConversationalFilterApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query_conversational_filter**](ConversationalFilterApi.md#query_conversational_filter) | **POST** /api/_private/datasets/{owner}/{dataset_name}/conversational-filter-query | Query comments in a dataset with an LLM



## query_conversational_filter

> models::ConversationalFilterResponse query_conversational_filter(owner, dataset_name, conversational_filter_request)
Query comments in a dataset with an LLM

Query comments in a dataset with an LLM

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**conversational_filter_request** | [**ConversationalFilterRequest**](ConversationalFilterRequest.md) |  | [required] |

### Return type

[**models::ConversationalFilterResponse**](ConversationalFilterResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

