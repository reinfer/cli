# \LabelDefsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_label_defs_bulk**](LabelDefsApi.md#create_label_defs_bulk) | **PUT** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name} | Label Group Bulk Create
[**delete_label_def**](LabelDefsApi.md#delete_label_def) | **DELETE** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name} | Delete Label Def
[**get_field_group_history**](LabelDefsApi.md#get_field_group_history) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name}/{label_name}/history | Get history for a field group
[**get_field_history**](LabelDefsApi.md#get_field_history) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name}/{label_name}/fields/{field_id}/history | Get history for a field
[**get_label_def**](LabelDefsApi.md#get_label_def) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name} | Get Label Def
[**update_label_def**](LabelDefsApi.md#update_label_def) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labels/{label_group_name} | Update label def



## create_label_defs_bulk

> models::CreateLabelDefsBulkResponse create_label_defs_bulk(owner, dataset_name, label_group_name, create_or_update_label_defs_bulk_request)
Label Group Bulk Create

Label Group Bulk Create

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**create_or_update_label_defs_bulk_request** | [**CreateOrUpdateLabelDefsBulkRequest**](CreateOrUpdateLabelDefsBulkRequest.md) |  | [required] |

### Return type

[**models::CreateLabelDefsBulkResponse**](CreateLabelDefsBulkResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_label_def

> models::DeleteLabelDefResponse delete_label_def(owner, dataset_name, label_group_name, label_name)
Delete Label Def

Delete Label Def

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**label_name** | **String** |  | [required] |

### Return type

[**models::DeleteLabelDefResponse**](DeleteLabelDefResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_field_group_history

> models::GetFieldGroupHistoryResponse get_field_group_history(owner, dataset_name, label_group_name, label_name, older_than_version)
Get history for a field group

Get the history for a field group by name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**label_name** | **String** |  | [required] |
**older_than_version** | Option<**i32**> | Start from below this model version (for pagination) |  |

### Return type

[**models::GetFieldGroupHistoryResponse**](GetFieldGroupHistoryResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_field_history

> models::GetFieldHistoryResponse get_field_history(owner, dataset_name, label_group_name, label_name, field_id, older_than_version)
Get history for a field

Get the history for a specific field within a field group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**label_name** | **String** |  | [required] |
**field_id** | **String** |  | [required] |
**older_than_version** | Option<**i32**> | Start from below this model version (for pagination) |  |

### Return type

[**models::GetFieldHistoryResponse**](GetFieldHistoryResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_def

> models::GetLabelDefResponse get_label_def(owner, dataset_name, label_group_name, label_name)
Get Label Def

Get Label Def

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**label_name** | **String** |  | [required] |

### Return type

[**models::GetLabelDefResponse**](GetLabelDefResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_label_def

> models::UpdateLabelDefResponse update_label_def(owner, dataset_name, label_group_name, label_name, label_def_update_request)
Update label def

Update label def

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**label_name** | **String** |  | [required] |
**label_def_update_request** | [**LabelDefUpdateRequest**](LabelDefUpdateRequest.md) |  | [required] |

### Return type

[**models::UpdateLabelDefResponse**](UpdateLabelDefResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

