# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2021-08-20

### Added

- `Request` now omits `params` from serialization when empty for version 2.0 jsonrpm calls as defined [here](https://www.jsonrpc.org/specification#request_object).

## [0.7.0] - 2021-07-27

### Added

- `JsonRpcError` now includes the optional `data` field, defined [here](https://www.jsonrpc.org/specification#error_object).

## [0.6.0] - 2021-04-26

### Changed

- APIs defined with `version = 2.0` (as in `#[jsonrpc_client::api(version = "2.0")]`) will now serialize their arguments by name instead of by position.
  The JSON-RPC spec is vague on whether every `2.0` server must accept parameters by name.
  For now we assume that this is the case.
  This restriction may be lifted in the future.

## [0.5.1] - 2021-02-22

### Changed

- Deactivate `default-features` of the `reqwest` dependency.

## [0.5.0] - 2021-01-11

### Changed

- Bump version of `reqwest` to 0.11 and thereby change the transitive dependency of `tokio` to 1.0.

## [0.4.0] - 2021-01-11

### Added

- Re-export `async_trait` and `serde` dependencies from `jsonrpc_client` (<https://github.com/thomaseizinger/rust-jsonrpc-client/issues/6>).
  This allows usage of the macros without having to add these dependencies to your own `Cargo.toml`.

## [0.3.0] - 2021-01-11

This version is a complete re-write of the original `jsonrpc_client` crate.
It features a proc-macro based approach for declaring JSON-RPC APIs which you can then interact with using a number of different backends.

[unreleased]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/0.7.0...HEAD
[0.7.0]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/v0.6.0...0.7.0
[0.6.0]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/thomaseizinger/rust-jsonrpc-client/compare/32da264b1fdccf4302dc889ca8b2a407fe5b294f...v0.3.0
