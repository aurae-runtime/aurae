[![crates.io](https://img.shields.io/crates/v/rust-criu.svg)](https://crates.io/crates/rust-criu)
[![ci](https://github.com/checkpoint-restore/rust-criu/actions/workflows/test.yml/badge.svg)](https://github.com/checkpoint-restore/rust-criu/actions)

# rust-criu

`rust-criu` provides an interface to use [CRIU](https://criu.org/) in the
same way as [go-criu](https://github.com/checkpoint-restore/go-criu) does.

## Generate protobuf bindings

The CRIU RPC protobuf bindings are pre-generated and part of the rust-criu
repository. The bindings can be re-generated with
```shell
$ GENERATE_PROTOBUF=1 cargo build
```

## Run tests

To run the included tests please use the following command to build `rust-criu`:
```
$ GENERATE_TEST_PROCESS=1 cargo build
$ sudo target/debug/rust-criu-test /path/to/criu/binary
```
