# Integration

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique ID of the integration | 
**owner** | **String** | The project which owns the integration | 
**name** | **String** | The API name of the integration | 
**title** | **String** | A short description of the source | 
**created_at** | **String** | Timestamp when the integration was created | 
**updated_at** | **String** | Timestamp when the integration was last updated | 
**configuration** | [**serde_json::Value**](.md) | Custom settings to configure the integration | 
**enabled** | **bool** | If this integration is active | 
**r#type** | [**models::IntegrationType**](IntegrationType.md) | The type of the integration | 
**disabled_reason** | Option<[**models::IntegrationDisabledReason**](IntegrationDisabledReason.md)> | Reason for which the integration was disabled | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


