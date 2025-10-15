# \IxpProjectsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_ixp_project**](IxpProjectsApi.md#get_ixp_project) | **GET** /api/_private/ixp/projects/{project_uuid} | Get an IXP project
[**get_ixp_projects**](IxpProjectsApi.md#get_ixp_projects) | **GET** /api/_private/ixp/projects | Get all IXP projects
[**import_taxonomy**](IxpProjectsApi.md#import_taxonomy) | **POST** /api/_private/ixp/projects/{owner}/{dataset_name}/import-taxonomy | Import an IXP Taxonomy using a Dataset JSON file
[**suggest_taxonomy**](IxpProjectsApi.md#suggest_taxonomy) | **POST** /api/_private/ixp/projects/{owner}/{dataset_name}/suggest-taxonomy | Suggest an IXP Taxonomy from context



## get_ixp_project

> models::GetIxpProjectResponse get_ixp_project(project_uuid)
Get an IXP project

Get an IXP project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_uuid** | **String** |  | [required] |

### Return type

[**models::GetIxpProjectResponse**](GetIxpProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ixp_projects

> models::GetAllIxpProjectsResponse get_ixp_projects()
Get all IXP projects

Get all IXP projects

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllIxpProjectsResponse**](GetAllIxpProjectsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## import_taxonomy

> models::ImportTaxonomyResponse import_taxonomy(owner, dataset_name, import_taxonomy_request)
Import an IXP Taxonomy using a Dataset JSON file

Import an IXP Taxonomy using a Dataset JSON file

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**import_taxonomy_request** | [**ImportTaxonomyRequest**](ImportTaxonomyRequest.md) |  | [required] |

### Return type

[**models::ImportTaxonomyResponse**](ImportTaxonomyResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## suggest_taxonomy

> models::SuggestTaxonomyResponse suggest_taxonomy(owner, dataset_name, suggest_taxonomy_request)
Suggest an IXP Taxonomy from context

Suggest an IXP Taxonomy from context

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**suggest_taxonomy_request** | [**SuggestTaxonomyRequest**](SuggestTaxonomyRequest.md) |  | [required] |

### Return type

[**models::SuggestTaxonomyResponse**](SuggestTaxonomyResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

