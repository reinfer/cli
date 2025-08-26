# \DeploymentApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_qualified_login_url**](DeploymentApi.md#get_qualified_login_url) | **POST** /api/_private/deployment/login-location | Get the fully qualified login url for an email address.
[**get_subdomain**](DeploymentApi.md#get_subdomain) | **GET** /api/_private/deployment/subdomain | Whether the request was made from a tenant subdomain.



## get_qualified_login_url

> models::GetQualifiedLoginUrlResponse get_qualified_login_url(get_qualified_login_url_request)
Get the fully qualified login url for an email address.

Get the fully qualified login url for an email address.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**get_qualified_login_url_request** | [**GetQualifiedLoginUrlRequest**](GetQualifiedLoginUrlRequest.md) |  | [required] |

### Return type

[**models::GetQualifiedLoginUrlResponse**](GetQualifiedLoginUrlResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_subdomain

> models::GetSubdomainResponse get_subdomain()
Whether the request was made from a tenant subdomain.

Whether the request was made from a tenant subdomain.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetSubdomainResponse**](GetSubdomainResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

