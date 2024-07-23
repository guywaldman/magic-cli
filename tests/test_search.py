import json
import tempfile

from tests.utils import Env, run_subcommand, set_config


class TestSearchSubcommand:
    """Tests for the `search` subcommand, which allows users to semantically search for commands across their shell history."""

    def test_basic_search_openai(self):
        env = Env.from_env()

        with tempfile.TemporaryDirectory() as index_dir, tempfile.NamedTemporaryFile(
            mode="w", delete=False
        ) as config, tempfile.NamedTemporaryFile(mode="w", delete=False) as shell_history:
            with open(config.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "openai"}, "search": {"shell_history": shell_history.name, "index_dir": index_dir}}))

            with open(shell_history.name, "w") as f:
                f.write("echo foobar\n")
                f.write("echo lorem ipsum\n")

            set_config("openai.model", "gpt-4o-mini", config_path=config.name)
            set_config("openai.embedding_model", "text-embedding-ada-002", config_path=config.name)
            set_config("openai.api_key", env.openai_api_key, config_path=config.name)

            results = run_subcommand("search", ["foobar", "--output-only"], mcli_args=["--config", config.name])
            assert results.status == 0
            assert "echo foobar" in results.stdout

    def test_search_with_remote_llm(self):
        env = Env.from_env()

        with tempfile.TemporaryDirectory() as index_dir, tempfile.NamedTemporaryFile(
            mode="w", delete=False
        ) as config, tempfile.NamedTemporaryFile(mode="w", delete=False) as shell_history:
            with open(config.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "openai"}, "search": {"shell_history": shell_history.name, "index_dir": index_dir}}))

            with open(shell_history.name, "w") as f:
                f.write("echo foobar\n")
                f.write("echo lorem ipsum\n")

            set_config("openai.model", "gpt-4o-mini", config_path=config.name)
            set_config("openai.embedding_model", "text-embedding-ada-002", config_path=config.name)
            set_config("openai.api_key", env.openai_api_key, config_path=config.name)

            results = run_subcommand("search", ["foobar", "--output-only"], mcli_args=["--config", config.name])
            print(results)
