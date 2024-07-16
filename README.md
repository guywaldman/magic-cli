# Magic CLI

> A command line utility to make you a magician in the terminal

![Magic CLI logo](/assets/logo_sm.png)

![GitHub Actions CI](https://github.com/guywaldman/magic-cli/actions/workflows/ci.yml/badge.svg)

---

Magic CLI is a command line utility which uses LLMs to help you use the command line more efficiently.

## Features

- Suggest a command (see [section](#feature-suggest-a-command))
- Ask to generate a command to perform a task (see [section](#feature-ask-to-generate-a-command))
- Use different LLMs (see [section](#use-different-llms))

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

Options:
  -h, --help     Print help
  -V, --version  Print version
```


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

Use `magic-cli config` (to see the options, use `magic-cli config --help`) to set the configuration options.

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
- [ ] More test coverage

---

## Security

Security is taken seriously and all vulnerabilities will be handled with utmost care and priority.

In terms of data stored, the only credential that is currently handled by Magic CLI is the OpenAI API key, which is stored in the **user home directory**.

There are plans to store this token in the system's secure key storage, but this is not yet implemented.

Please see [SECURITY.md](SECURITY.md) for more information, and instructions on how to report potential vulnerabilities.

## Contributing

Contributions are welcome!  

There are not yet official contribution guidelines, so if you have a request or found a bug, feel free to open an issue. 

My only request is that pull requests follow an issue, such that we avoid situations of your hard work not being accepted due to a lack of context or necessity.