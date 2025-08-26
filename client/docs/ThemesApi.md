# \ThemesApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_themes**](ThemesApi.md#get_themes) | **GET** /api/_private/datasets/{owner}/{dataset_name}/themes | Get a themes by project and name



## get_themes

> models::GetThemesResponse get_themes(owner, dataset_name)
Get a themes by project and name

Get a themes by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetThemesResponse**](GetThemesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

