# TLS test fixtures

These are public, test-only certificates and a matching server private key.
They must never be used outside local tests. `ca.pem` signs `localhost.der`,
which is valid for the DNS name `localhost` (not IP addresses).
`unrelated.pem` is a different trust anchor used to verify rejection.

The certificates are valid from January 2020 to January 2120. The server key
is intentionally committed so tests do not need OpenSSL or external networking.
