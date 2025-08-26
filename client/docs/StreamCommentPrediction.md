# StreamCommentPrediction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**taxonomies** | [**Vec<models::StreamTaxonomyPrediction>**](StreamTaxonomyPrediction.md) | Currently only one taxonomy can be defined per dataset, but for future compatibility search for the taxonomy named `default`. | 
**scores** | [**Vec<models::StreamScorePrediction>**](StreamScorePrediction.md) | The score predictions for the comment, such as Quality of Service or Tone, when these scores are enabled on the dataset. Scores with a value of 0.0 are omitted. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


