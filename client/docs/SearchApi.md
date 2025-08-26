# \SearchApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_document_search**](SearchApi.md#get_document_search) | **GET** /api/_private/attachments/{attachment_reference}/selections/search | Get the search result selections for a given query



## get_document_search

> models::GetDocumentSearchResponse get_document_search(attachment_reference, query)
Get the search result selections for a given query

Get document search selections

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |
**query** | **String** |  | [required] |

### Return type

[**models::GetDocumentSearchResponse**](GetDocumentSearchResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

