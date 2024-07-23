# Version 0.0.5

- Adds a new `general.access_to_shell_history` configuration flag for explicitly allowing access to shell history ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Adds a new `search.allow_remote_llm` configuration flag for explicitly allowing access to shell history to non-local LLM providers ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Adds a new `search.shell_history` configuration for specifying a custom path to a shell history file (added for testing, but may be optionally useful for users none the less) ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Adds a new `search.index_dir` configuration for specifying a custom path to a directory which will contain the indexing data and metadata (added for testing, but may be optionally useful for users none the less) ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Adds and improves on E2E test coverage ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Fixes an issue where embeddings wouldn't work for OpenAI since embedding dimensions were provided and were throwing an error (see https://github.com/guywaldman/orch/pull/13) ([#33](https://github.com/guywaldman/magic-cli/pull/33))
- Various fixes & enhancements ([#33](https://github.com/guywaldman/magic-cli/pull/33))

## Version 0.0.4

- Fixed issue where the OpenAI API key was not used from the configuration ([#31](https://github.com/guywaldman/magic-cli/pull/31))

## Version 0.0.3

- A default configuration file is now created if it doesn't exist
- The default suggest mode is now `clipboard` instead of `unsafe-execution`
- The `suggest` subcommand now supports an `--output-only` flag to skip the interactive prompts (revisions and execution)
- A `--config` flag is now supported for all subcommands, where you can specify a custom configuration file
- Improved error messages (a lot more work is required)
- Under the hood:
  - Added integration with the [orch](https://github.com/guywaldman/orch) library
  - Added basic integration tests
  - Improved CI

## Version 0.0.2

- Fixed an issue where error messages were not shown when subcommands failed
- Improved the `search` subcommand
- Added documentation for the `search` subcommand

## Version 0.0.1

This is the first public release of Magic CLI.

It introduces:

- The `suggest` and `ask` subcommands
- Initial set of configuration options:
  - ollama Config
  - OpenAI Config
  - Suggestion mode (clipboard, execution)
- A Homebrew formula for easy installation on macOS
- Improved README
