# Source

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique ID of the source | 
**_kind** | [**models::SourceKind**](SourceKind.md) |  | 
**owner** | **String** | The project which owns the source | 
**name** | **String** | The API name of the source | 
**language** | **String** | The language of the messages in the source | 
**title** | **String** | A short description of the source | 
**description** | **String** | A longer description of the source | 
**should_translate** | **bool** | Whether messages in this source will be automatically translated | 
**sensitive_properties** | **Vec<String>** | User properties that require additional permissions to view | 
**created_at** | **String** | Timestamp when the source was created | 
**updated_at** | **String** | Timestamp when the source was last updated | 
**last_modified** | **String** | Timestamp when the source was last modified | 
**bucket_id** | Option<**String**> |  | [optional]
**email_transform_tag** | Option<**String**> | A tag for email parsing logic | [optional]
**email_transform_version** | Option<**i32**> | A version for parsing of this source with corresponding tag | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


