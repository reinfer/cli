# \PermissionsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_derived_permissions**](PermissionsApi.md#get_derived_permissions) | **POST** /api/_private/permissions/derived | Get the derived permissions for a user.



## get_derived_permissions

> models::GetDerivedPermissionResponse get_derived_permissions(get_derived_permissions_request)
Get the derived permissions for a user.

Get the derived permissions for a user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**get_derived_permissions_request** | [**GetDerivedPermissionsRequest**](GetDerivedPermissionsRequest.md) |  | [required] |

### Return type

[**models::GetDerivedPermissionResponse**](GetDerivedPermissionResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

