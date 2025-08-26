# \DatasetsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_dataset**](DatasetsApi.md#create_dataset) | **PUT** /api/v1/datasets/{owner}/{dataset_name} | Create a dataset
[**delete_dataset_by_id**](DatasetsApi.md#delete_dataset_by_id) | **DELETE** /api/v1/datasets/id:{dataset_id} | Delete a dataset by id
[**delete_dataset_v1**](DatasetsApi.md#delete_dataset_v1) | **DELETE** /api/v1/datasets/{owner}/{dataset_name} | Delete a dataset
[**export_dataset**](DatasetsApi.md#export_dataset) | **POST** /api/v1/datasets/{owner}/{dataset_name}/export | Export dataset
[**get_all_datasets**](DatasetsApi.md#get_all_datasets) | **GET** /api/v1/datasets | Get all datasets
[**get_all_datasets_in_project**](DatasetsApi.md#get_all_datasets_in_project) | **GET** /api/v1/datasets/{owner} | Get all datasets in a project
[**get_comparison**](DatasetsApi.md#get_comparison) | **POST** /api/_private/datasets/{owner}/{dataset_name}/compare | Get comparison for two filtered groups of verbatims
[**get_dataset**](DatasetsApi.md#get_dataset) | **GET** /api/v1/datasets/{owner}/{dataset_name} | Get a dataset by project and name
[**get_dataset_statistics**](DatasetsApi.md#get_dataset_statistics) | **POST** /api/_private/datasets/{owner}/{dataset_name}/statistics | Get dataset statistics
[**get_dataset_status**](DatasetsApi.md#get_dataset_status) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labellers/status | Get dataset status
[**get_dataset_summary**](DatasetsApi.md#get_dataset_summary) | **POST** /api/_private/datasets/{owner}/{dataset_name}/summary | Get dataset summary
[**get_dataset_user_properties_summary**](DatasetsApi.md#get_dataset_user_properties_summary) | **GET** /api/_private/datasets/{owner}/{dataset_name}/user-properties | Get user property summary for a dataset
[**get_labellings**](DatasetsApi.md#get_labellings) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labellings | Get labellings in a dataset
[**query_dataset_user_property_values**](DatasetsApi.md#query_dataset_user_property_values) | **POST** /api/_private/datasets/{owner}/{dataset_name}/user-properties/values | Query user property values for a dataset
[**reset_annotations_to_previous_pinned_model**](DatasetsApi.md#reset_annotations_to_previous_pinned_model) | **POST** /api/_private/datasets/{owner}/{dataset_name}/reset-annotations | Reset annotations to the previous pinned model
[**sync_annotations**](DatasetsApi.md#sync_annotations) | **POST** /api/v1/datasets/{owner}/{dataset_name}/sync-annotations | Sync annotations
[**update_comment_labelling**](DatasetsApi.md#update_comment_labelling) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labellings/{labelling_id} | Update the labelling of a comment
[**update_dataset**](DatasetsApi.md#update_dataset) | **POST** /api/v1/datasets/{owner}/{dataset_name} | Update a dataset



## create_dataset

> models::CreateDatasetResponse create_dataset(owner, dataset_name, create_dataset_request)
Create a dataset

Create a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**create_dataset_request** | [**CreateDatasetRequest**](CreateDatasetRequest.md) |  | [required] |

### Return type

[**models::CreateDatasetResponse**](CreateDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dataset_by_id

> models::DeleteDatasetByIdResponse delete_dataset_by_id(dataset_id)
Delete a dataset by id

Delete a dataset by id

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** |  | [required] |

### Return type

[**models::DeleteDatasetByIdResponse**](DeleteDatasetByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dataset_v1

> models::DeleteDatasetResponse delete_dataset_v1(owner, dataset_name)
Delete a dataset

Delete a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::DeleteDatasetResponse**](DeleteDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## export_dataset

> models::ExportDatasetResponse export_dataset(owner, dataset_name, export_dataset_request)
Export dataset

Export dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**export_dataset_request** | [**ExportDatasetRequest**](ExportDatasetRequest.md) |  | [required] |

### Return type

[**models::ExportDatasetResponse**](ExportDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_datasets

> models::GetAllDatasetsResponse get_all_datasets()
Get all datasets

Get all datasets

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllDatasetsResponse**](GetAllDatasetsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_datasets_in_project

> models::GetAllDatasetsInProjectResponse get_all_datasets_in_project(owner)
Get all datasets in a project

Get all datasets in a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |

### Return type

[**models::GetAllDatasetsInProjectResponse**](GetAllDatasetsInProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comparison

> models::GetComparisonResponse get_comparison(owner, dataset_name, get_comparison_request)
Get comparison for two filtered groups of verbatims

Get comparison for two filtered groups of verbatims

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**get_comparison_request** | [**GetComparisonRequest**](GetComparisonRequest.md) |  | [required] |

### Return type

[**models::GetComparisonResponse**](GetComparisonResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset

> models::GetDatasetResponse get_dataset(owner, dataset_name)
Get a dataset by project and name

Get a dataset by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetDatasetResponse**](GetDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset_statistics

> models::GetDatasetStatisticsResponse get_dataset_statistics(owner, dataset_name, get_dataset_statistics_request)
Get dataset statistics

Get dataset statistics

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**get_dataset_statistics_request** | [**GetDatasetStatisticsRequest**](GetDatasetStatisticsRequest.md) |  | [required] |

### Return type

[**models::GetDatasetStatisticsResponse**](GetDatasetStatisticsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset_status

> models::GetDatasetStatusResponse get_dataset_status(owner, dataset_name)
Get dataset status

Get dataset status

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetDatasetStatusResponse**](GetDatasetStatusResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset_summary

> models::GetDatasetSummaryResponse get_dataset_summary(owner, dataset_name, get_dataset_summary_request)
Get dataset summary

Get dataset summary

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**get_dataset_summary_request** | [**GetDatasetSummaryRequest**](GetDatasetSummaryRequest.md) |  | [required] |

### Return type

[**models::GetDatasetSummaryResponse**](GetDatasetSummaryResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset_user_properties_summary

> models::GetDatasetUserPropertiesSummaryResponse get_dataset_user_properties_summary(owner, dataset_name)
Get user property summary for a dataset

Get user property summary for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetDatasetUserPropertiesSummaryResponse**](GetDatasetUserPropertiesSummaryResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_labellings

> models::GetLabellingsResponse get_labellings(owner, dataset_name, id, ids, allow_missing, compute_moon_predictions, source_id, after, limit, return_predictions)
Get labellings in a dataset

Gets labellings in a dataset for a given set of comment         ids. Note: Calling this endpoint with `compute_moon_predictions` will         be significantly slower.         

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**id** | Option<[**Vec<String>**](String.md)> | List of comment IDs to get labellings for. Each ID should be in the format 'source_id.comment_id'. Maximum 128 IDs allowed. |  |
**ids** | Option<**String**> | Comma-separated list of comment IDs (old style). Each ID should be in the format 'source_id.comment_id'. |  |
**allow_missing** | Option<**String**> | Whether to allow missing comments in the response. If true, the response will include partial results even if some comment IDs are not found. |  |
**compute_moon_predictions** | Option<**String**> | Whether to compute moon predictions for the comments. WARNING: This will make the request significantly slower. |  |
**source_id** | Option<**String**> | Source ID for bulk requests. When provided, will return labellings for all comments in the source with pagination. |  |
**after** | Option<**String**> | Continuation token for pagination in bulk requests. Use this to fetch the next batch of labellings after a specific point. |  |
**limit** | Option<**String**> | Maximum number of labellings to return in bulk requests. Must be between 1 and 999. |  |
**return_predictions** | Option<**String**> | Whether to return predictions in bulk requests. |  |

### Return type

[**models::GetLabellingsResponse**](GetLabellingsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## query_dataset_user_property_values

> models::QueryDatasetUserPropertyValuesResponse query_dataset_user_property_values(owner, dataset_name, query_dataset_user_property_values_request)
Query user property values for a dataset

Query user property values for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**query_dataset_user_property_values_request** | [**QueryDatasetUserPropertyValuesRequest**](QueryDatasetUserPropertyValuesRequest.md) |  | [required] |

### Return type

[**models::QueryDatasetUserPropertyValuesResponse**](QueryDatasetUserPropertyValuesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_annotations_to_previous_pinned_model

> models::ResetAnnotationsToPreviousPinnedModelResponse reset_annotations_to_previous_pinned_model(owner, dataset_name, reset_annotations_to_previous_pinned_model_request)
Reset annotations to the previous pinned model

Reset annotations to the previous pinned model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**reset_annotations_to_previous_pinned_model_request** | [**ResetAnnotationsToPreviousPinnedModelRequest**](ResetAnnotationsToPreviousPinnedModelRequest.md) |  | [required] |

### Return type

[**models::ResetAnnotationsToPreviousPinnedModelResponse**](ResetAnnotationsToPreviousPinnedModelResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_annotations

> models::SyncAnnotationsResponse sync_annotations(owner, dataset_name, sync_annotations_request)
Sync annotations

Sync annotations

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**sync_annotations_request** | [**SyncAnnotationsRequest**](SyncAnnotationsRequest.md) |  | [required] |

### Return type

[**models::SyncAnnotationsResponse**](SyncAnnotationsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_comment_labelling

> models::UpdateCommentLabellingResponse update_comment_labelling(owner, dataset_name, labelling_id, update_comment_labelling_request)
Update the labelling of a comment

Update the labelling of a comment

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**labelling_id** | **String** |  | [required] |
**update_comment_labelling_request** | [**UpdateCommentLabellingRequest**](UpdateCommentLabellingRequest.md) |  | [required] |

### Return type

[**models::UpdateCommentLabellingResponse**](UpdateCommentLabellingResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_dataset

> models::UpdateDatasetResponse update_dataset(owner, dataset_name, update_dataset_request)
Update a dataset

Update a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**update_dataset_request** | [**UpdateDatasetRequest**](UpdateDatasetRequest.md) |  | [required] |

### Return type

[**models::UpdateDatasetResponse**](UpdateDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

