# Security policy

Please report vulnerabilities privately through GitHub's security-advisory interface. Do not open a public issue containing access tokens, account identifiers, order details, or unredacted API responses.

OANDA access tokens must be treated as passwords. Prefer `OANDA_ACCESS_TOKEN` over `--token` so the value is not stored in shell history or exposed in process listings.
