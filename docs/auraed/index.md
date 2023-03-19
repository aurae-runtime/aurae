# Aurae Daemon

The Aurae Daemon (auraed) is the main daemon that powers Aurae.

The Aurae Daemon runs as a gRPC server which listens over a Unix domain socket by default.

```
/var/run/aurae/aurae.sock
```

## Running auraed

Running as `/sbin/init` is currently under active development.

To run auraed as a standard library server you can run the daemon alongside your current init system.

## Building from source

We suggest using the [aurae](https://github.com/aurae-runtime/aurae) repository for building all parts of the project.

If you intend on building this repository directly you can leverage the Makefile in this repository.

```bash
make auraed
```

or using Cargo directly

```bash
cargo clippy
cargo install --debug --path .
```


## Running auraed in a Container

It is possible to run auraed in a container as long as the following is considered:

 - Populating mTLS certificate material into the container.
 - Exposing either the socket or a network interface from the container for client connections.

Building the container (replace with your values).

```
sudo -E docker build -t krisnova/aurae:latest -t krisnova/aurae:$sha -f images/Dockerfile.nested .
sudo -E docker push krisnova/aurae:latest
sudo -E docker push krisnova/aurae:$sha
```

Running the container as PID 1:

**Note**: This pattern (and the `krisnova` registry) will never be officially supported by the project. This is temporary as with bootstrap the project.

```
make pki config
sudo -E docker run -v /etc/aurae:/etc/aurae krisnova/aurae:latest
```
