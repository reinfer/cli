# reinfer/cli

`re` is the command line interface for [reinfer](https://reinfer.io). It simplifies managing reinfer resources, such as sources and datasets, as well as importing or exporting communications data. Additionally, `re` maintains multiple contexts, making it easy to switch between multiple authentication tokens for different users (and endpoints for multiple clusters if needed).

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

### Debian / Ubuntu

You can download a `.deb` package [here](https://reinfer.io/public/cli/debian/reinfer-cli_0.2.1_amd64.deb).

### Binary

You can download binaries for your platform below:

- [x86_64-unknown-linux-musl](https://reinfer.io/public/cli/bin/x86_64-unknown-linux-musl/0.2.1/re) (statically linked)
- [x86_64-unknown-linux](https://reinfer.io/public/cli/bin/x86_64-unknown-linux/0.2.1/re) (dynamically linked)

### From Source

To build from source, you need a recent version of the [Rust toolchain](https://rustup.rs/) installed.

#### Using `cargo install`

To install using `cargo install` run the following.

```
cd cli
cargo install --force --path . reinfer-cli
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

Check the installation and see a full listing of the available commands by running `re`.

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
- The reinfer endpoint to talk to, typically `https://reinfer.io` (which cluster?)
- A human rememberable name

Commands for managing _contexts_ are under `re config` and allow one to create, update, set and delete contexts. Run `re config -h` to see all the options.

When creating the very first context, this will be set as the active one

```
➜ re config add --name production --endpoint https://reinfer.io/
I A new context `production` will be created.
* Enter API token [none]: MYSUPERSECRETTOKEN
W Be careful, API tokens are stored in cleartext in /home/marius/.config/reinfer/contexts.json.
I New context `production` was created.
```

The token and endpoint for the current context will be used for all subsequent commands (these be overwritten as a one off using the `--token` and `--endpoint` arguments).

```
➜ re get datasets
 Name                                ID                Updated (UTC)        Title
 InvestmentBank/collateral-triggers  aa9dda7c059e5a8d  2019-04-30 17:25:03  IB Collateral Triggers
 InvestmentBank/george-test          1aaeacd49dfce8a0  2019-05-10 15:32:34  Test Dataset
 InvestmentBank/margin-call          b9d50fb2b38c3af5  2019-05-08 07:51:09  IB Margin Call
 InvestmentBank/margin-call-large    6d00b9f69ab059f6  2019-05-11 09:23:43  IB Margin Call Large
```

### Uploading Comments

WIP

## Roadmap and known issues

- [x] Ability to upload comments to a source
- [x] Ability to upload comments to a source and jointly upload associated
      labellings/entities to a dataset (similar to the `tools/put_comments`)
- [ ] Get CI to build `deb` package and binary automatically
- [ ] `inspect` command for resources with a detailed view
- [ ] Configurable columns for table view for `re get`
- [ ] Ability to create users
- [ ] Update operations for sources, datasets and users
- [ ] Specialise errors for common failures (such as missing sources, datasets etc.)
- [ ] Global `--no-color` argument for headless usage
- [ ] CRUD operations for labellers

### Updating binary and Debian package

Creating binaries and Debian packages should eventually be done automatically by CI. For now, there's a small script `publish-binaries` that does it.
