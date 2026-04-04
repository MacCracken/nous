# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | Yes                |

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

- `cargo audit` is run as part of every development cycle
- `cargo deny` enforces dependency license and source policies
- No network access in the resolver — all operations are local
- System command execution (`apt-cache`, `dpkg-query`) uses `std::process::Command::arg()` which does not invoke a shell, preventing command injection
