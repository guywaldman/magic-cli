# Contributing to Magic CLI

Thank you for your interest in contributing to Magic CLI!

Please follow this process for contributing code:  
1. Open an issue to discuss the changes you want to make
1. Wait for maintainers to approve this change
   > This step is solely to reduce noise and avoid redundant work for implementing a change that isn't accepted. The assumption is that that a maintainer will review the issue in reasonable time.

## Contributioon Workflow

1. Fork the respository
1. Implement the change you wish to make
1. Open a pull request that references the relevant issue
1. Make sure that the CI passes
1. Wait for maintainers to review and approve the pull request

## Development Environment

### Prerequisites

1. Rust toolchain and Cargo (MSVC: 1.78.0)

### Workflow

```shell
git clone https://github.com/{your-username}/magic-cli
cd magic-cli

# ...Implement the changes you wish to make

cargo fmt
cargo test

# ...Commit your changes

# ...Push your changes

# ...Open a pull request
```