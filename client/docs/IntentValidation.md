# IntentValidation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**average_precision** | Option<**f64**> |  | 
**train_size** | **i32** |  | 
**test_size** | **i32** |  | 
**reviewed_size** | **i32** |  | 
**recalls** | **Vec<f64>** |  | 
**test_error_count** | **f64** |  | 
**test_expected_count** | **f64** |  | 
**precisions** | **Vec<f64>** |  | 
**thresholds** | **Vec<f64>** |  | 
**optimal_thresholds** | Option<[**models::OptimalThreshold**](OptimalThreshold.md)> |  | 
**health** | [**models::LabelHealth**](LabelHealth.md) |  | 
**num_test_captures** | **i32** |  | 
**num_train_captures** | **i32** |  | 
**num_reviewed_captures** | **i32** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


