# StreamConfidence

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**value** | **f64** | A confidence value between 0 and 1. | 
**thresholds** | **Vec<String>** | A list of thresholds that have been cleared by this confidence value. There are builtin thresholds (`high_recall`, `high_precision`, `balanced`) as well as user-defined thresholds. Currently, if a threshold is set on a stream, it shows up as threshold `stream` called stream on the `occurence_confidence` of the corresponding label. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


