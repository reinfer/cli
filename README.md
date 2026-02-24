<p align="center">
    <a href="https://docs.uipath.com/communications-mining/automation-cloud/latest/developer-guide/overview-cli">
    <img alt="reinfer-cli" src="https://avatars.githubusercontent.com/u/375663?s=200&v=4" width="128">
  </a>
</p>

<p align="center">
  Command line interface for UiPath IXP
</p>

<p align="center">
  <a href="https://github.com/reinfer/cli/actions?query=workflow%3ABuild">
    <img alt="Build Status" src="https://github.com/reinfer/cli/workflows/Build/badge.svg">
  </a>

  <a href="https://crates.io/crates/reinfer-cli">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/reinfer-cli.svg">
  </a>
</p>

<p align="center">
  <a href="https://docs.uipath.com/communications-mining/automation-cloud/latest/developer-guide/overview-cli">
    API Documentation
  </a> | <a href="https://docs.uipath.com/communications-mining/automation-cloud/latest/developer-guide/overview-cli">
    Website
  </a>
</p>

`re` is the official command line interface for [UiPath IXP](https://docs.uipath.com/). It simplifies managing resources, such as sources and datasets, as well as importing or exporting communications data. Additionally, `re` maintains multiple contexts, making it easy to switch between multiple authentication tokens for different users (and endpoints for multiple clusters if needed).

#### API Library

The [api](/api) directory contains a Rust client library for IXP which can be used for API access independently. Please refer to that directory for more information. The rest of the README is about the command line tool for managing IXP resources.

### Features

- _Create, get, update and delete_ operations for sources, datasets, comments and more.
- Context management for multiple endpoints (IXP clusters) and user tokens.
- Upload new verbatims to a source.
- Easily download raw verbatims from a set of sources and datasets together
  with human applied annotations. Useful for backups, migrating data or for
  applying some transformations to the data.
- Basic shell autocompletion for `zsh` and `bash`.
- Colorized terminal output and progress bars.

### Demo

![](/readme-demo.gif)

## Installation

### Binary

Statically linked binaries with no dependencies are provided for selected platforms:

> [!NOTE]  
> Most users should install using these binaries

- [Linux (x86_64-unknown-linux-musl)](https://reinfer.dev/public/cli/bin/x86_64-unknown-linux-musl/0.38.17/re)
- [macOS (aarch64-apple-darwin)](https://reinfer.dev/public/cli/bin/aarch64-apple-darwin/0.38.17/re)
- [Windows (x86_64-pc-windows-gnu)](https://reinfer.dev/public/cli/bin/x86_64-pc-windows-gnu/0.38.17/re.exe)


<details>

<summary>How do I install or update a statically linked binary?</summary>

The binaries linked above include all necessary dependencies and are directly
runnable. To update or install a binary, follow the instructions below -
replacing the existing binaries when updating.

#### Mac and Linux
1. Download the relevant binary
2. Make it executable with `chmod +x re`
3. Move it to `/usr/local/bin`
4. You should now be able to run the `re` command in your terminal

💡 On mac, you may need to allow the binary to run in the `Privacy & Security` section of system settings on first use.

#### Windows
1. Download the relevant binary
2. Move it to a folder in your PATH environment variable
3. You should now be able to run the `re` command in your terminal


💡 If you don't know what folders are in your PATH environment variable; search
for "Edit environment variables" in the windows menu, select the `Path`
variable in the top "User Variables" section and then click edit. You may need
to add a folder to this list and move the binary to that folder.

</details>



### Debian / Ubuntu

You can download a `.deb` package [here](https://reinfer.dev/public/cli/debian/reinfer-cli_0.38.17_amd64.deb).

### From Source

> [!IMPORTANT]  
> It is not recommended that you build from source unless you are actively contributing to this code base. You should use the static binaries so that you don't need to worry about build dependencies.

To build from source, you need a recent version of the [Rust toolchain](https://rustup.rs/) installed.

#### Using `cargo install`

To install using `cargo install` run the following. Note; you'll need to ensure that you have all relevant build dependencies installed.

```
cargo install reinfer-cli
```

Ensure you have the cargo bin directory in your path (typically `~/.cargo/bin`).

#### Manual

Make sure you have the following build dependencies installed: 

##### Debian/Ubuntu

```
sudo apt install autoconf automake autopoint libtool pkg-config
```

##### macOS

```
sudo port install autoconf automake gettext libtool pkgconfig
```

Build it the usual way using cargo

```
cargo build --release
```

The binary is located at `../target/release/re`. Move it somewhere suitable, e.g.

```
sudo mv ../target/release/re /usr/local/bin/
```

## Getting Started

Check the installation and see a full listing of the available commands by running `re` with no arguments.

### Authentication

#### Per Session

The simplest way to authenticate is to specify the API token for every command. By default `re` will prompt you to enter it interactively. E.g. to list the available datasets

```
➜ re get datasets
input: Enter API token [none]: MYSUPERSECRETAPITOKEN
 Name                                ID                Updated (UTC)        Title
 InvestmentBank/collateral-triggers  aa9dda7c059e5a8d  2019-04-30 17:25:03  IB Collateral Triggers
 InvestmentBank/george-test          1aaeacd49dfce8a0  2019-05-10 15:32:34  Test Dataset
 InvestmentBank/margin-call          b9d50fb2b38c3af5  2019-05-08 07:51:09  IB Margin Call
 InvestmentBank/margin-call-large    6d00b9f69ab059f6  2019-05-11 09:23:43  IB Margin Call Large
```

The token can also be specified using `--token`

```
➜ re --token MYSUPERSECRETAPITOKEN get datasets
```

This is not generally a good idea (e.g. it'll be stored in your shell history). Better to store in a environment variable.

```
➜ re --token $IXP_TOKEN get datasets
```

Even better to use contexts, see further below.

#### Contexts (stateful authentication)

Contexts help avoid having to manually specify the token and endpoint with every command. A _context_ is composed of

- The authentication token (which user?)
- The IXP cluster endpoint to talk to, typically `cloud.uipath.com/<tenant>/<org>/reinfer_`
- (Optional) An HTTP proxy to use for all requests
- A memorable name which serves as an identifier for the "context"

Commands for managing _contexts_ are under `re config` and allow one to create, update, set and delete contexts. Run `re config -h` to see all the options.

When creating the very first context, this will be set as the active one

```
➜ re config add --name production --endpoint cloud.uipath.com/<tenant>/<org>/reinfer_
I A new context `production` will be created.
* Enter API token [none]: MYSUPERSECRETTOKEN
W Be careful, API tokens are stored in cleartext in /home/marius/.config/reinfer/contexts.json.
I New context `production` was created.
```

The current context will be used for all subsequent commands.

```
➜ re get datasets
 Name                                ID                Updated (UTC)        Title
 InvestmentBank/collateral-triggers  aa9dda7c059e5a8d  2019-04-30 17:25:03  IB Collateral Triggers
 InvestmentBank/george-test          1aaeacd49dfce8a0  2019-05-10 15:32:34  Test Dataset
 InvestmentBank/margin-call          b9d50fb2b38c3af5  2019-05-08 07:51:09  IB Margin Call
 InvestmentBank/margin-call-large    6d00b9f69ab059f6  2019-05-11 09:23:43  IB Margin Call Large
```

Any of the context settings can be overwritten as a one off using global flags such as `--token`, `--endpoint` and `--proxy`.

```
➜ re --proxy http://proxy.example get datasets
```

Adding a context with a name that already exists will allow you to update any of the saved settings.

### Uploading Comments

WIP

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the `reinfer-cli` or `reinfer-client` crate, as defined in the
Apache-2.0 license, shall be licensed as above, without any additional terms or
conditions.

### Release Preparation

- Update repo files to represent the new version
  - Bump the version number in both `api/Cargo.toml` and `cli/Cargo.toml`.
    - You will also need to update the version spec in the `cli/Cargo.toml` that points to `api/Cargo.toml`.
  - Run `cargo build` to update all lockfiles.
  - Update the `CHANGELOG.md` to mark all `Unreleased` changes as part of the new version.
    - Check the git log since last release to make sure it's not missing anything.
  - Update the download links to the static binaries in `README.md`
  - Commit everything and PR it as usual.
- Cut a release by [creating a new Github release](https://github.com/reinfer/cli/releases/new).
  - Use the version number as the release title: `v0.10.1`.
  - Use the relevant changelog section as the release description.
  - `Publish release` will upload build artefacts and tag the relevant commit.
  - Check the [Github actions log](https://github.com/reinfer/cli/actions/workflows/publish.yml) to make sure the release was successful.

## License

This project is licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).
