# puls

`puls` is a CLI tool that allows you to run a local Pulsar instance with different configurations.

The motivation is to minimize the entry threshold for Pulsar novices and quickly try different Puslar features without spending much time.

## Install

### MacOS using Homebrew

```
brew tap tealtools/tap
brew install tealtools/tap/puls
```

### Linux

TODO

### Build from source

- Clone the repository `git clone git@github.com:tealtools/puls.git && cd ./puls`
- [Install Rust](https://www.rust-lang.org/tools/install) or alternatively [install Nix](https://nixos.org/download/) and run `make dev` at the repository root to enter dev shell with all pre-installed tools.
- Rust `cargo install --path .`
- Check the installation `puls --version`

## Usage

```
puls start
```

## Requirements

- Installed [Docker](https://docs.docker.com/engine/install/) >= 2.24.0
- Enough computing resources. For Docker Desktop, you can adjust available resources by following these [instructions](https://docs.docker.com/desktop/settings/mac/#resources).

You can take the following numbers as a basis:
- A cluster with 1 broker and 1 bookie needs 1 CPU core and 1GB RAM.
- A cluster with 3 brokers and 3 bookies needs 1.5-2 CPU cores and 3GB RAM.

For example, you'll need about 3-4CPU cores and 6GB RAM for the following Pulsar instance with two clusters: 

`puls create --num-clusters 2 --num-bookies 3 --num-brokers 3 multi-cluster`

## Planned features

- [Minio S3](https://github.com/minio/minio) to demonstrate the [Pulsar tiered storage](https://pulsar.apache.org/docs/next/concepts-tiered-storage/) feature
- Postgres to demonstrate [Pulsar IO](https://pulsar.apache.org/docs/next/io-overview/)
- [Dekaf UI](https://www.dekaf.io/) as Pulsar management UI
- Monitoring stack - Prometheus, Grafana
