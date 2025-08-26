# MoonFormFieldAnnotationNew

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional]
**name** | Option<**String**> |  | [optional]
**span** | Option<[**models::CommentSpanNewUtf16**](CommentSpanNewUtf16.md)> | Deprecated. Use `spans` instead. | [optional]
**spans** | Option<[**Vec<models::CommentSpanNewUtf16>**](CommentSpanNewUtf16.md)> | Spans of the entity in the comment. | [optional]
**formatted_value** | **String** |  | 
**field_id** | Option<**String**> |  | [optional]
**document_spans** | Option<[**Vec<models::DocumentSpan>**](DocumentSpan.md)> |  | [optional]
**confirmed** | Option<**bool**> | Whether the field is confirmed. | [optional][default to true]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


