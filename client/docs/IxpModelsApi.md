# \IxpModelsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_ixp_consumability**](IxpModelsApi.md#get_ixp_consumability) | **GET** /api/_private/ixp/consumability | Get IXP consumability information for the tenant
[**get_ixp_models**](IxpModelsApi.md#get_ixp_models) | **GET** /api/_private/ixp/projects/{project_uuid}/models | Get all pinned IXP models in a Project
[**get_ixp_predictions**](IxpModelsApi.md#get_ixp_predictions) | **GET** /api/_private/ixp/projects/{project_uuid}/models/{model_version}/documents/{document_id}/extractions | Get IXP Document Predictions
[**upload_ixp_document**](IxpModelsApi.md#upload_ixp_document) | **PUT** /api/_private/ixp/projects/{project_uuid}/models/{model_version}/documents | Upload IXP Document for Runtime Predictions



## get_ixp_consumability

> models::GetIxpConsumabilityInfoResponse get_ixp_consumability()
Get IXP consumability information for the tenant

Get IXP consumability information for the tenant

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetIxpConsumabilityInfoResponse**](GetIxpConsumabilityInfoResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ixp_models

> models::GetAllIxpModelsInProjectResponse get_ixp_models(project_uuid)
Get all pinned IXP models in a Project

Get all pinned IXP models in a Project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_uuid** | **String** |  | [required] |

### Return type

[**models::GetAllIxpModelsInProjectResponse**](GetAllIxpModelsInProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ixp_predictions

> models::IxpPredictExtractionsResponse get_ixp_predictions(project_uuid, model_version, document_id)
Get IXP Document Predictions

Get IXP Document Predictions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_uuid** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**document_id** | **String** |  | [required] |

### Return type

[**models::IxpPredictExtractionsResponse**](IxpPredictExtractionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_ixp_document

> models::IxpUploadDocumentResponse upload_ixp_document(project_uuid, model_version, file)
Upload IXP Document for Runtime Predictions

Upload IXP Document for Predictions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_uuid** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**file** | Option<**std::path::PathBuf**> |  |  |

### Return type

[**models::IxpUploadDocumentResponse**](IxpUploadDocumentResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

