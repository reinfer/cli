# UserModelMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version** | **i32** | The version of the model | 
**model_id** | **String** | The type of model | 
**model_name** | [**models::ModelName**](ModelName.md) | The name of the model | 
**fingerprint** | **String** | The fingerprint of the model | 
**trained_time** | **String** | Timestamp when the model was trained | 
**training_duration** | **f64** | Duration of training in seconds | 
**input_updated_at** | **String** | Timestamp when the model was trained | 
**validated** | **bool** | Whether the model has been validated | 
**pinned** | **bool** | Whether the model is pinned | 
**reviewed_counts** | **std::collections::HashMap<String, i32>** | The number of times each label has been reviewed | 
**used_by_triggers** | **bool** | Whether the model is used by triggers | 
**settings** | [**models::UserModelMetadataSettings**](UserModelMetadataSettings.md) | Settings for the model | 
**freshness** | [**models::Freshness**](Freshness.md) | Whether the model is fresh, deprecated or unsupported | 
**flags** | [**Vec<models::UserModelMetadataFlag>**](UserModelMetadataFlag.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


