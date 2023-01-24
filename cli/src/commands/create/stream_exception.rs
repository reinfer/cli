use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    Client, CommentUid, StreamException, StreamExceptionMetadata, StreamFullName,
};
use structopt::StructOpt;

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
    uid: CommentUid,
}

pub fn create(client: &Client, args: &CreateStreamExceptionArgs, _printer: &Printer) -> Result<()> {
    let CreateStreamExceptionArgs {
        stream,
        r#type,
        uid,
    } = args;

    client
        .tag_stream_exceptions(
            stream,
            &[StreamException {
                metadata: StreamExceptionMetadata { r#type },
                uid,
            }],
        )
        .context("Operation to create a stream exception has failed")?;
    info!("New stream exception created successfully");
    Ok(())
}
