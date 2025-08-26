# \IxpDatasetsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_ixp_dataset**](IxpDatasetsApi.md#create_ixp_dataset) | **PUT** /api/_private/ixp/datasets | Create a new IXP dataset
[**delete_ixp_dataset**](IxpDatasetsApi.md#delete_ixp_dataset) | **DELETE** /api/_private/ixp/datasets/{dataset_id} | Delete an IXP dataset



## create_ixp_dataset

> models::CreateIxpDatasetResponse create_ixp_dataset(create_ixp_dataset_request)
Create a new IXP dataset

Create a new IXP dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_ixp_dataset_request** | [**CreateIxpDatasetRequest**](CreateIxpDatasetRequest.md) |  | [required] |

### Return type

[**models::CreateIxpDatasetResponse**](CreateIxpDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_ixp_dataset

> models::DeleteIxpDatasetResponse delete_ixp_dataset(dataset_id)
Delete an IXP dataset

Delete an IXP dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** |  | [required] |

### Return type

[**models::DeleteIxpDatasetResponse**](DeleteIxpDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

