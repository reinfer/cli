# DatasetStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**training_complete** | Option<**f64**> |  | 
**validation_complete** | Option<**f64**> |  | 
**training_completion_time** | **String** |  | 
**inference_complete** | **f64** |  | 
**total_comments_with_text** | **i32** |  | 
**inference_stats_by_version** | [**std::collections::HashMap<String, models::InferenceStats>**](InferenceStats.md) |  | 
**inference_stuck** | **bool** |  | 
**operation** | Option<[**models::DatasetOperation**](DatasetOperation.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


