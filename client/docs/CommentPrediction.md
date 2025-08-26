# CommentPrediction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **String** | Comment id | 
**labels** | [**Vec<models::PredictedLabel>**](PredictedLabel.md) | List of predicted labels | 
**entities** | Option<[**Vec<models::PredictedEntity>**](PredictedEntity.md)> | List of predicted entities | [optional]
**label_properties** | Option<[**Vec<models::PredictedLabelProperty>**](PredictedLabelProperty.md)> | List of predicted label properties | [optional]
**sentiment** | Option<[**models::CommentSentiment**](CommentSentiment.md)> | Predicted sentiment | [optional]
**translations** | Option<[**Vec<models::Translation>**](Translation.md)> | List of translations | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


