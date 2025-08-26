# \DocumentsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_document**](DocumentsApi.md#get_document) | **GET** /api/_private/sources/id:{source_id}/documents/{comment_id} | Get an unstructured document.
[**upload_document**](DocumentsApi.md#upload_document) | **PUT** /api/_private/sources/id:{source_id}/documents | Upload an unstructured document.



## get_document

> std::path::PathBuf get_document(source_id, comment_id)
Get an unstructured document.

Get an unstructured document.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source_id** | **String** |  | [required] |
**comment_id** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_document

> models::UploadDocumentResponse upload_document(source_id, file)
Upload an unstructured document.

Upload an unstructured document.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source_id** | **String** |  | [required] |
**file** | Option<**std::path::PathBuf**> |  |  |

### Return type

[**models::UploadDocumentResponse**](UploadDocumentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

