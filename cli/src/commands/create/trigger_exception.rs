use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    Client, CommentUid, TriggerException, TriggerExceptionMetadata, TriggerFullName,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateTriggerExceptionArgs {
    #[structopt(short = "t", long = "trigger")]
    /// The dataset name or id
    trigger: TriggerFullName,

    #[structopt(long = "type")]
    /// The type of exception. Please choose a short, easy-to-understand string such as "No Prediction".
    r#type: String,

    #[structopt(long = "uid")]
    /// The uid of the comment that should be tagged as an exception.
    uid: CommentUid,
}

pub fn create(
    client: &Client,
    args: &CreateTriggerExceptionArgs,
    _printer: &Printer,
) -> Result<()> {
    let CreateTriggerExceptionArgs {
        trigger,
        r#type,
        uid,
    } = args;

    client
        .tag_trigger_exceptions(
            trigger,
            &[TriggerException {
                metadata: TriggerExceptionMetadata { r#type },
                uid,
            }],
        )
        .context("Operation to create a trigger exception has failed")?;
    info!("New trigger exception created successfully");
    Ok(())
}
