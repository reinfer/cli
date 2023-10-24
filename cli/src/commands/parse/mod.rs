mod msgs;

use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

use self::msgs::ParseMsgArgs;

#[derive(Debug, StructOpt)]
pub enum ParseArgs {
    #[structopt(name = "msgs")]
    /// Parse unicode msg files. Note: Currently the body is processed as plain text.
    /// Html bodies are not supported.
    Msgs(ParseMsgArgs),
}

pub fn run(args: &ParseArgs, client: Client) -> Result<()> {
    match args {
        ParseArgs::Msgs(parse_msg_args) => msgs::parse(&client, parse_msg_args),
    }
}
