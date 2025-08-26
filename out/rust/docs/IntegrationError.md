# IntegrationError

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique ID of the error | 
**integration_id** | **String** | Unique ID of the integration | 
**error_text** | **String** | The message on the thrown error | 
**error_details** | [**models::IntegrationErrorDetails**](IntegrationErrorDetails.md) | The type of server error that occurred | 
**error_action** | [**models::ErrorAction**](Error_action.md) |  | 
**created_at** | **String** | Timestamp when the error was created | 
**processed_at** | Option<**String**> | Timestamp when a notification was sent if possible | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


