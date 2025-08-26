# \CommentsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_comments**](CommentsApi.md#add_comments) | **PUT** /api/_private/sources/{owner}/{source_name}/comments | Add a batch of comments
[**delete_comment**](CommentsApi.md#delete_comment) | **DELETE** /api/v1/sources/{owner}/{source_name}/comments | Delete a comment by ID
[**get_comment**](CommentsApi.md#get_comment) | **GET** /api/v1/sources/{owner}/{source_name}/comments/{comment_id} | Get a comment by ID
[**get_comment_audio**](CommentsApi.md#get_comment_audio) | **GET** /api/_private/sources/id:{source_id}/comments/{comment_id}/audio | Get the audio for a comment
[**get_source_comments**](CommentsApi.md#get_source_comments) | **GET** /api/_private/sources/{owner}/{source_name}/comments | Get comments from a source
[**query_comments**](CommentsApi.md#query_comments) | **POST** /api/_private/datasets/{owner}/{dataset_name}/query | Query comments in a dataset
[**set_comment_audio**](CommentsApi.md#set_comment_audio) | **PUT** /api/_private/sources/id:{raw_source_id}/comments/{raw_comment_id}/audio | Set the audio for a comment
[**sniff_csv**](CommentsApi.md#sniff_csv) | **PUT** /api/_private/sources/sniff-csv | Sniff a CSV file
[**sync_comments**](CommentsApi.md#sync_comments) | **POST** /api/v1/sources/{owner}/{source_name}/sync | Sync a batch of comments
[**sync_comments_from_csv**](CommentsApi.md#sync_comments_from_csv) | **PUT** /api/_private/sources/{owner}/{source_name}/sync-csv | Sync comments from a CSV file
[**sync_raw_emails**](CommentsApi.md#sync_raw_emails) | **POST** /api/v1/sources/{owner}/{source_name}/sync-raw-emails | Sync a batch of raw emails
[**upload_comment_attachment**](CommentsApi.md#upload_comment_attachment) | **PUT** /api/_private/sources/id:{source_id}/comments/{comment_id}/attachments/{attachment_index} | Upload an attachment for a comment.



## add_comments

> models::AddCommentsResponse add_comments(owner, source_name, add_comments_request, no_charge)
Add a batch of comments

Add a batch of comments. To overwrite existing comments, you need to specify the latest `context` field.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**add_comments_request** | [**AddCommentsRequest**](AddCommentsRequest.md) |  | [required] |
**no_charge** | Option<**bool**> | If set to true, bypasses billing for this request. **For internal use only** - requires DEBUG permission or the 'billing-no-charge' feature flag to be enabled. |  |

### Return type

[**models::AddCommentsResponse**](AddCommentsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_comment

> models::DeleteCommentResponse delete_comment(owner, source_name, id, ids)
Delete a comment by ID

Delete a comment by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**id** | Option<[**Vec<String>**](String.md)> | List of comment IDs to delete. Use multiple 'id' query parameters like ?id=abc123&id=def456. Maximum 128 comment IDs. |  |
**ids** | Option<**String**> | Comma-separated list of comment IDs to delete. Example: ?ids=abc123,def456. Maximum 128 comment IDs. |  |

### Return type

[**models::DeleteCommentResponse**](DeleteCommentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comment

> models::GetCommentResponse get_comment(owner, source_name, comment_id, include_markup)
Get a comment by ID

Get a comment by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**comment_id** | **String** |  | [required] |
**include_markup** | Option<**bool**> | Include markup in the comment |  |

### Return type

[**models::GetCommentResponse**](GetCommentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comment_audio

> std::path::PathBuf get_comment_audio(source_id, comment_id)
Get the audio for a comment

Get the audio for a comment

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
- **Accept**: audio/x-wav, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_source_comments

> models::GetSourceCommentsResponse get_source_comments(owner, source_name, after, limit, from_timestamp, to_timestamp, include_thread_properties, include_markup, direction)
Get comments from a source

Get comments from a source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**after** | Option<**String**> | An opaque continuation token for pagination. Use this to fetch the next batch of comments after a specific point in time. |  |
**limit** | Option<**i32**> | The maximum number of comments to return in this batch. |  |
**from_timestamp** | Option<**String**> | Only return comments created at or after this timestamp. Format: ISO 8601 datetime string (e.g., '2023-01-01T00:00:00Z'). |  |
**to_timestamp** | Option<**String**> | Only return comments created before this timestamp. Format: ISO 8601 datetime string (e.g., '2023-12-31T23:59:59Z'). |  |
**include_thread_properties** | Option<**bool**> | Whether to include thread properties in the response. Thread properties contain conversation-level metadata. |  |
**include_markup** | Option<**bool**> | Whether to include rich text markup in comment content. This includes formatting information for the comment text. |  |
**direction** | Option<**String**> | The sort direction for comments. 'ascending' sorts by timestamp from oldest to newest, 'descending' sorts from newest to oldest. |  |

### Return type

[**models::GetSourceCommentsResponse**](GetSourceCommentsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## query_comments

> models::QueryCommentsResponse query_comments(owner, dataset_name, query_comments_request, limit, continuation)
Query comments in a dataset

Query comments in a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**query_comments_request** | [**QueryCommentsRequest**](QueryCommentsRequest.md) |  | [required] |
**limit** | Option<**i32**> | Maximum number of comments to return. Overrides limit in request body if provided. |  |
**continuation** | Option<**String**> | Continuation token for pagination. Overrides continuation in request body if provided. |  |

### Return type

[**models::QueryCommentsResponse**](QueryCommentsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_comment_audio

> models::SetCommentAudioResponse set_comment_audio(raw_source_id, raw_comment_id, body)
Set the audio for a comment

Set the audio for a comment

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_source_id** | **String** |  | [required] |
**raw_comment_id** | **String** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::SetCommentAudioResponse**](SetCommentAudioResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sniff_csv

> models::SniffCsvResponse sniff_csv(body)
Sniff a CSV file

Sniff a CSV file

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::SniffCsvResponse**](SniffCsvResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_comments

> models::SyncCommentsResponse sync_comments(owner, source_name, sync_comments_request, no_charge)
Sync a batch of comments

Sync a batch of comments. Any comments with the same IDs in the source will be overwritten.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**sync_comments_request** | [**SyncCommentsRequest**](SyncCommentsRequest.md) |  | [required] |
**no_charge** | Option<**bool**> | If set to true, bypasses billing for this request. **For internal use only** - requires DEBUG permission or the 'billing-no-charge' feature flag to be enabled. |  |

### Return type

[**models::SyncCommentsResponse**](SyncCommentsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_comments_from_csv

> models::SyncCommentsFromCsvResponse sync_comments_from_csv(owner, source_name, body)
Sync comments from a CSV file

Sync comments from a CSV file

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::SyncCommentsFromCsvResponse**](SyncCommentsFromCsvResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_raw_emails

> models::SyncRawEmailsResponse sync_raw_emails(owner, source_name, sync_raw_emails_request, no_charge)
Sync a batch of raw emails

Sync a batch of raw emails. Any comments with the same IDs in the source will be overwritten.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**sync_raw_emails_request** | [**SyncRawEmailsRequest**](SyncRawEmailsRequest.md) |  | [required] |
**no_charge** | Option<**bool**> | If set to true, bypasses billing for this request. **For internal use only** - requires DEBUG permission or the 'billing-no-charge' feature flag to be enabled. |  |

### Return type

[**models::SyncRawEmailsResponse**](SyncRawEmailsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

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

