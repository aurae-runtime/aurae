# Aurae Daemon

The Aurae Daemon (auraed) is the main daemon that powers Aurae. 

The Aurae Daemon runs as a gRPC server which listens over a Unix domain socket by default.

``` 
/var/run/aurae/aurae.sock
```

## Running Auraed 

Running as `/init` is currently under active development.

To run auraed as a standard library server you can run the daemon alongside your current init system.

```bash 
sudo -E auraed
```

Additional flags are listed below.

```
USAGE:
    auraed [OPTIONS]

OPTIONS:
        --ca-crt <CA_CRT>            [default: /etc/aurae/pki/ca.crt]
    -h, --help                       Print help information
    -s, --socket <SOCKET>            [default: /var/run/aurae/aurae.sock]
        --server-crt <SERVER_CRT>    [default: /etc/aurae/pki/_signed.server.crt]
        --server-key <SERVER_KEY>    [default: /etc/aurae/pki/server.key]
    -v, --verbose                    
    -V, --version                    Print version information

```

## Building from source

We suggest using the [aurae](https://github.com/aurae-runtime/aurae) repository for building all parts of the project.

If you intend on building this repository directly you can leverage the Makefile in this repository.

```bash
make
```

or using Cargo directly

```bash
cargo clippy
cargo install --debug --path .
```


## Running Auraed in a Container 

It is possible to run Auraed in a container as long as the following is considered:

 - Populating mTLS certificate material into the container.
 - Exposing either the socket or a network interface from the container for client connections.

Building the container (replace with your values).

```
sudo -E docker build -t krisnova/aurae:latest -t krisnova/aurae:$sha -f images/Dockerfile.nested .
sudo -E docker push krisnova/aurae:latest
sudo -E docker push krisnova/aurae:$sha
```

Running the container as pid 1

```
sudo -E docker run -v /etc/aurae:/etc/aurae -v /var/run/aurae:/var/run/aurae -it krisnova/aurae:latest /sbin/init
```