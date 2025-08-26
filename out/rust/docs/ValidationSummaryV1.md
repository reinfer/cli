# ValidationSummaryV1

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version** | **i32** | Version of the API | 
**labels** | [**Vec<models::LabelSummaryV1>**](_LabelSummaryV1.md) | List of labels | 
**entities** | [**Vec<models::EntitySummaryV1>**](_EntitySummaryV1.md) | List of entities | 
**num_reviewed_comments** | **i32** | Number of reviewed comments | 
**num_labels** | **i32** | Number of labels | 
**num_amber_labels** | **i32** | Number of amber labels | 
**num_red_labels** | **i32** | Number of red labels | 
**mean_average_precision_safe** | Option<**f64**> | Mean average precision safe | 
**dataset_quality** | [**models::DatasetQuality**](DatasetQuality.md) | Dataset quality | 
**dataset_score** | **i32** | Dataset score | 
**balance** | Option<**f64**> | Balance | 
**balance_quality** | Option<**String**> | Balance quality | 
**coverage** | Option<**f64**> | Coverage | 
**coverage_quality** | Option<[**models::DatasetQuality**](DatasetQuality.md)> |  | 
**underperforming_labels_quality** | Option<[**models::DatasetQuality**](DatasetQuality.md)> |  | 
**all_labels_quality** | Option<[**models::DatasetQuality**](DatasetQuality.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


