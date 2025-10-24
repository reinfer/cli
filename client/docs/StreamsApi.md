# \StreamsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**advance_stream**](StreamsApi.md#advance_stream) | **POST** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/advance | Advance stream
[**create_stream**](StreamsApi.md#create_stream) | **PUT** /api/v1/datasets/{owner}/{dataset_name}/streams | Create stream
[**delete_exception**](StreamsApi.md#delete_exception) | **DELETE** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/exceptions | Delete exception
[**delete_stream**](StreamsApi.md#delete_stream) | **DELETE** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name} | Delete stream
[**fetch_from_gx_stream**](StreamsApi.md#fetch_from_gx_stream) | **POST** /api/preview/datasets/{owner}/{dataset_name}/streams/{stream_name}/gx-fetch | Fetch from GX stream
[**fetch_from_stream**](StreamsApi.md#fetch_from_stream) | **POST** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/fetch | Fetch from stream
[**get_all_streams**](StreamsApi.md#get_all_streams) | **GET** /api/v1/datasets/{owner}/{dataset_name}/streams | Get all streams
[**get_stream_by_name**](StreamsApi.md#get_stream_by_name) | **GET** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name} | Get stream by name
[**get_stream_results**](StreamsApi.md#get_stream_results) | **GET** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/results | Get new comments from stream
[**get_stream_results_preview**](StreamsApi.md#get_stream_results_preview) | **GET** /api/preview/datasets/{owner}/{dataset_name}/streams/{stream_name}/results | Get new comments from stream
[**reset_stream**](StreamsApi.md#reset_stream) | **POST** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/reset | Reset stream
[**store_exception**](StreamsApi.md#store_exception) | **PUT** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name}/exceptions | Store exception
[**update_stream**](StreamsApi.md#update_stream) | **POST** /api/v1/datasets/{owner}/{dataset_name}/streams/{stream_name} | Update stream



## advance_stream

> models::AdvanceStreamResponse advance_stream(owner, dataset_name, stream_name, advance_stream_request)
Advance stream

Advance stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**advance_stream_request** | [**AdvanceStreamRequest**](AdvanceStreamRequest.md) |  | [required] |

### Return type

[**models::AdvanceStreamResponse**](AdvanceStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_stream

> models::CreateStreamResponse create_stream(owner, dataset_name, create_stream_request)
Create stream

Create stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**create_stream_request** | [**CreateStreamRequest**](CreateStreamRequest.md) |  | [required] |

### Return type

[**models::CreateStreamResponse**](CreateStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_exception

> models::DeleteStreamExceptionResponse delete_exception(owner, dataset_name, stream_name)
Delete exception

Delete exception

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |

### Return type

[**models::DeleteStreamExceptionResponse**](DeleteStreamExceptionResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_stream

> models::DeleteStreamResponse delete_stream(owner, dataset_name, stream_name)
Delete stream

Delete stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |

### Return type

[**models::DeleteStreamResponse**](DeleteStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_from_gx_stream

> models::FetchFromGxStreamResponse fetch_from_gx_stream(owner, dataset_name, stream_name, fetch_from_gx_stream_request)
Fetch from GX stream

Fetch from GX stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**fetch_from_gx_stream_request** | [**FetchFromGxStreamRequest**](FetchFromGxStreamRequest.md) |  | [required] |

### Return type

[**models::FetchFromGxStreamResponse**](FetchFromGXStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_from_stream

> models::FetchFromStreamResponse fetch_from_stream(owner, dataset_name, stream_name, fetch_from_stream_request)
Fetch from stream

Fetch from stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**fetch_from_stream_request** | [**FetchFromStreamRequest**](FetchFromStreamRequest.md) |  | [required] |

### Return type

[**models::FetchFromStreamResponse**](FetchFromStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_streams

> models::GetAllStreamsResponse get_all_streams(owner, dataset_name)
Get all streams

Get all streams

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetAllStreamsResponse**](GetAllStreamsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_stream_by_name

> models::GetStreamByNameResponse get_stream_by_name(owner, dataset_name, stream_name)
Get stream by name

Get stream by name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |

### Return type

[**models::GetStreamByNameResponse**](GetStreamByNameResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_stream_results

> models::GetStreamResultsResponse get_stream_results(owner, dataset_name, stream_name, max_results, max_filtered)
Get new comments from stream

Returns the next batch of comments from a stream together with their predictions, if the trigger has a model associated with it.  This route can be called multiple times and it will return the same results until it's explicitly advances using one of the continuations from the response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**max_results** | Option<**i32**> | The maximum number of results to return from this stream. |  |
**max_filtered** | Option<**i32**> | The maximum number of results to be skipped by the stream's filter before returning. |  |

### Return type

[**models::GetStreamResultsResponse**](GetStreamResultsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_stream_results_preview

> models::GetStreamResultsResponse get_stream_results_preview(owner, dataset_name, stream_name, max_results, max_filtered)
Get new comments from stream

Returns the next batch of comments from a stream together with their predictions, if the trigger has a model associated with it.  This route can be called multiple times and it will return the same results until it's explicitly advances using one of the continuations from the response. Now deprecated, please use `v1` route intead

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**max_results** | Option<**i32**> | The maximum number of results to return from this stream. |  |
**max_filtered** | Option<**i32**> | The maximum number of results to be skipped by the stream's filter before returning. |  |

### Return type

[**models::GetStreamResultsResponse**](GetStreamResultsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_stream

> models::ResetStreamResponse reset_stream(owner, dataset_name, stream_name, reset_stream_request)
Reset stream

Reset stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**reset_stream_request** | [**ResetStreamRequest**](ResetStreamRequest.md) |  | [required] |

### Return type

[**models::ResetStreamResponse**](ResetStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## store_exception

> models::StoreExceptionResponse store_exception(owner, dataset_name, stream_name, store_exception_request)
Store exception

Store exception

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**store_exception_request** | [**StoreExceptionRequest**](StoreExceptionRequest.md) |  | [required] |

### Return type

[**models::StoreExceptionResponse**](StoreExceptionResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_stream

> models::CreateStreamResponse update_stream(owner, dataset_name, stream_name, create_stream_request)
Update stream

Update stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**stream_name** | **String** |  | [required] |
**create_stream_request** | [**CreateStreamRequest**](CreateStreamRequest.md) |  | [required] |

### Return type

[**models::CreateStreamResponse**](CreateStreamResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

