# GptIxpModelConfig

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**kind** | Option<**String**> |  | [optional][default to GptIxp]
**model_version** | Option<[**models::GptModelVersion**](GptModelVersion.md)> |  | [optional]
**input_config** | Option<[**models::InputConfig**](Input_Config.md)> |  | [optional]
**system_prompt_override** | Option<**String**> |  | [optional]
**frequency_penalty** | Option<**f64**> |  | [optional]
**temperature** | Option<**f64**> |  | [optional]
**top_p** | Option<**f64**> |  | [optional]
**seed** | Option<**i32**> |  | [optional]
**flags** | [**Vec<models::GptIxpFlag>**](GptIxpFlag.md) |  | 
**iterative_config** | Option<[**models::IterativeConfig**](IterativeConfig.md)> |  | [optional]
**attribution_method** | [**models::AttributionMethod**](AttributionMethod.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


