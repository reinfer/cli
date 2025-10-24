# Alert

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique ID of the alert | 
**name** | **String** | The API name of the alert | 
**owner** | **String** | The project which owns the alert | 
**created_at** | **String** | Timestamp when the alert was created | 
**updated_at** | **String** | Timestamp when the alert was updated | 
**revision** | **i32** | The latest revision of the alert | 
**title** | **String** | A short title for the alert | 
**description** | **String** | A longer description of the alert | 
**dataset_ids** | **Vec<String>** | The datasets associated with this alert | 
**subscribed_user_ids** | **Vec<String>** | The users subscribed to receive this alert | 
**config** | [**models::AlertConfig**](AlertConfig.md) | Alert Configuration | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


