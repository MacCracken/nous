# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.7.x   | Yes                |
| < 1.7   | No                 |

## Reporting a Vulnerability

If you discover a security vulnerability in Nous, please report it responsibly.

**Do not open a public issue.**

Instead, email the maintainer directly or use GitHub's private vulnerability reporting feature on the [repository](https://github.com/MacCracken/nous).

### What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response timeline

- Acknowledgment within 48 hours
- Assessment and fix within 7 days for critical issues
- Public disclosure after a fix is available

## Security Practices

- `cyrius vet` (include-graph audit) and `cyrius deny` (dependency policy) run in CI on every change
- No network access in the resolver — all operations are local
- System commands (`apt-cache`, `dpkg-query`, `apt-get -s`) run as array-form exec with an absolute tool path and an empty environment, never through a shell; only names in Debian Policy syntax ever reach an argv, so an option-shaped name such as `-c/tmp/x` is never passed
- Untrusted input (manifests, recipes, apt output, lockfiles) goes through parsers with fuzz targets, run under `cyrius fuzz --poison=ab` in CI
- Lockfiles pin each package's source and, where the source publishes one, its artifact checksum; locked resolution refuses any drift (ADR 0008, ADR 0009)
