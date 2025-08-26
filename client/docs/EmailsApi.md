# \EmailsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_emails_to_bucket**](EmailsApi.md#add_emails_to_bucket) | **PUT** /api/_private/buckets/{owner}/{bucket_name}/emails | Add emails to bucket
[**get_bucket_emails**](EmailsApi.md#get_bucket_emails) | **POST** /api/_private/buckets/{owner}/{bucket_name}/emails | Get emails from a bucket
[**get_email_from_bucket_by_id**](EmailsApi.md#get_email_from_bucket_by_id) | **GET** /api/_private/buckets/{owner}/{bucket_name}/emails | Get email from bucket
[**upload_email_attachment**](EmailsApi.md#upload_email_attachment) | **PUT** /api/_private/buckets/id:{bucket_id}/emails/{email_id}/attachments/{attachment_index} | Upload an attachment for a email.



## add_emails_to_bucket

> models::AddEmailsToBucketResponse add_emails_to_bucket(owner, bucket_name, add_emails_to_bucket_request, no_charge)
Add emails to bucket

Add emails to bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**add_emails_to_bucket_request** | [**AddEmailsToBucketRequest**](AddEmailsToBucketRequest.md) |  | [required] |
**no_charge** | Option<**bool**> | If set to true, bypasses billing for this request. **For internal use only** - requires DEBUG permission or the 'billing-no-charge' feature flag to be enabled. |  |

### Return type

[**models::AddEmailsToBucketResponse**](AddEmailsToBucketResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket_emails

> models::GetBucketEmailsResponse get_bucket_emails(owner, bucket_name, get_bucket_emails_request)
Get emails from a bucket

Get emails from a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**get_bucket_emails_request** | [**GetBucketEmailsRequest**](GetBucketEmailsRequest.md) |  | [required] |

### Return type

[**models::GetBucketEmailsResponse**](GetBucketEmailsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_email_from_bucket_by_id

> models::GetEmailFromBucketByIdResponse get_email_from_bucket_by_id(owner, bucket_name, id)
Get email from bucket

Get email from bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**id** | **String** | The external ID of the email to retrieve from the bucket. | [required] |

### Return type

[**models::GetEmailFromBucketByIdResponse**](GetEmailFromBucketByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
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

