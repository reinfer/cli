# MoonFormFieldAnnotation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**field_type_id** | **String** |  | 
**name** | **String** |  | 
**span** | Option<[**models::TextSpan**](TextSpan.md)> | Deprecated. Will be set to the first span, if there's exactly one span. | [optional]
**spans** | [**Vec<models::TextSpan>**](TextSpan.md) |  | 
**kind** | **String** |  | 
**formatted_value** | **String** |  | 
**field_id** | **String** |  | 
**document_spans** | [**Vec<models::DocumentSpan>**](DocumentSpan.md) |  | 
**confirmed** | Option<**bool**> | Whether the field is confirmed. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


