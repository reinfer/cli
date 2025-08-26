# ValidationSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**fingerprint** | **String** |  | 
**model_kind** | **String** |  | 
**version** | **i32** |  | 
**training_time** | **f64** |  | 
**input_updated_at** | **String** |  | 
**trained_time** | **String** |  | 
**entities** | [**models::EntitiesPrStats**](EntitiesPRStats.md) |  | 
**train_size** | **i32** |  | 
**test_size** | **i32** |  | 
**reviewed_size** | **i32** |  | 
**min_comments_threshold** | **i32** |  | 
**mean_average_precision_safe** | Option<**f64**> |  | 
**recalls** | **Vec<f64>** |  | 
**mean_precision_curve_safe** | **Vec<f64>** |  | 
**labels** | [**std::collections::HashMap<String, models::LabelValidationSummary>**](LabelValidationSummary.md) |  | 
**model_rating** | [**models::ModelRating**](ModelRating.md) |  | 
**cooccurrence** | Option<[**models::Coocurrence**](Coocurrence.md)> |  | [optional]
**has_uninformative** | **bool** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


