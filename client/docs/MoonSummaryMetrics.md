# MoonSummaryMetrics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**num_train_captures** | **i32** |  | 
**num_test_captures** | **i32** |  | 
**num_reviewed_captures** | **i32** |  | 
**num_train_documents** | **i32** |  | 
**num_test_documents** | **i32** |  | 
**num_reviewed_documents** | **i32** |  | 
**intents** | [**std::collections::HashMap<String, models::IntentsValue>**](Intents_value.md) |  | 
**recalls** | **Vec<f64>** |  | 
**mean_curve** | [**models::MeanPrCurve**](MeanPRCurve.md) |  | 
**average_field_stats** | [**models::EntityAverageStats**](EntityAverageStats.md) |  | 
**mean_capture_curve** | [**models::MeanPrCurve**](MeanPRCurve.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


