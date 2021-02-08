use lazy_static::lazy_static;
use std::{
    env,
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub struct TestCli {
    cli_path: PathBuf,
}

impl TestCli {
    pub fn get() -> &'static Self {
        lazy_static! {
            static ref TEST_CLI: TestCli = {
                let cli_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| Some(p.parent()?.parent()?.join("re")))
                    .expect("Could not resolve CLI executable from test executable");

                TestCli { cli_path }
            };
        };

        &TEST_CLI
    }

    pub fn organisation() -> String {
        env::var("REINFER_CLI_TEST_ORG")
            .expect("REINFER_CLI_TEST_ORG must be set for integration tests")
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.cli_path);

        if let Some(context) = env::var_os("REINFER_CLI_TEST_CONTEXT") {
            command.arg("--context").arg(context);
        }

        if let Some(endpoint) = env::var_os("REINFER_CLI_TEST_ENDPOINT") {
            command.arg("--endpoint").arg(endpoint);
        }

        if let Some(token) = env::var_os("REINFER_CLI_TEST_TOKEN") {
            command.arg("--token").arg(token);
        }

        command
    }

    pub fn run(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> String {
        self.output(self.command().args(args))
    }

    pub fn run_and_error(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> String {
        self.output_error(self.command().args(args))
    }

    pub fn run_with_stdin(
        &self,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        stdin: &[u8],
    ) -> String {
        let mut process = self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        process.stdin.as_mut().unwrap().write_all(stdin).unwrap();
        let output = process.wait_with_output().unwrap();

        if !output.status.success() {
            panic!(
                "failed to run command:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8(output.stdout).unwrap()
    }

    pub fn output(&self, command: &mut Command) -> String {
        let output = command.output().unwrap();

        if !output.status.success() {
            panic!(
                "failed to run command:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8(output.stdout).unwrap()
    }

    pub fn output_error(&self, command: &mut Command) -> String {
        let output = command.output().unwrap();

        if output.status.success() {
            panic!(
                "succeeded running command (expected failure):\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }

        String::from_utf8(output.stderr).unwrap()
    }
}
