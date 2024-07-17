
# Magic CLI
![Magic CLI logo](/assets/logo_sm.png)


![GitHub Actions CI](https://github.com/guywaldman/magic-cli/actions/workflows/ci.yml/badge.svg)

Magic CLI is a command line utility which uses LLMs to help you use the command line more efficiently, inspired by projects such as [Amazon Q (prev. Fig terminal)](https://fig.io/) and [GitHub Copilot for CLI](https://docs.github.com/en/copilot/using-github-copilot/using-github-copilot-in-the-command-line).

> **Read the [announcement blog post](https://guywaldman.com/posts/introducing-magic-cli).**

> [!CAUTION]
>
> This project is still in early development.  
> Expect breaking changes and bugs, and please report any issues you encounter.  
> Thank you!

---

## Installation

> [!NOTE]
>
> For more options on how to install Magic CLI, see the [releases page](https://github.com/guywaldman/magic-cli/releases) for the version you wish to install.

### Shell

```shell
curl -LsSf https://github.com/guywaldman/magic-cli/releases/download/0.0.2/magic-cli-installer.sh | sh
```

### Homebrew

```shell
brew install guywaldman/tap/magic-cli
```

### PowerShell

```powershell
powershell -c "irm https://github.com/guywaldman/magic-cli/releases/download/0.0.2/magic-cli-installer.ps1 | iex"
```

### Binaries

See the [releases page](https://github.com/guywaldman/magic-cli/releases) for binaries for your platform.

## Usage Tip

Add functions to your `~/.bashrc` or `~/.zshrc` to make formatting the prompts in the terminal easier (not requiring quotes).

For example:

```shell
function mcs {
  model_prompt="$*"
  magic-cli suggest "$model_prompt"
}

function mcf {
  model_prompt="$*"
  magic-cli search "$model_prompt"
}

function mca {
  model_prompt="$*"
  magic-cli ask "$model_prompt"
}
```

## Features

- Suggest a command (see [section](#feature-suggest-a-command))
- Ask to generate a command to perform a task (see [section](#feature-ask-to-generate-a-command))
- Semantic search of commands across your shell history
- Use a local or remote LLM (see [section](#use-different-llms))

### Suggest a command

Supply a prompt to get a suggestion for the command to run.  
This is useful in scenarios where you know approximately what you want (and perhaps even the tool) but don't remember the exact arguments or their order.  
This is especially useful with CLI tools like `ffmpeg` or `kubectl`.

![Suggest subcommand](/assets/suggest_screenshot.png)

```shell
magic-cli suggest "Resize test_image.png to 300x300 with ffmpeg"
```

```
Usage: magic-cli suggest <PROMPT>

Arguments:
  <PROMPT>  The prompt to suggest a command for (e.g., "Resize image to 300x300 with ffmpeg")
```

### Search across your command history (experimental)

Search a command across your shell history, and get a list of the top results.

![Search subcommand](/assets/search_screenshot.png)

```shell
magic-cli search "zellij attach"
```

```
Usage: magic-cli search [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>  The prompt to search for
```

> [!IMPORTANT]
>
> Word to the wise: If you're using a non-local LLM, be wary about the cost of the embeddings, especially for long shell histories.

### Ask to generate a command (experimental)

Supply a prompt with a task you want the model to perform, and watch it try to suggest a command to run to achieve the goal.
It may prompt you along the way to run commands if it needs more context.

```shell
magic-cli ask "Set up the dev environment as described in the README"
```

```
Usage: magic-cli ask <PROMPT>

Arguments:
  <PROMPT>  The prompt to ask for (e.g., "Set up the development environment")
```

### Use different LLMs

Magic CLI supports two LLM providers:

- `ollama`: [Ollama](https://github.com/ollama/ollama) is a local LLM provider. The command expects Ollama to be installed and running on your local machine.
- `openai`: [OpenAI](https://openai.com/) is a cloud LLM provider. You configure an API token, and Magic CLI uses it with the OpenAI APIs.

## Configuration

Magic CLI stores configurations in `~/.config/magic_cli/config.json`.

Use `magic-cli config` (to see the options, use `magic-cli config --help`) to set the configuration options:

```
Usage: magic-cli config <COMMAND>

Commands:
  set    Set a value
  get    Get a value
  list   List the configurations
  reset  Reset the configurations to the default values
  path   Get the path to the configuration file
```

The currently suppprted configuration options are:

- `llm`: The LLM to use for generating responses. Supported values: "ollama", "openai"
- `ollama.base_url`: The base URL of the Ollama API (default: "http://localhost:11434")
- `ollama.embedding_model`: The model to use for generating embeddings (default: "nomic-embed-text:latest")
- `ollama.model`: The model to use for generating responses (default: "codestral:latest")
- `openai.api_key` (secret): The API key for the OpenAI API
- `openai.embedding_model`: The model to use for generating embeddings (default: "text-embedding-ada-002")
- `openai.model`: The model to use for generating responses (default: "gpt-4o")
- `suggest.add_to_history`: Whether to add the suggested command to the shell history (default: false)
- `suggest.mode`: The mode to use for suggesting commands. Supported values: "clipboard" (copying command to clipboard), "unsafe-execution" (executing in the current shell session) (default: "unsafe-execution")
  > Note: `unsafe-execution` is named as such to make it clear that the CLI executes the command in the current shell session. Please be extremely careful when using this mode - Magic CLI is not responsible for the execution of commands suggested.

## Roadmap

- [ ] Windows support (PowerShell is supported, but Windows has not been tested properly)
- [ ] Support for more LLM providers (e.g., [Anthropic](https://www.anthropic.com/))
- [ ] Improve local embedding index (currently stored naively as a JSON, looked into SQLLite with vector extensions)

- [ ] More test coverage

---

## Security

Security is taken seriously and all vulnerabilities will be handled with utmost care and priority.

In terms of data stored, the sensitive data that is currently handled by Magic CLI is:

- OpenAI API key, which is stored in the configuration within the **user home directory** (`!/.config/magic_cli`).
  > There are plans to store this token in the system's secure key storage, but this is not yet implemented.
- Embeddings of the shell history for the `magic-cli search` command, which are stored in the configuration within the **user home directory** (`!/.config/magic_cli`)

Please see [SECURITY.md](SECURITY.md) for more information, and instructions on how to report potential vulnerabilities.

## Contributing

Contributions are welcome!

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

My only request is that pull requests follow an issue, such that we avoid situations of your hard work not being accepted due to a lack of context or necessity.
