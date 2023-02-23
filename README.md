# .tmod unpacker

A simple CLI to unpack `.tmod` files, written in pure Rust.

**This branch is modified to support a much lower MSRV of `1.56.1`. Only use this branch if necessary.**

# Installation

```sh
cargo install tmod-unpacker
```

# Usage

```sh
tmod-unpacker <input file> <output directory>
```

There is a simple help option which displays the above usage as well:

```sh
tmod-unpacker -h
# or
tmod-unpacker --help
```

You can enable logging with this crate using the `RUST_LOG` environment variable. If you are not experiencing errors, it is recommended that you stick to `RUST_LOG=info` or maybe `RUST_LOG=debug` if you are interested in the metadata of the mod. Using `RUST_LOG=trace` is extremely verbose and is intended to help diagnose errors in the reading and extraction of a `.tmod` file. Use with caution.

### Side Note

There are no tests written for this crate because I do not have the capability to create a dummy `.tmod` file at the moment. This was tested on a few `.tmod` files from the Steam Workshop, but retrieving those requires SteamCMD and being logged in, which is not feasible for tests. If you encounter any issues, please open an issue on [GitHub issues](https://github.com/campbellcole/tmod-unpacker/issues).
