# Message

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | Option<**String**> | Message sender | [optional]
**to** | Option<**Vec<String>**> | Message recipients | [optional]
**cc** | Option<**Vec<String>**> | Message cc field | [optional]
**bcc** | Option<**Vec<String>**> | Message bcc field | [optional]
**sent_at** | Option<**String**> | The time when the message was sent originally | [optional]
**language** | Option<**String**> | The original language of the message | [optional]
**body** | [**models::MessageRichText**](MessageRichText.md) | The body of the message | 
**subject** | Option<[**models::MessageText**](MessageText.md)> | The subject of the message | [optional]
**signature** | Option<[**models::MessageRichText**](MessageRichText.md)> | The signature of the message | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


