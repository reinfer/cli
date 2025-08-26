# \ApplianceConfigsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_appliance_config**](ApplianceConfigsApi.md#get_appliance_config) | **GET** /api/_private/appliance-configs/{owner}/{config_key} | Get an appliance config
[**update_appliance_config**](ApplianceConfigsApi.md#update_appliance_config) | **PUT** /api/_private/appliance-configs/{owner}/{config_key} | Update an appliance config



## get_appliance_config

> std::path::PathBuf get_appliance_config(owner, config_key)
Get an appliance config

Get an appliance config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**config_key** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_appliance_config

> models::UpdateApplianceConfigResponse update_appliance_config(owner, config_key, body)
Update an appliance config

Update an appliance config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**config_key** | **String** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::UpdateApplianceConfigResponse**](UpdateApplianceConfigResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

