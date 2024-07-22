import tempfile
from tests.utils import run_subcommand


class TestConfig:
    def test_custom_config_get(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write('{"llm": "ollama"}')

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "ollama" in results.stdout

            with open(tmp.name, "w") as f:
                f.write('{"llm": "openai"}')

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "openai" in results.stdout

    def test_custom_config_set(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write('{"llm": "ollama"}')

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["set", "--key", "llm", "--value", "openai"], mcli_args=["--config", f.name])
                assert results.status == 0

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "openai" in results.stdout
