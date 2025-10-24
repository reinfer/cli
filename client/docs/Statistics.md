# Statistics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**thread_mode** | **bool** |  | 
**by_labels** | **Vec<String>** |  | 
**by_label_properties** | **Vec<String>** |  | 
**num_comments** | **f64** |  | 
**min_timestamp** | Option<**String**> |  | 
**max_timestamp** | Option<**String**> |  | 
**label_counts** | [**std::collections::HashMap<String, models::Sentiment>**](Sentiment.md) |  | 
**string_user_property_counts** | [**Vec<models::StringUserPropertyCount>**](StringUserPropertyCount.md) |  | 
**email_property_counts** | [**models::EmailPropertyCountsByProperty**](EmailPropertyCountsByProperty.md) |  | 
**thread_histogram_counts** | [**models::ThreadHistogramCountsByProperty**](ThreadHistogramCountsByProperty.md) |  | 
**source_counts** | Option<[**models::SourceCounts**](SourceCounts.md)> |  | [optional]
**label_timeseries** | Option<[**Vec<models::LabelTimeseriesEntry>**](LabelTimeseriesEntry.md)> |  | [optional]
**nps_summary** | Option<[**models::NpsSummary**](NpsSummary.md)> |  | [optional]
**label_property_summary** | Option<[**models::LabelPropertySummary**](LabelPropertySummary.md)> |  | [optional]
**timezone** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


