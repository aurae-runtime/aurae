# Unreleased

## Added

- Implemented `Eq` for `common::headers::Encoding`, `common::headers::MediaType`, 
  `common::headers::Headers`, `common::HttpHeaderError`, `common::Body`, `common::Version`,
  `common::RequestError`, `request::Uri`, `request::RequestLine`, `response::StatusCode`,
  `response::ResponseHeaders`

## Changed

- Mark `HttpServer::new_from_fd` as `unsafe` as the correctness of the unsafe code
  in this method relies on an invariant the caller has to uphold
- Always set 'Content-Length' in non-100/204 responses regardless of whether the
  body is empty. Otherwise, the client waits for the server to close the
  connection or for timeout to occur.

# v0.1.0

- micro-http v0.1.0 first release.
