# \ThreadThemesApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_thread_themes**](ThreadThemesApi.md#get_thread_themes) | **POST** /api/preview/datasets/{owner}/{dataset_name}/thread-themes | Get description of a thread



## get_thread_themes

> models::GetThreadThemesResponse get_thread_themes(owner, dataset_name, get_thread_themes_request)
Get description of a thread

Get description of a thread

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**get_thread_themes_request** | [**GetThreadThemesRequest**](GetThreadThemesRequest.md) |  | [required] |

### Return type

[**models::GetThreadThemesResponse**](GetThreadThemesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

