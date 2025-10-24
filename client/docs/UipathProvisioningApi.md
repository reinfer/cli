# \UipathProvisioningApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**uipath_provision_create**](UipathProvisioningApi.md#uipath_provision_create) | **POST** /api/tenantserviceinstances | Create resources for a Uipath tenant
[**uipath_provision_delete**](UipathProvisioningApi.md#uipath_provision_delete) | **DELETE** /api/tenantserviceinstances/{service_type}/{tenant_id} | Delete resources for a Uipath tenant
[**uipath_provision_restore**](UipathProvisioningApi.md#uipath_provision_restore) | **POST** /api/tenantserviceinstances/{service_type}/{tenant_id}/restore | Restore resources for a Uipath tenant
[**uipath_provision_update**](UipathProvisioningApi.md#uipath_provision_update) | **PATCH** /api/tenantserviceinstances/{service_type}/{tenant_id} | Update resources for a Uipath tenant



## uipath_provision_create

> models::UiPathProvisionCreateResponse uipath_provision_create(ui_path_provision_create_request)
Create resources for a Uipath tenant

Create resources for a Uipath tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ui_path_provision_create_request** | [**UiPathProvisionCreateRequest**](UiPathProvisionCreateRequest.md) |  | [required] |

### Return type

[**models::UiPathProvisionCreateResponse**](UiPathProvisionCreateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## uipath_provision_delete

> models::UiPathProvisionDeleteResponse uipath_provision_delete(service_type, tenant_id, ui_path_provision_delete_request)
Delete resources for a Uipath tenant

Delete resources for a Uipath tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service_type** | **String** |  | [required] |
**tenant_id** | **String** |  | [required] |
**ui_path_provision_delete_request** | [**UiPathProvisionDeleteRequest**](UiPathProvisionDeleteRequest.md) |  | [required] |

### Return type

[**models::UiPathProvisionDeleteResponse**](UiPathProvisionDeleteResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## uipath_provision_restore

> models::UiPathProvisionRestoreResponse uipath_provision_restore(service_type, tenant_id, ui_path_provision_restore_request)
Restore resources for a Uipath tenant

Restore resources for a Uipath tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service_type** | **String** |  | [required] |
**tenant_id** | **String** |  | [required] |
**ui_path_provision_restore_request** | [**UiPathProvisionRestoreRequest**](UiPathProvisionRestoreRequest.md) |  | [required] |

### Return type

[**models::UiPathProvisionRestoreResponse**](UiPathProvisionRestoreResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## uipath_provision_update

> models::UiPathProvisionUpdateResponse uipath_provision_update(service_type, tenant_id, ui_path_provision_update_request)
Update resources for a Uipath tenant

Update resources for a Uipath tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service_type** | **String** |  | [required] |
**tenant_id** | **String** |  | [required] |
**ui_path_provision_update_request** | [**UiPathProvisionUpdateRequest**](UiPathProvisionUpdateRequest.md) |  | [required] |

### Return type

[**models::UiPathProvisionUpdateResponse**](UiPathProvisionUpdateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

