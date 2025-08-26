# \SourcesApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_source**](SourcesApi.md#create_source) | **PUT** /api/v1/sources/{owner}/{source_name} | Create a source
[**delete_source**](SourcesApi.md#delete_source) | **DELETE** /api/v1/sources/id:{source_id} | Delete a source
[**get_all_sources**](SourcesApi.md#get_all_sources) | **GET** /api/v1/sources | Get all sources
[**get_all_sources_in_project**](SourcesApi.md#get_all_sources_in_project) | **GET** /api/v1/sources/{owner} | Get all sources in a project
[**get_email_transform_tag_info**](SourcesApi.md#get_email_transform_tag_info) | **GET** /api/_private/sources/email-transform/tag:{transform_tag} | Get info about email transform
[**get_source**](SourcesApi.md#get_source) | **GET** /api/v1/sources/{owner}/{source_name} | Get a source by project and name
[**get_source_by_id**](SourcesApi.md#get_source_by_id) | **GET** /api/v1/sources/id:{source_id} | Get a source by ID
[**get_source_statistics**](SourcesApi.md#get_source_statistics) | **POST** /api/v1/sources/{owner}/{source_name}/statistics | Get source statistics
[**get_threads_by_source**](SourcesApi.md#get_threads_by_source) | **GET** /api/v1/sources/{owner}/{source_name}/threads | Get threads by source
[**update_source**](SourcesApi.md#update_source) | **POST** /api/v1/sources/{owner}/{source_name} | Update a source



## create_source

> models::CreateSourceResponse create_source(owner, source_name, create_source_request)
Create a source

Create a source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**create_source_request** | [**CreateSourceRequest**](CreateSourceRequest.md) |  | [required] |

### Return type

[**models::CreateSourceResponse**](CreateSourceResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_source

> models::DeleteSourceResponse delete_source(source_id)
Delete a source

Delete a source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source_id** | **String** |  | [required] |

### Return type

[**models::DeleteSourceResponse**](DeleteSourceResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_sources

> models::GetAllSourcesResponse get_all_sources()
Get all sources

Get all sources

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllSourcesResponse**](GetAllSourcesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_sources_in_project

> models::GetAllSourcesInProjectResponse get_all_sources_in_project(owner)
Get all sources in a project

Get all sources in a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |

### Return type

[**models::GetAllSourcesInProjectResponse**](GetAllSourcesInProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_email_transform_tag_info

> models::GetEmailTransformTagInfoResponse get_email_transform_tag_info(transform_tag)
Get info about email transform

Get info about email transform

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**transform_tag** | **String** |  | [required] |

### Return type

[**models::GetEmailTransformTagInfoResponse**](GetEmailTransformTagInfoResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_source

> models::GetSourceResponse get_source(owner, source_name)
Get a source by project and name

Get a source by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |

### Return type

[**models::GetSourceResponse**](GetSourceResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_source_by_id

> models::GetSourceByIdResponse get_source_by_id(source_id)
Get a source by ID

Get a source by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source_id** | **String** |  | [required] |

### Return type

[**models::GetSourceByIdResponse**](GetSourceByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_source_statistics

> models::GetSourceStatisticsResponse get_source_statistics(owner, source_name, get_source_statistics_request)
Get source statistics

Get source statistics

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**get_source_statistics_request** | [**GetSourceStatisticsRequest**](GetSourceStatisticsRequest.md) |  | [required] |

### Return type

[**models::GetSourceStatisticsResponse**](GetSourceStatisticsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_threads_by_source

> models::GetThreadsBySourceResponse get_threads_by_source(owner, source_name)
Get threads by source

Get threads by source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |

### Return type

[**models::GetThreadsBySourceResponse**](GetThreadsBySourceResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_source

> models::UpdateSourceResponse update_source(owner, source_name, update_source_request)
Update a source

Update a source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**source_name** | **String** |  | [required] |
**update_source_request** | [**UpdateSourceRequest**](UpdateSourceRequest.md) |  | [required] |

### Return type

[**models::UpdateSourceResponse**](UpdateSourceResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

