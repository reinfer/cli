# \IxpProjectsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_ixp_project**](IxpProjectsApi.md#get_ixp_project) | **GET** /api/_private/ixp/projects/{project_uuid} | Get an IXP project
[**get_ixp_projects**](IxpProjectsApi.md#get_ixp_projects) | **GET** /api/_private/ixp/projects | Get all IXP projects



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

