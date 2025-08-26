# PredictRawEmailsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**documents** | [**Vec<models::RawEmailDocument>**](RawEmailDocument.md) |  | 
**threshold** | Option<[**models::Threshold**](Threshold.md)> |  | [optional]
**labels** | Option<[**Vec<models::TriggerLabelThreshold>**](TriggerLabelThreshold.md)> |  | [optional]
**include_comments** | Option<**bool**> |  | [optional]
**transform_tag** | Option<**String**> | A tag identifying the email integration sending the data. You should have received this tag during integration configuration setup. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


