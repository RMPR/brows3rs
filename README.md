# Brows3rs

Implementation in 🦀 of a simple web interface to browse a minio bucket.

## Build

Download cargo and rust toolchain - https://doc.rust-lang.org/book/ch01-01-installation.html.
From repository root:

```sh
cargo build
```

## Usage

Check the top level `Cargo.toml` file for available binary crates (`[[bin]]` sections). Run the built binary with `--help`.

To download all artifacts in given URL, from repo root:

```sh
./target/debug/downloader <URL>
```

To list all artifacts:

```sh
./target/debug/downloader <URL> list
```

To list in a UNIX style `tree` view:

```sh
./target/debug/downloader <URL> list tree
```

## Tasks

09 August, 2024

- Start the work on web interface
- Add some unit tests
