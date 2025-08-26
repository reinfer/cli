# PredictResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** |  | 
**predictions** | [**Vec<Vec<models::PredictedLabel>>**](Vec.md) |  | 
**model** | Option<[**models::PredictionsModel**](PredictionsModel.md)> |  | 
**entities** | Option<[**Vec<Vec<models::PredictedEntity>>**](Vec.md)> |  | [optional]
**sentiment** | Option<[**Vec<models::CommentSentiment>**](CommentSentiment.md)> |  | [optional]
**comments** | Option<[**Vec<models::ParseEmailNewComment>**](ParseEmailNewComment.md)> |  | [optional]
**label_properties** | Option<[**Vec<Vec<models::DocumentLabelPropertyPrediction>>**](Vec.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


