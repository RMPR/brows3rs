# Brows3rs

Implementation in 🦀 of a simple web interface to browse a minio bucket.

## Build

Download cargo and rust toolchain - https://doc.rust-lang.org/book/ch01-01-installation.html.
From repository root:

```sh
cargo build
```

## Usage

Check the top level `Cargo.toml` file for available binary crates (`[[bin]]` sections). Run the built/downloaded binary with `--help`.
Before running the binary, make sure you have the following environment variables set:

```sh
S3_HOSTNAME

S3_ACCESSKEY

S3_SECRETKEY

S3_BUCKET
```

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

13th September, 2024

- Add a API project
- Make http server in API project
- Add a frontend project that compiles to web assembly (dioxus has file browser example)
- Make requests from frontend to http server.
- Add some unit tests

## Notes

The rust-s3 library that we use depends on openssl-sys crate, which cannot be compiled to wasm32
target. Additionally, many features in tokio cannot be compiled to wasm32 target, so even the
official amazon s3 sdk for rust does not compile to wasm32 target. Given these limitations, the
future path is to separate out the project into two parts:
- api: Compiled for x86-64 target and uses tokio, s3 libraries etc. This will make a http server.
- frontend: Compiled for wasm32 target and makes requests to the API.
