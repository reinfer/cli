# \ModelsApi

All URIs are relative to *https://cloud.uipath.com/demo/demo/reinfer_*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_model_tag**](ModelsApi.md#delete_model_tag) | **DELETE** /api/_private/datasets/{owner}/{dataset_name}/model-tags/{tag_name} | Delete a model tag
[**get_all_models_in_dataset**](ModelsApi.md#get_all_models_in_dataset) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labellers | Get all the models for a dataset
[**get_comment_predictions**](ModelsApi.md#get_comment_predictions) | **POST** /api/v1/datasets/{owner}/{dataset_name}/labellers/{model_version}/predict-comments | Get predictions for a list of comments
[**get_label_validation**](ModelsApi.md#get_label_validation) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version}/label-validation | Get label validation for a dataset
[**get_model_tags**](ModelsApi.md#get_model_tags) | **GET** /api/v1/datasets/{owner}/{dataset_name}/model-tags | Get model tags
[**get_training_action_ordered_comment_uids**](ModelsApi.md#get_training_action_ordered_comment_uids) | **GET** /api/_private/datasets/{owner}/{dataset_name}/{training_action_id}/training-action-comment-uids | Get ordered comment uids for training action
[**get_training_actions_labels**](ModelsApi.md#get_training_actions_labels) | **GET** /api/_private/datasets/{owner}/{dataset_name}/{model_version}/training-actions/labels | Get ordered training actions for labels
[**get_validation**](ModelsApi.md#get_validation) | **GET** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version}/validation | Get validation for a dataset
[**get_validation_v1**](ModelsApi.md#get_validation_v1) | **GET** /api/v1/datasets/{owner}/{dataset_name}/labellers/{model_version}/validation | Get validation for a dataset
[**pin_model**](ModelsApi.md#pin_model) | **PUT** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version}/pin | Pin a model
[**predict**](ModelsApi.md#predict) | **POST** /api/v1/datasets/{owner}/{dataset_name}/labellers/{model_version}/predict | Get predictions from a specific version of a pinned model
[**predict_extractions**](ModelsApi.md#predict_extractions) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version}/predict-extractions | Get extraction predictions from a specific version of a pinned model
[**predict_latest**](ModelsApi.md#predict_latest) | **POST** /api/_private/datasets/{owner}/{dataset_name}/labellers/predict | Get predictions from the latest model
[**predict_raw_emails**](ModelsApi.md#predict_raw_emails) | **POST** /api/v1/datasets/{owner}/{dataset_name}/labellers/{model_version}/predict-raw-emails | Get predictions on raw emails from a specific model version
[**put_training_comment_seen**](ModelsApi.md#put_training_comment_seen) | **PUT** /api/_private/datasets/{owner}/{dataset_name}/{training_action_id}/{comment_uid}/seen-training-comment | Put a training comment as seen
[**unpin_model**](ModelsApi.md#unpin_model) | **DELETE** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version}/pin | Unpin a model
[**update_model**](ModelsApi.md#update_model) | **PATCH** /api/_private/datasets/{owner}/{dataset_name}/labellers/{model_version} | Update a model
[**update_model_tag**](ModelsApi.md#update_model_tag) | **PUT** /api/_private/datasets/{owner}/{dataset_name}/model-tags/{tag_name} | Update a model tag



## delete_model_tag

> models::DeleteModelTagResponse delete_model_tag(owner, dataset_name, tag_name)
Delete a model tag

Delete a model tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**tag_name** | **String** |  | [required] |

### Return type

[**models::DeleteModelTagResponse**](DeleteModelTagResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_models_in_dataset

> models::GetAllModelsInDatasetResponse get_all_models_in_dataset(owner, dataset_name, get_all_models_in_dataset_request)
Get all the models for a dataset

Get all the models for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**get_all_models_in_dataset_request** | [**GetAllModelsInDatasetRequest**](GetAllModelsInDatasetRequest.md) |  | [required] |

### Return type

[**models::GetAllModelsInDatasetResponse**](GetAllModelsInDatasetResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comment_predictions

> models::GetCommentPredictionsResponse get_comment_predictions(owner, dataset_name, model_version, get_comment_predictions_request)
Get predictions for a list of comments

Get predictions for a list of comments

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**get_comment_predictions_request** | [**GetCommentPredictionsRequest**](GetCommentPredictionsRequest.md) |  | [required] |

### Return type

[**models::GetCommentPredictionsResponse**](GetCommentPredictionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_validation

> models::GetLabelValidationResponse get_label_validation(owner, dataset_name, model_version, get_label_validation_request, beta)
Get label validation for a dataset

Get label validation for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**get_label_validation_request** | [**GetLabelValidationRequest**](GetLabelValidationRequest.md) |  | [required] |
**beta** | Option<**bool**> | Show beta features |  |

### Return type

[**models::GetLabelValidationResponse**](GetLabelValidationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_model_tags

> models::GetModelTagsResponse get_model_tags(owner, dataset_name)
Get model tags

Get model tags

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |

### Return type

[**models::GetModelTagsResponse**](GetModelTagsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_training_action_ordered_comment_uids

> models::GetTrainingActionsCommentUidsResponse get_training_action_ordered_comment_uids(owner, dataset_name, training_action_id)
Get ordered comment uids for training action

Get ordered comment uids for training action

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** | Owner | [required] |
**dataset_name** | **String** | Dataset Name | [required] |
**training_action_id** | **String** | Training Action Id. This is a UUID and may be in any `str` formats allowed by Python's `uuid.UUID`. | [required] |

### Return type

[**models::GetTrainingActionsCommentUidsResponse**](GetTrainingActionsCommentUidsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_training_actions_labels

> models::GetTrainingActionsLabelsResponse get_training_actions_labels(owner, dataset_name, model_version, beta)
Get ordered training actions for labels

Get ordered training actions for labels

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** | Owner | [required] |
**dataset_name** | **String** | Dataset Name | [required] |
**model_version** | **String** | Model Version | [required] |
**beta** | Option<**bool**> | Show beta features |  |

### Return type

[**models::GetTrainingActionsLabelsResponse**](GetTrainingActionsLabelsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_validation

> models::GetValidationResponse get_validation(owner, dataset_name, model_version, beta)
Get validation for a dataset

Get validation for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**beta** | Option<**bool**> | Show beta features |  |

### Return type

[**models::GetValidationResponse**](GetValidationResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_validation_v1

> models::GetValidationV1Response get_validation_v1(owner, dataset_name, model_version)
Get validation for a dataset

Get validation for a dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |

### Return type

[**models::GetValidationV1Response**](GetValidationV1Response.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pin_model

> models::PinModelResponse pin_model(owner, dataset_name, model_version)
Pin a model

Pin a model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |

### Return type

[**models::PinModelResponse**](PinModelResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## predict

> models::PredictResponse predict(owner, dataset_name, model_version, predict_request)
Get predictions from a specific version of a pinned model

Get predictions from a specific version of a pinned model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**predict_request** | [**PredictRequest**](PredictRequest.md) |  | [required] |

### Return type

[**models::PredictResponse**](PredictResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## predict_extractions

> models::PredictExtractionsResponse predict_extractions(owner, dataset_name, model_version, predict_extractions_request)
Get extraction predictions from a specific version of a pinned model

Get extraction predictions from a specific version of a pinned model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**predict_extractions_request** | [**PredictExtractionsRequest**](PredictExtractionsRequest.md) |  | [required] |

### Return type

[**models::PredictExtractionsResponse**](PredictExtractionsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## predict_latest

> models::PredictLatestResponse predict_latest(owner, dataset_name, predict_latest_request)
Get predictions from the latest model

Get predictions from the latest model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**predict_latest_request** | [**PredictLatestRequest**](PredictLatestRequest.md) |  | [required] |

### Return type

[**models::PredictLatestResponse**](PredictLatestResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## predict_raw_emails

> models::PredictRawEmailsResponse predict_raw_emails(owner, dataset_name, model_version, predict_raw_emails_request)
Get predictions on raw emails from a specific model version

Get predictions on raw emails from a specific model version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**predict_raw_emails_request** | [**PredictRawEmailsRequest**](PredictRawEmailsRequest.md) |  | [required] |

### Return type

[**models::PredictRawEmailsResponse**](PredictRawEmailsResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_training_comment_seen

> models::PutCommentAsSeenResponse put_training_comment_seen(owner, dataset_name, training_action_id, comment_uid, put_comment_as_seen_request)
Put a training comment as seen

Put a training comment as seen, and perhaps with annotation

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** | Owner | [required] |
**dataset_name** | **String** | Dataset Name | [required] |
**training_action_id** | **String** | Training Action Id | [required] |
**comment_uid** | **String** | Comment Uid | [required] |
**put_comment_as_seen_request** | [**PutCommentAsSeenRequest**](PutCommentAsSeenRequest.md) |  | [required] |

### Return type

[**models::PutCommentAsSeenResponse**](PutCommentAsSeenResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unpin_model

> models::UnpinModelResponse unpin_model(owner, dataset_name, model_version)
Unpin a model

Unpin a model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |

### Return type

[**models::UnpinModelResponse**](UnpinModelResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_model

> models::UpdateModelResponse update_model(owner, dataset_name, model_version, update_model_request)
Update a model

Update a model

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**model_version** | **String** |  | [required] |
**update_model_request** | [**UpdateModelRequest**](UpdateModelRequest.md) |  | [required] |

### Return type

[**models::UpdateModelResponse**](UpdateModelResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_model_tag

> models::UpdateModelTagResponse update_model_tag(owner, dataset_name, tag_name, update_model_tag_request)
Update a model tag

Update a model tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**owner** | **String** |  | [required] |
**dataset_name** | **String** |  | [required] |
**tag_name** | **String** |  | [required] |
**update_model_tag_request** | [**UpdateModelTagRequest**](UpdateModelTagRequest.md) |  | [required] |

### Return type

[**models::UpdateModelTagResponse**](UpdateModelTagResponse.md)

### Authorization

[APIKeyHeader](../README.md#APIKeyHeader)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

