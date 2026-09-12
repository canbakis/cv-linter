# Security policy

## Supported versions

Until CV Linter reaches 1.0, only the latest tagged release receives security
fixes.

## Reporting a vulnerability

Please report vulnerabilities through
[GitHub private vulnerability reporting](https://github.com/canbakis/cv-linter/security/advisories/new).
Its signed-out availability is part of the release checklist.
Do not open a public issue for an unpatched vulnerability or include a real CV,
personal data, secrets, or sensitive document content in a report.

Include the CV Linter version, operating system, input format, the smallest safe
reproduction you can provide, and the observed impact. Use synthetic documents
whenever possible.

CV Linter reads explicitly selected local documents with the permissions of the
user who launches it. Its local stdio transport does not make the configured AI
host or model local; extracted content handed to a host follows that host's data
policies.
