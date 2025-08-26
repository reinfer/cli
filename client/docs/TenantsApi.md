# \TenantsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_tenant**](TenantsApi.md#create_tenant) | **PUT** /api/_private/tenants/{name} | Create a new tenant
[**delete_tenant**](TenantsApi.md#delete_tenant) | **DELETE** /api/_private/tenants/{name} | Delete a tenant by name
[**delete_tenant_default_project_permissions**](TenantsApi.md#delete_tenant_default_project_permissions) | **DELETE** /api/_private/tenants/{name}/default_project_permissions | Clear default project permissions for a tenant
[**get_tenant**](TenantsApi.md#get_tenant) | **GET** /api/_private/tenants/{name} | Get a single tenant by name
[**get_tenant_client_subnets**](TenantsApi.md#get_tenant_client_subnets) | **GET** /api/_private/tenants/{name}/client_subnets | Get client subnets for a tenant
[**get_tenant_default_project_permissions**](TenantsApi.md#get_tenant_default_project_permissions) | **GET** /api/_private/tenants/{name}/default_project_permissions | Get default project permissions for a tenant
[**get_tenant_domains**](TenantsApi.md#get_tenant_domains) | **GET** /api/_private/tenants/{name}/domains | Get domains for a tenant
[**get_tenant_entity_def_ids**](TenantsApi.md#get_tenant_entity_def_ids) | **GET** /api/_private/tenants/{name}/entity_def_ids | Get entity_def_ids for a tenant
[**get_tenants**](TenantsApi.md#get_tenants) | **GET** /api/_private/tenants | Get tenants
[**set_tenant_state**](TenantsApi.md#set_tenant_state) | **POST** /api/_private/tenants/{name}/state | Sets the enabled or disabled state for a tenant
[**update_tenant**](TenantsApi.md#update_tenant) | **POST** /api/_private/tenants/{name} | Update a tenant
[**update_tenant_client_subnets**](TenantsApi.md#update_tenant_client_subnets) | **POST** /api/_private/tenants/{name}/client_subnets | Update client subnets for a tenant
[**update_tenant_default_project_permissions**](TenantsApi.md#update_tenant_default_project_permissions) | **POST** /api/_private/tenants/{name}/default_project_permissions | Update default project permissions for a tenant
[**update_tenant_domains**](TenantsApi.md#update_tenant_domains) | **POST** /api/_private/tenants/{name}/domains | Update domains for a tenant
[**update_tenant_entity_def_ids**](TenantsApi.md#update_tenant_entity_def_ids) | **POST** /api/_private/tenants/{name}/entity_def_ids | Update entity_def_ids for a tenant



## create_tenant

> models::CreateTenantResponse create_tenant(name, create_tenant_request)
Create a new tenant

Create a new tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**create_tenant_request** | [**CreateTenantRequest**](CreateTenantRequest.md) |  | [required] |

### Return type

[**models::CreateTenantResponse**](CreateTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tenant

> models::DeleteTenantResponse delete_tenant(name)
Delete a tenant by name

Delete a tenant by name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::DeleteTenantResponse**](DeleteTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tenant_default_project_permissions

> models::DeleteTenantDefaultProjectPermissionsResponse delete_tenant_default_project_permissions(name)
Clear default project permissions for a tenant

Clear default project permissions for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::DeleteTenantDefaultProjectPermissionsResponse**](DeleteTenantDefaultProjectPermissionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant

> models::SingleTenantsResponse get_tenant(name)
Get a single tenant by name

Get a single tenant by name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::SingleTenantsResponse**](SingleTenantsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_client_subnets

> models::GetTenantClientSubnetsResponse get_tenant_client_subnets(name)
Get client subnets for a tenant

Get client subnets for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetTenantClientSubnetsResponse**](GetTenantClientSubnetsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_default_project_permissions

> models::GetTenantDefaultProjectPermissionsResponse get_tenant_default_project_permissions(name)
Get default project permissions for a tenant

Get default project permissions for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetTenantDefaultProjectPermissionsResponse**](GetTenantDefaultProjectPermissionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_domains

> models::GetTenantDomainsResponse get_tenant_domains(name)
Get domains for a tenant

Get domains for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetTenantDomainsResponse**](GetTenantDomainsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenant_entity_def_ids

> models::GetTenantEntityDefIdsResponse get_tenant_entity_def_ids(name)
Get entity_def_ids for a tenant

Get entity_def_ids for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |

### Return type

[**models::GetTenantEntityDefIdsResponse**](GetTenantEntityDefIdsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tenants

> models::GetTenantsResponse get_tenants()
Get tenants

Get tenants

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetTenantsResponse**](GetTenantsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tenant_state

> models::SetTenantStateResponse set_tenant_state(name, set_tenant_state_request)
Sets the enabled or disabled state for a tenant

Sets the enabled or disabled state for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**set_tenant_state_request** | [**SetTenantStateRequest**](SetTenantStateRequest.md) |  | [required] |

### Return type

[**models::SetTenantStateResponse**](SetTenantStateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant

> models::UpdateTenantResponse update_tenant(name, update_tenant_request)
Update a tenant

Update a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_tenant_request** | [**UpdateTenantRequest**](UpdateTenantRequest.md) |  | [required] |

### Return type

[**models::UpdateTenantResponse**](UpdateTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant_client_subnets

> models::UpdateTenantClientSubnetsResponse update_tenant_client_subnets(name, update_tenant_client_subnets_request)
Update client subnets for a tenant

Update client subnets for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_tenant_client_subnets_request** | [**UpdateTenantClientSubnetsRequest**](UpdateTenantClientSubnetsRequest.md) |  | [required] |

### Return type

[**models::UpdateTenantClientSubnetsResponse**](UpdateTenantClientSubnetsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant_default_project_permissions

> models::UpdateTenantDefaultProjectPermissionsResponse update_tenant_default_project_permissions(name, update_tenant_default_project_permissions_request)
Update default project permissions for a tenant

Update default project permissions for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_tenant_default_project_permissions_request** | [**UpdateTenantDefaultProjectPermissionsRequest**](UpdateTenantDefaultProjectPermissionsRequest.md) |  | [required] |

### Return type

[**models::UpdateTenantDefaultProjectPermissionsResponse**](UpdateTenantDefaultProjectPermissionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant_domains

> models::UpdateTenantDomainsResponse update_tenant_domains(name, update_tenant_domains_request)
Update domains for a tenant

Update domains for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_tenant_domains_request** | [**UpdateTenantDomainsRequest**](UpdateTenantDomainsRequest.md) |  | [required] |

### Return type

[**models::UpdateTenantDomainsResponse**](UpdateTenantDomainsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tenant_entity_def_ids

> models::UpdateTenantEntityDefIdsResponse update_tenant_entity_def_ids(name, update_tenant_entity_def_ids_request)
Update entity_def_ids for a tenant

Update entity_def_ids for a tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**update_tenant_entity_def_ids_request** | [**UpdateTenantEntityDefIdsRequest**](UpdateTenantEntityDefIdsRequest.md) |  | [required] |

### Return type

[**models::UpdateTenantEntityDefIdsResponse**](UpdateTenantEntityDefIdsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

