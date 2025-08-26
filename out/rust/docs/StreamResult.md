# StreamResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment** | [**models::Comment**](Comment.md) | The comment itself---the data WILL reflect changes that have occurred since the data was first uploaded and the time this call was made. | 
**prediction** | Option<[**models::StreamCommentPrediction**](StreamCommentPrediction.md)> |  | 
**continuation** | **String** | An opaque token that can be used to advance the trigger past this result. If used, the next results to be returned will exclude all results that precede this one in this list. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


