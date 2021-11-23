<p align="center">
    <a href="https://reinfer.io">
    <img alt="reinfer-cli" src="https://user-images.githubusercontent.com/797170/86259580-19d33180-bbb4-11ea-9909-3c31251345f1.png" width="128">
  </a>
</p>

<p align="center">
  Command line interface for Re:infer
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
  <a href="https://reinfer.io/docs">
    API Documentation
  </a> | <a href="https://reinfer.io">
    Website
  </a>
</p>

`re` is the official command line interface for [Re:infer](https://reinfer.io). It simplifies managing resources, such as sources and datasets, as well as importing or exporting communications data. Additionally, `re` maintains multiple contexts, making it easy to switch between multiple authentication tokens for different users (and endpoints for multiple clusters if needed).

#### API Library

The [api](/api) directory contains a Rust client library for reinfer which can be used for API access independently. Please refer to that directory for more information. The rest of the README is about the command line tool for managing reinfer resources.

### Features

- _Create, get, update and delete_ operations for sources, datasets, comments and more.
- Context management for multiple endpoints (reinfer clusters) and user tokens.
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

- [Linux (x86_64-unknown-linux-musl)](https://reinfer.io/public/cli/bin/x86_64-unknown-linux-musl/0.10.0/re)
- [macOS (x86_64-apple-darwin)](https://reinfer.io/public/cli/bin/x86_64-apple-darwin/0.10.0/re)
- [Windows (x86_64-pc-windows-gnu)](https://reinfer.io/public/cli/bin/x86_64-pc-windows-gnu/0.10.0/re.exe)

### Debian / Ubuntu

You can download a `.deb` package [here](https://reinfer.io/public/cli/debian/reinfer-cli_0.10.0_amd64.deb).

### From Source

To build from source, you need a recent version of the [Rust toolchain](https://rustup.rs/) installed.

#### Using `cargo install`

To install using `cargo install` run the following.

```
cargo install reinfer-cli
```

Ensure you have the cargo bin directory in your path (typically `~/.cargo/bin`).

#### Manual

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
➜ re --token $REINFER_TOKEN get datasets
```

Even better to use contexts, see further below.

#### Different Clusters

By default, the endpoint for all commands is `https://reinfer.io`. This can be overidden using `--endpoint`, e.g.

```
re --endpoint http://localhost:8000 --token $REINFER_TOKEN get datasets
```

#### Contexts (stateful authentication)

Contexts help avoid having to manually specify the token and endpoint with every command. A _context_ is composed of

- The authentication token (which user?)
- The Re:infer cluster endpoint to talk to, typically `https://reinfer.io`
- (Optional) An HTTP proxy to use for all requests
- A memorable name which serves as an identifier for the "context"

Commands for managing _contexts_ are under `re config` and allow one to create, update, set and delete contexts. Run `re config -h` to see all the options.

When creating the very first context, this will be set as the active one

```
➜ re config add --name production --endpoint https://reinfer.io/
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

# License

This project is licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the `reinfer-cli` or `reinfer-client` crate, as defined in the
Apache-2.0 license, shall be licensed as above, without any additional terms or
conditions.

#### Release Preparation

- Update repo files to represent the new version
  - Bump the version number in both `api/Cargo.toml` and `cli/Cargo.toml`.
    - You will also need to update the version spec in the `cli/Cargo.toml` that points to `api/Cargo.toml`.
  - Run `cargo build` to update all lockfiles.
  - Update the `CHANGELOG.md` to mark all `Unreleased` changes as part of the new version.
    - Check the git log since last release to make sure it's not missing anything.
  - Commit everything and PR it as usual.
- Cut a release by [creating a new Github release](https://github.com/reinfer/cli/releases/new).
  - Use the version number as the release title: `v0.10.0`.
  - Use the relevant changelog section as the release description.
  - `Publish release` will upload build artefacts and tag the relevant commit.
  - Check the [Github actions log](https://github.com/reinfer/cli/actions/workflows/publish.yml) to make sure the release was successful.
