# \MetadataApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_document_metadata**](MetadataApi.md#get_document_metadata) | **GET** /api/_private/attachments/{attachment_reference}/render | Get the metadata for all pages in a document
[**get_page_image**](MetadataApi.md#get_page_image) | **GET** /api/_private/attachments/{attachment_reference}/render/pages/{page_index} | Get the image for a given page
[**get_page_selections**](MetadataApi.md#get_page_selections) | **GET** /api/_private/attachments/{attachment_reference}/selections/pages/{page_index} | Get the OCR selections for a given page
[**get_page_thumbnail**](MetadataApi.md#get_page_thumbnail) | **GET** /api/_private/attachments/{attachment_reference}/thumbnail/pages/{page_index} | Get the thumbnail for a given page



## get_document_metadata

> models::GetDocumentMetadataResponse get_document_metadata(attachment_reference)
Get the metadata for all pages in a document

Get metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |

### Return type

[**models::GetDocumentMetadataResponse**](GetDocumentMetadataResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_image

> std::path::PathBuf get_page_image(attachment_reference, page_index)
Get the image for a given page

Get page image

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |
**page_index** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_selections

> models::GetPageSelectionsResponse get_page_selections(attachment_reference, page_index)
Get the OCR selections for a given page

Get ocr selections

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |
**page_index** | **String** |  | [required] |

### Return type

[**models::GetPageSelectionsResponse**](GetPageSelectionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_thumbnail

> std::path::PathBuf get_page_thumbnail(attachment_reference, page_index)
Get the thumbnail for a given page

Get page thumbnail

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |
**page_index** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

