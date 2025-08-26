# SyncRawEmailsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**documents** | [**Vec<models::RawEmailDocument>**](RawEmailDocument.md) | A batch of at most 4096 raw emails | 
**include_comments** | Option<**bool**> | If set to true, the comments parsed from the emailswill be returned in the response body | [optional]
**transform_tag** | Option<**String**> | A tag identifying the email integration sending the data. You should have received this tag during integration configuration setup. | [optional]
**override_user_properties** | Option<**Vec<String>**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


