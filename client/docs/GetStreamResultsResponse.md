# GetStreamResultsResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** | `ok` if request was successful, `error` otherwise | 
**results** | [**Vec<models::StreamResult>**](StreamResult.md) | New comments since last advance, with their predictions if the stream has a model selected. | 
**model** | Option<[**models::UserModel**](UserModel.md)> |  | [optional]
**num_filtered** | **i32** | How many comments were downloaded but filtered out by the stream's filter. | 
**more_results** | **bool** | True if there are more results after this batch---calling `fetch` after `advance` will return more results. | 
**continuation** | **String** | An opaque token that can be used to advance the trigger past the results in `results`. Individual results also contain `continuation`-s, but if you want to advance past ALL the results from this response, use this value as it will potentially advance past additional filtered values. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


