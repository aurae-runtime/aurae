# micro-http

This is a minimal implementation of the 
[HTTP/1.0](https://tools.ietf.org/html/rfc1945) and
[HTTP/1.1](https://www.ietf.org/rfc/rfc2616.txt) protocols. This HTTP
implementation is stateless thus it does not support chunking or compression.

The micro-http implementation is used in production by Firecracker.

As micro-http uses [`std::os::unix`](https://doc.rust-lang.org/std/os/unix/index.html) this crates only supports Unix-like targets.

## Contributing

To contribute to micro-http, checkout the
[contribution guidelines](CONTRIBUTING.md).

## Releases

New micro-http versions are released via the GitHub repository releases page. A
history of changes is recorded in our [changelog](CHANGELOG.md).

## Policy for Security Disclosures

If you suspect you have uncovered a vulnerability, contact us privately, as
outlined in our [security policy document](); we will immediately prioritize
your disclosure.