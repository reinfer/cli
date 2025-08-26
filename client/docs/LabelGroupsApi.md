# \LabelGroupsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_label_group**](LabelGroupsApi.md#create_label_group) | **POST** /api/_private/datasets/{owner}/{dataset_name}/label-groups/{label_group_name} | Create a label group
[**delete_label_group**](LabelGroupsApi.md#delete_label_group) | **DELETE** /api/_private/datasets/{owner}/{dataset_name}/label-groups/{label_group_name} | Delete a label group
[**get_all_label_groups**](LabelGroupsApi.md#get_all_label_groups) | **GET** /api/_private/datasets/{owner}/{dataset_name}/label-groups | Get all label groups
[**get_label_group**](LabelGroupsApi.md#get_label_group) | **GET** /api/_private/datasets/{owner}/{dataset_name}/label-groups/{label_group_name} | Get a label group



## create_label_group

> models::CreateLabelGroupResponse create_label_group(owner, dataset_name, label_group_name, create_label_group_request)
Create a label group

Create a label group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |
**create_label_group_request** | [**CreateLabelGroupRequest**](CreateLabelGroupRequest.md) |  | [required] |

### Return type

[**models::CreateLabelGroupResponse**](CreateLabelGroupResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_label_group

> models::DeleteLabelGroupResponse delete_label_group(owner, dataset_name, label_group_name)
Delete a label group

Delete a label group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |

### Return type

[**models::DeleteLabelGroupResponse**](DeleteLabelGroupResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_label_groups

> models::GetAllLabelGroupsResponse get_all_label_groups(owner, dataset_name)
Get all label groups

Get all label groups

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetAllLabelGroupsResponse**](GetAllLabelGroupsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_group

> models::GetLabelGroupResponse get_label_group(owner, dataset_name, label_group_name)
Get a label group

Get a label group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**label_group_name** | **String** |  | [required] |

### Return type

[**models::GetLabelGroupResponse**](GetLabelGroupResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

