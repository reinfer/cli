# \AttachmentsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_attachment**](AttachmentsApi.md#get_attachment) | **GET** /api/v1/attachments/{attachment_reference} | Get an attachment's content.
[**get_document_metadata**](AttachmentsApi.md#get_document_metadata) | **GET** /api/_private/attachments/{attachment_reference}/render | Get the metadata for all pages in a document
[**get_document_search**](AttachmentsApi.md#get_document_search) | **GET** /api/_private/attachments/{attachment_reference}/selections/search | Get the search result selections for a given query
[**get_page_image**](AttachmentsApi.md#get_page_image) | **GET** /api/_private/attachments/{attachment_reference}/render/pages/{page_index} | Get the image for a given page
[**get_page_selections**](AttachmentsApi.md#get_page_selections) | **GET** /api/_private/attachments/{attachment_reference}/selections/pages/{page_index} | Get the OCR selections for a given page
[**get_page_thumbnail**](AttachmentsApi.md#get_page_thumbnail) | **GET** /api/_private/attachments/{attachment_reference}/thumbnail/pages/{page_index} | Get the thumbnail for a given page
[**upload_comment_attachment**](AttachmentsApi.md#upload_comment_attachment) | **PUT** /api/_private/sources/id:{source_id}/comments/{comment_id}/attachments/{attachment_index} | Upload an attachment for a comment.
[**upload_email_attachment**](AttachmentsApi.md#upload_email_attachment) | **PUT** /api/_private/buckets/id:{bucket_id}/emails/{email_id}/attachments/{attachment_index} | Upload an attachment for a email.



## get_attachment

> std::path::PathBuf get_attachment(attachment_reference)
Get an attachment's content.

Get an attachment's content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_reference** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## upload_comment_attachment

> models::UploadAttachmentResponse upload_comment_attachment(source_id, comment_id, attachment_index, file)
Upload an attachment for a comment.

Upload an attachment for a comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source_id** | **String** |  | [required] |
**comment_id** | **String** |  | [required] |
**attachment_index** | **String** |  | [required] |
**file** | Option<**std::path::PathBuf**> |  |  |

### Return type

[**models::UploadAttachmentResponse**](UploadAttachmentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_email_attachment

> models::UploadAttachmentResponse upload_email_attachment(bucket_id, email_id, attachment_index, file)
Upload an attachment for a email.

Upload an attachment for a email.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** |  | [required] |
**email_id** | **String** |  | [required] |
**attachment_index** | **String** |  | [required] |
**file** | Option<**std::path::PathBuf**> |  |  |

### Return type

[**models::UploadAttachmentResponse**](UploadAttachmentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

