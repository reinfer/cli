use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        streams_api::store_exception,
    },
    models::{ExceptionModel, Metadata, StoreExceptionRequest},
};
use structopt::StructOpt;

use crate::utils::StreamFullName;

#[derive(Debug, StructOpt)]
pub struct CreateStreamExceptionArgs {
    #[structopt(long = "stream")]
    /// The stream full name, qualified by dataset, such as 'my-project-name/my-dataset-name/my-stream-name'.
    stream: StreamFullName,

    #[structopt(long = "type")]
    /// The type of exception. Please choose a short, easy-to-understand string such as "No Prediction".
    r#type: String,

    #[structopt(long = "uid")]
    /// The uid of the comment that should be tagged as an exception.
    uid: String,
}

pub fn create(config: &Configuration, args: &CreateStreamExceptionArgs, _printer: &Printer) -> Result<()> {
    let CreateStreamExceptionArgs {
        stream,
        r#type,
        uid,
    } = args;

    // Extract owner, dataset, and stream name from the StreamFullName
    let owner = stream.owner();
    let dataset_name = stream.dataset();
    let stream_name = stream.stream();

    let exception = ExceptionModel {
        uid: uid.clone(),
        metadata: Box::new(Metadata {
            r#type: r#type.clone(),
        }),
    };

    let request = StoreExceptionRequest {
        exceptions: vec![exception],
    };

    store_exception(config, owner, dataset_name, stream_name, request)
        .context("Operation to create a stream exception has failed")?;
    info!("New stream exception created successfully");
    Ok(())
}
