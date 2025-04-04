use crate::package::Package;
use std::path::PathBuf;

use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UploadPackageArgs {
    /// Path of the package to upload
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,
}

pub fn run(args: &UploadPackageArgs, client: &Client) -> Result<()> {
    let UploadPackageArgs { file } = args;
    let mut package = Package::new(file.into())?;

    let dataset = package.read_dataset();

    todo!()
}
