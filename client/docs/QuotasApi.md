# \QuotasApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_quotas_for_tenant**](QuotasApi.md#get_quotas_for_tenant) | **GET** /api/_private/quotas/{target_tenant_id} | Get quotas for tenant
[**get_tenant_quota**](QuotasApi.md#get_tenant_quota) | **GET** /api/_private/quotas | Get quotas
[**reset_tenant_quota**](QuotasApi.md#reset_tenant_quota) | **DELETE** /api/_private/quotas/{target_tenant_id}/{quota_kind} | Reset tenant quota
[**set_quota_for_tenant**](QuotasApi.md#set_quota_for_tenant) | **POST** /api/_private/quotas/{target_tenant_id}/{quota_kind} | Set quota for tenant
[**set_tenant_quota**](QuotasApi.md#set_tenant_quota) | **POST** /api/_private/quotas/{quota_kind} | Set tenant quota



## get_quotas_for_tenant

> models::GetQuotasForTenantResponse get_quotas_for_tenant(target_tenant_id)
Get quotas for tenant

Get all quotas for a given tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target_tenant_id** | **String** |  | [required] |

### Return type

[**models::GetQuotasForTenantResponse**](GetQuotasForTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_quota

> models::GetTenantQuotaResponse get_tenant_quota()
Get quotas

Get all quotas in current tenant

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetTenantQuotaResponse**](GetTenantQuotaResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_tenant_quota

> models::ResetTenantQuotaResponse reset_tenant_quota(target_tenant_id, quota_kind)
Reset tenant quota

Reset a tenant quota to its default value

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target_tenant_id** | **String** |  | [required] |
**quota_kind** | **String** |  | [required] |

### Return type

[**models::ResetTenantQuotaResponse**](ResetTenantQuotaResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_quota_for_tenant

> models::SetQuotaForTenantResponse set_quota_for_tenant(target_tenant_id, quota_kind, set_quota_for_tenant_request)
Set quota for tenant

Set the value of a tenant quota

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target_tenant_id** | **String** |  | [required] |
**quota_kind** | **String** |  | [required] |
**set_quota_for_tenant_request** | [**SetQuotaForTenantRequest**](SetQuotaForTenantRequest.md) |  | [required] |

### Return type

[**models::SetQuotaForTenantResponse**](SetQuotaForTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tenant_quota

> models::SetTenantQuotaResponse set_tenant_quota(quota_kind, set_tenant_quota_request)
Set tenant quota

Set the quota on the current tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**quota_kind** | **String** |  | [required] |
**set_tenant_quota_request** | [**SetTenantQuotaRequest**](SetTenantQuotaRequest.md) |  | [required] |

### Return type

[**models::SetTenantQuotaResponse**](SetTenantQuotaResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

