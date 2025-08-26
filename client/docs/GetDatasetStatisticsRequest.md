# GetDatasetStatisticsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment_filter** | Option<[**models::CommentFilter**](CommentFilter.md)> |  | [optional]
**label_filter** | Option<**Vec<String>**> |  | [optional]
**by_labels** | Option<[**models::ByLabels**](By_Labels.md)> |  | [optional]
**by_label_properties** | [**Vec<models::LabelPropertyId>**](LabelPropertyId.md) |  | 
**time_resolution** | Option<[**models::TimeResolution**](TimeResolution.md)> |  | [optional]
**string_user_property_counts** | Option<[**Vec<models::StringUserPropertyCountsSettings>**](StringUserPropertyCountsSettings.md)> |  | [optional]
**email_property_counts** | Option<[**models::EmailPropertyCountsSettingsByProperty**](EmailPropertyCountsSettingsByProperty.md)> |  | [optional]
**source_counts** | Option<[**models::SourceCountsSettings**](SourceCountsSettings.md)> |  | [optional]
**label_timeseries** | Option<**bool**> |  | [optional]
**label_property_timeseries** | Option<**bool**> |  | [optional]
**thread_histogram** | Option<[**models::ThreadHistogramSettingsByProperty**](ThreadHistogramSettingsByProperty.md)> |  | [optional]
**nps_property** | Option<**String**> |  | [optional]
**attribute_filters** | Option<[**Vec<models::AttributeFilter>**](AttributeFilter.md)> |  | [optional]
**thread_mode** | Option<**bool**> |  | [optional]
**timezone** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


