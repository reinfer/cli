# SourceUpdate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**_kind** | Option<[**models::SourceKind**](SourceKind.md)> |  | [optional]
**language** | Option<[**models::Language**](Language.md)> |  | [optional]
**title** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**should_translate** | Option<**bool**> |  | [optional]
**sensitive_properties** | Option<**Vec<String>**> |  | [optional]
**bucket_id** | Option<**String**> |  | [optional]
**email_transform_tag** | Option<**String**> | A tag for email parsing logic. Changing this will cause a source to re-parse with other logic. | [optional]
**email_transform_version** | Option<**i32**> | A version for parsing of this source with corresponding tag. Incrementing this will cause the source to re-parse. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


