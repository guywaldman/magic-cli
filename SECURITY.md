# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.0.3   | :white_check_mark: |
| 0.0.2   | :white_check_mark: |
| 0.0.1   | :white_check_mark: |

## Data sensitivity

The sensitive data that is currently handled by Magic CLI is:

- OpenAI API key, which is stored in the configuration within the **user home directory** (`!/.config/magic_cli`).
  > There are plans to store this token in the system's secure key storage, but this is not yet implemented.
- Embeddings of the shell history for the `magic-cli search` command, which are stored in the configuration within the **user home directory** (`!/.config/magic_cli`)

## Reporting a Vulnerability

Report vulnerabilities to security [at] guywaldman.com.

They will be handled ASAP.
If the vulnerability is accepted, a mitigation will be implemented and a proper disclosure will be released.
The triage and mitigation process will depend on the potential severity and impact of the vulnerability.
