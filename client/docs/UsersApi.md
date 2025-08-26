# \UsersApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_support_tenant**](UsersApi.md#add_support_tenant) | **POST** /api/_private/users/support-tenants/add | Add a support tenant
[**create_user**](UsersApi.md#create_user) | **PUT** /api/_private/users | Create a user
[**delete_user**](UsersApi.md#delete_user) | **DELETE** /api/_private/users/{user_id} | Delete a user
[**get_reduced_permissions**](UsersApi.md#get_reduced_permissions) | **POST** /api/_private/permissions/reduced | Get the reduced permissions for a user
[**get_user_by_id**](UsersApi.md#get_user_by_id) | **GET** /api/_private/users/{user_id} | Get a user by id
[**get_users**](UsersApi.md#get_users) | **GET** /api/_private/users | Get all users
[**get_users_v1**](UsersApi.md#get_users_v1) | **GET** /api/v1/users | Get all users.
[**remove_support_tenant**](UsersApi.md#remove_support_tenant) | **POST** /api/_private/users/support-tenants/remove | Remove a support tenant
[**send_welcome_email**](UsersApi.md#send_welcome_email) | **POST** /api/_private/users/{user_id}/welcome-email | Send a welcome email
[**update_user**](UsersApi.md#update_user) | **POST** /api/_private/users/{user_id} | Update a user



## add_support_tenant

> models::AddSupportTenantResponse add_support_tenant(add_support_tenant_request)
Add a support tenant

Add a support tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**add_support_tenant_request** | [**AddSupportTenantRequest**](AddSupportTenantRequest.md) |  | [required] |

### Return type

[**models::AddSupportTenantResponse**](AddSupportTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_user

> models::CreateUserResponse create_user(create_user_request)
Create a user

Create a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_user_request** | [**CreateUserRequest**](CreateUserRequest.md) |  | [required] |

### Return type

[**models::CreateUserResponse**](CreateUserResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user

> models::DeleteUserResponse delete_user(user_id)
Delete a user

Delete a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

[**models::DeleteUserResponse**](DeleteUserResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_reduced_permissions

> models::GetReducedPermissionsResponse get_reduced_permissions(get_reduced_permissions_request)
Get the reduced permissions for a user

Get the reduced permissions for a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**get_reduced_permissions_request** | [**GetReducedPermissionsRequest**](GetReducedPermissionsRequest.md) |  | [required] |

### Return type

[**models::GetReducedPermissionsResponse**](GetReducedPermissionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_by_id

> models::GetUserByIdResponse get_user_by_id(user_id)
Get a user by id

Get a user by id

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

[**models::GetUserByIdResponse**](GetUserByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users

> models::GetUsersResponse get_users()
Get all users

Get all users

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetUsersResponse**](GetUsersResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_v1

> models::GetUsersV1Response get_users_v1()
Get all users.

Get all users.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetUsersV1Response**](GetUsersV1Response.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_support_tenant

> models::RemoveSupportTenantResponse remove_support_tenant(remove_support_tenant_request)
Remove a support tenant

Remove a support tenant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remove_support_tenant_request** | [**RemoveSupportTenantRequest**](RemoveSupportTenantRequest.md) |  | [required] |

### Return type

[**models::RemoveSupportTenantResponse**](RemoveSupportTenantResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_welcome_email

> models::SendWelcomeEmailResponse send_welcome_email(user_id)
Send a welcome email

Send a welcome email

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

[**models::SendWelcomeEmailResponse**](SendWelcomeEmailResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user

> models::UpdateUserResponse update_user(user_id, update_user_request)
Update a user

Update a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |
**update_user_request** | [**UpdateUserRequest**](UpdateUserRequest.md) |  | [required] |

### Return type

[**models::UpdateUserResponse**](UpdateUserResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

