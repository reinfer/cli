# \DashboardsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_dashboard**](DashboardsApi.md#create_dashboard) | **PUT** /api/_private/dashboards/{owner}/{dashboard_name} | Create a new dashboard
[**delete_dashboard**](DashboardsApi.md#delete_dashboard) | **DELETE** /api/_private/dashboards/{owner}/{dashboard_name} | Delete a dashboard
[**get_dashboard**](DashboardsApi.md#get_dashboard) | **GET** /api/_private/dashboards/{owner}/{dashboard_name} | Get a dashboard by project and name
[**get_dashboards_in_dataset**](DashboardsApi.md#get_dashboards_in_dataset) | **GET** /api/_private/datasets/{owner}/{dataset_name}/dashboards | Get all dashboards in dataset
[**get_dashboards_in_project**](DashboardsApi.md#get_dashboards_in_project) | **GET** /api/_private/dashboards/{owner} | Get all dashboards in project
[**update_dashboard**](DashboardsApi.md#update_dashboard) | **POST** /api/_private/dashboards/{owner}/{dashboard_name} | Get a dashboard by project and name



## create_dashboard

> models::CreateDashboardResponse create_dashboard(owner, dashboard_name, create_dashboard_request)
Create a new dashboard

Create a new dashboard

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dashboard_name** | **String** |  | [required] |
**create_dashboard_request** | [**CreateDashboardRequest**](CreateDashboardRequest.md) |  | [required] |

### Return type

[**models::CreateDashboardResponse**](CreateDashboardResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dashboard

> models::DeleteDashboardResponse delete_dashboard(owner, dashboard_name)
Delete a dashboard

Delete a dashboard

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dashboard_name** | **String** |  | [required] |

### Return type

[**models::DeleteDashboardResponse**](DeleteDashboardResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dashboard

> models::GetDashboardResponse get_dashboard(owner, dashboard_name)
Get a dashboard by project and name

Get a dashboard by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dashboard_name** | **String** |  | [required] |

### Return type

[**models::GetDashboardResponse**](GetDashboardResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dashboards_in_dataset

> models::GetAllDashboardsInDatasetResponse get_dashboards_in_dataset(owner, dataset_name)
Get all dashboards in dataset

Get all dashboards in dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetAllDashboardsInDatasetResponse**](GetAllDashboardsInDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dashboards_in_project

> models::GetAllDashboardsInProjectResponse get_dashboards_in_project(owner)
Get all dashboards in project

Get all dashboards in project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |

### Return type

[**models::GetAllDashboardsInProjectResponse**](GetAllDashboardsInProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_dashboard

> models::UpdateDashboardResponse update_dashboard(owner, dashboard_name, update_dashboard_request)
Get a dashboard by project and name

Get a dashboard by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dashboard_name** | **String** |  | [required] |
**update_dashboard_request** | [**UpdateDashboardRequest**](UpdateDashboardRequest.md) |  | [required] |

### Return type

[**models::UpdateDashboardResponse**](UpdateDashboardResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

