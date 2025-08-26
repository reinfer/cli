# \ProjectsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_project**](ProjectsApi.md#create_project) | **PUT** /api/_private/projects/{name} | Create a project
[**create_project_setup**](ProjectsApi.md#create_project_setup) | **PUT** /api/_private/solution-accelerator-project-setup/{name} | Create and set up a project
[**delete_project**](ProjectsApi.md#delete_project) | **DELETE** /api/_private/projects/{name} | Delete a project
[**get_all_projects**](ProjectsApi.md#get_all_projects) | **GET** /api/_private/projects | Get all projects
[**get_all_projects_v1**](ProjectsApi.md#get_all_projects_v1) | **GET** /api/v1/projects | Get all projects
[**get_project**](ProjectsApi.md#get_project) | **GET** /api/_private/projects/{name} | Get a project by name
[**get_project_resource_counts**](ProjectsApi.md#get_project_resource_counts) | **GET** /api/_private/projects/{name}/resource_counts | Get resource counts for a Project
[**update_project**](ProjectsApi.md#update_project) | **POST** /api/_private/projects/{name} | Update a project



## create_project

> models::CreateProjectResponse create_project(name, create_project_request)
Create a project

Create a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**create_project_request** | [**CreateProjectRequest**](CreateProjectRequest.md) |  | [required] |

### Return type

[**models::CreateProjectResponse**](CreateProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_project_setup

> models::CreateProjectSetupResponse create_project_setup(name, create_project_setup_request)
Create and set up a project

Create a project with provisioned resources

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**create_project_setup_request** | [**CreateProjectSetupRequest**](CreateProjectSetupRequest.md) |  | [required] |

### Return type

[**models::CreateProjectSetupResponse**](CreateProjectSetupResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_project

> models::DeleteProjectResponse delete_project(name, force)
Delete a project

Delete a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**force** | Option<**bool**> |  |  |

### Return type

[**models::DeleteProjectResponse**](DeleteProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_projects

> models::GetAllProjectsResponse get_all_projects(cm_only)
Get all projects

Get all projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cm_only** | Option<**bool**> | Only return CM projects |  |

### Return type

[**models::GetAllProjectsResponse**](GetAllProjectsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_projects_v1

> models::GetAllProjectsV1Response get_all_projects_v1()
Get all projects

Get all projects

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllProjectsV1Response**](GetAllProjectsV1Response.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_project

> models::GetProjectResponse get_project(name)
Get a project by name

Get a project by name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetProjectResponse**](GetProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_project_resource_counts

> models::GetProjectResourceCountsResponse get_project_resource_counts(name)
Get resource counts for a Project

Get resource counts for a Project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetProjectResourceCountsResponse**](GetProjectResourceCountsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_project

> models::UpdateProjectResponse update_project(name, update_project_request)
Update a project

Update a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_project_request** | [**UpdateProjectRequest**](UpdateProjectRequest.md) |  | [required] |

### Return type

[**models::UpdateProjectResponse**](UpdateProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

