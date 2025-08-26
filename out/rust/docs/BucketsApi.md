# \BucketsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_bucket**](BucketsApi.md#create_bucket) | **PUT** /api/_private/buckets/{owner}/{bucket_name} | Create a bucket
[**delete_bucket**](BucketsApi.md#delete_bucket) | **DELETE** /api/_private/buckets/id:{raw_bucket_id} | Delete a bucket
[**delete_keyed_sync_state**](BucketsApi.md#delete_keyed_sync_state) | **DELETE** /api/_private/buckets/id:{raw_bucket_id}/keyed-sync-state/{sync_state_key} | Delete a keyed sync state by bucket Id.
[**get_all_buckets**](BucketsApi.md#get_all_buckets) | **GET** /api/_private/buckets | Get all buckets
[**get_bucket**](BucketsApi.md#get_bucket) | **GET** /api/_private/buckets/{owner}/{bucket_name} | Get a bucket
[**get_bucket_by_id**](BucketsApi.md#get_bucket_by_id) | **GET** /api/_private/buckets/id:{bucket_id} | Get a bucket by ID
[**get_bucket_statistics**](BucketsApi.md#get_bucket_statistics) | **GET** /api/_private/buckets/{owner}/{bucket_name}/statistics | Get bucket statistics
[**get_bucket_sync_state**](BucketsApi.md#get_bucket_sync_state) | **GET** /api/_private/bucket-sync-state/{owner}/{bucket_name} | Get bucket sync state
[**get_buckets_by_owner**](BucketsApi.md#get_buckets_by_owner) | **GET** /api/_private/buckets/{owner} | Get buckets by owner
[**get_keyed_sync_state**](BucketsApi.md#get_keyed_sync_state) | **GET** /api/_private/buckets/id:{raw_bucket_id}/keyed-sync-state/{sync_state_key} | Get a keyed sync state by bucket Id.
[**list_keyed_sync_states**](BucketsApi.md#list_keyed_sync_states) | **GET** /api/_private/buckets/id:{raw_bucket_id}/keyed-sync-states/ | List keyed sync states for a bucket.
[**query_keyed_sync_state_ids**](BucketsApi.md#query_keyed_sync_state_ids) | **POST** /api/_private/buckets/id:{raw_bucket_id}/keyed-sync-state-ids | Query keyed sync state ids for a bucket.
[**store_bucket_sync_state**](BucketsApi.md#store_bucket_sync_state) | **PUT** /api/_private/bucket-sync-state/{owner}/{bucket_name} | Store bucket sync state
[**store_keyed_sync_state**](BucketsApi.md#store_keyed_sync_state) | **PUT** /api/_private/buckets/id:{raw_bucket_id}/keyed-sync-state/{sync_state_key} | Store a keyed sync state by bucket Id.
[**update_bucket**](BucketsApi.md#update_bucket) | **POST** /api/_private/buckets/{owner}/{bucket_name} | Update a bucket



## create_bucket

> models::CreateBucketResponse create_bucket(owner, bucket_name, create_bucket_request)
Create a bucket

Create a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**create_bucket_request** | [**CreateBucketRequest**](CreateBucketRequest.md) |  | [required] |

### Return type

[**models::CreateBucketResponse**](CreateBucketResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_bucket

> models::DeleteBucketResponse delete_bucket(raw_bucket_id)
Delete a bucket

Delete a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |

### Return type

[**models::DeleteBucketResponse**](DeleteBucketResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_keyed_sync_state

> models::DeleteKeyedSyncStateResponse delete_keyed_sync_state(raw_bucket_id, sync_state_key)
Delete a keyed sync state by bucket Id.

Delete a keyed sync state by bucket Id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |
**sync_state_key** | **String** |  | [required] |

### Return type

[**models::DeleteKeyedSyncStateResponse**](DeleteKeyedSyncStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_buckets

> models::GetAllBucketsResponse get_all_buckets()
Get all buckets

Get all buckets

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllBucketsResponse**](GetAllBucketsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket

> models::GetBucketResponse get_bucket(owner, bucket_name)
Get a bucket

Get a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |

### Return type

[**models::GetBucketResponse**](GetBucketResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket_by_id

> models::GetBucketByIdResponse get_bucket_by_id(bucket_id)
Get a bucket by ID

Get a bucket by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** |  | [required] |

### Return type

[**models::GetBucketByIdResponse**](GetBucketByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket_statistics

> models::GetBucketStatisticsResponse get_bucket_statistics(owner, bucket_name)
Get bucket statistics

Get bucket statistics

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |

### Return type

[**models::GetBucketStatisticsResponse**](GetBucketStatisticsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket_sync_state

> models::GetBucketSyncStateResponse get_bucket_sync_state(owner, bucket_name)
Get bucket sync state

Get bucket sync state

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |

### Return type

[**models::GetBucketSyncStateResponse**](GetBucketSyncStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_buckets_by_owner

> models::GetBucketsByOwnerResponse get_buckets_by_owner(owner)
Get buckets by owner

Get buckets by owner

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |

### Return type

[**models::GetBucketsByOwnerResponse**](GetBucketsByOwnerResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_keyed_sync_state

> models::GetKeyedSyncStateResponse get_keyed_sync_state(raw_bucket_id, sync_state_key)
Get a keyed sync state by bucket Id.

Get a keyed sync state by bucket Id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |
**sync_state_key** | **String** |  | [required] |

### Return type

[**models::GetKeyedSyncStateResponse**](GetKeyedSyncStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_keyed_sync_states

> models::ListKeyedSyncStatesResponse list_keyed_sync_states(raw_bucket_id)
List keyed sync states for a bucket.

List keyed sync states for a bucket.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |

### Return type

[**models::ListKeyedSyncStatesResponse**](ListKeyedSyncStatesResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## query_keyed_sync_state_ids

> models::QueryKeyedSyncStateIdsResponse query_keyed_sync_state_ids(raw_bucket_id, query_keyed_sync_state_ids_request)
Query keyed sync state ids for a bucket.

Query keyed sync state ids for a bucket.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |
**query_keyed_sync_state_ids_request** | [**QueryKeyedSyncStateIdsRequest**](QueryKeyedSyncStateIdsRequest.md) |  | [required] |

### Return type

[**models::QueryKeyedSyncStateIdsResponse**](QueryKeyedSyncStateIdsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## store_bucket_sync_state

> models::StoreBucketSyncStateResponse store_bucket_sync_state(owner, bucket_name, store_bucket_sync_state_request)
Store bucket sync state

Store bucket sync state

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**store_bucket_sync_state_request** | [**StoreBucketSyncStateRequest**](StoreBucketSyncStateRequest.md) |  | [required] |

### Return type

[**models::StoreBucketSyncStateResponse**](StoreBucketSyncStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## store_keyed_sync_state

> models::StoreKeyedSyncStateResponse store_keyed_sync_state(raw_bucket_id, sync_state_key, store_keyed_sync_state_request)
Store a keyed sync state by bucket Id.

Store a keyed sync state by bucket Id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**raw_bucket_id** | **String** |  | [required] |
**sync_state_key** | **String** |  | [required] |
**store_keyed_sync_state_request** | [**StoreKeyedSyncStateRequest**](StoreKeyedSyncStateRequest.md) |  | [required] |

### Return type

[**models::StoreKeyedSyncStateResponse**](StoreKeyedSyncStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_bucket

> models::UpdateBucketResponse update_bucket(owner, bucket_name, update_bucket_request)
Update a bucket

Update a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**bucket_name** | **String** |  | [required] |
**update_bucket_request** | [**UpdateBucketRequest**](UpdateBucketRequest.md) |  | [required] |

### Return type

[**models::UpdateBucketResponse**](UpdateBucketResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

