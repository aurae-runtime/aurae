# Generating Client Certificate Material 

For an easy start for managing certificate material you can leverage the convenient make target.

``` 
make pki config
```

Which uses the scripts in [/hack](https://github.com/aurae-runtime/aurae/tree/main/hack) to self sign X509 certificates with mock identities. 

### Creating Clients 

After the initial PKI has been generated using the above `make pki` command, clients can easily be created using the following.

```bash 
./hack/certgen-client <name>
```

Where `<name>` is a unique string for your client you wish to provide authentication material for.