# \IntegrationsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_integration**](IntegrationsApi.md#create_integration) | **PUT** /api/_private/integrations/{owner}/{integration_name} | Create a integration
[**delete_integration**](IntegrationsApi.md#delete_integration) | **DELETE** /api/_private/integrations/id:{integration_id} | Delete a integration
[**get_all_integrations**](IntegrationsApi.md#get_all_integrations) | **GET** /api/_private/integrations | Get all integrations
[**get_all_integrations_in_project**](IntegrationsApi.md#get_all_integrations_in_project) | **GET** /api/_private/integrations/{owner} | Get all integrations in a project
[**get_integration**](IntegrationsApi.md#get_integration) | **GET** /api/_private/integrations/{owner}/{integration_name} | Get a integration by project and name
[**get_integration_by_id**](IntegrationsApi.md#get_integration_by_id) | **GET** /api/_private/integrations/id:{integration_id} | Get a integration by ID
[**get_integration_errors**](IntegrationsApi.md#get_integration_errors) | **GET** /api/_private/integrations/{owner}/{integration_name}/errors | Get integration sync errors
[**integrations_o_auth2_authenticate**](IntegrationsApi.md#integrations_o_auth2_authenticate) | **GET** /api/_private/integrations/{owner}/{integration_name}/oauth2/authenticate | OAuth2 `authenticate` endpoint
[**o_auth2_salesforce_callback**](IntegrationsApi.md#o_auth2_salesforce_callback) | **PUT** /api/_private/salesforce/oauth2/callback | Salesforce OAuth2 `callback` endpoint
[**update_integration**](IntegrationsApi.md#update_integration) | **POST** /api/_private/integrations/{owner}/{integration_name} | Update a integration
[**validate_exchange_credentials**](IntegrationsApi.md#validate_exchange_credentials) | **PUT** /api/_private/validate-exchange-credentials | Validate Exchange credentials



## create_integration

> models::CreateIntegrationResponse create_integration(owner, integration_name, create_integration_request)
Create a integration

Create a integration

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**integration_name** | **String** |  | [required] |
**create_integration_request** | [**CreateIntegrationRequest**](CreateIntegrationRequest.md) |  | [required] |

### Return type

[**models::CreateIntegrationResponse**](CreateIntegrationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_integration

> models::DeleteIntegrationResponse delete_integration(integration_id)
Delete a integration

Delete a integration

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**integration_id** | **String** |  | [required] |

### Return type

[**models::DeleteIntegrationResponse**](DeleteIntegrationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_integrations

> models::GetAllIntegrationsResponse get_all_integrations()
Get all integrations

Get all integrations

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetAllIntegrationsResponse**](GetAllIntegrationsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_integrations_in_project

> models::GetAllIntegrationsInProjectResponse get_all_integrations_in_project(owner)
Get all integrations in a project

Get all integrations in a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |

### Return type

[**models::GetAllIntegrationsInProjectResponse**](GetAllIntegrationsInProjectResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_integration

> models::GetIntegrationResponse get_integration(owner, integration_name)
Get a integration by project and name

Get a integration by project and name

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**integration_name** | **String** |  | [required] |

### Return type

[**models::GetIntegrationResponse**](GetIntegrationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_integration_by_id

> models::GetIntegrationByIdResponse get_integration_by_id(integration_id)
Get a integration by ID

Get a integration by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**integration_id** | **String** |  | [required] |

### Return type

[**models::GetIntegrationByIdResponse**](GetIntegrationByIdResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_integration_errors

> models::GetIntegrationErrorsResponse get_integration_errors(owner, integration_name)
Get integration sync errors

Get integration sync errors

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**integration_name** | **String** |  | [required] |

### Return type

[**models::GetIntegrationErrorsResponse**](GetIntegrationErrorsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_o_auth2_authenticate

> models::IntegrationsOAuth2AuthenticateResponse integrations_o_auth2_authenticate(owner, integration_name)
OAuth2 `authenticate` endpoint

OAuth2 `authenticate` endpoint

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**integration_name** | **String** |  | [required] |

### Return type

[**models::IntegrationsOAuth2AuthenticateResponse**](IntegrationsOAuth2AuthenticateResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## o_auth2_salesforce_callback

> models::OAuth2SalesforceCallbackResponse o_auth2_salesforce_callback(o_auth2_salesforce_callback_request)
Salesforce OAuth2 `callback` endpoint

Salesforce OAuth2 `callback` endpoint

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**o_auth2_salesforce_callback_request** | [**OAuth2SalesforceCallbackRequest**](OAuth2SalesforceCallbackRequest.md) |  | [required] |

### Return type

[**models::OAuth2SalesforceCallbackResponse**](OAuth2SalesforceCallbackResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_integration

> models::UpdateIntegrationResponse update_integration(owner, integration_name, update_integration_request)
Update a integration

Update a integration

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**integration_name** | **String** |  | [required] |
**update_integration_request** | [**UpdateIntegrationRequest**](UpdateIntegrationRequest.md) |  | [required] |

### Return type

[**models::UpdateIntegrationResponse**](UpdateIntegrationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## validate_exchange_credentials

> models::ValidateExchangeCredentialsResponse validate_exchange_credentials(validate_exchange_credentials_request)
Validate Exchange credentials

Validate the provided Exchange credentials with the remote server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**validate_exchange_credentials_request** | [**ValidateExchangeCredentialsRequest**](ValidateExchangeCredentialsRequest.md) |  | [required] |

### Return type

[**models::ValidateExchangeCredentialsResponse**](ValidateExchangeCredentialsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

