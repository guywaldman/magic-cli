import json
import tempfile
from tests.utils import run_subcommand


class TestConfig:
    def test_config_populate_defaults(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write("{}")

            results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
            assert results.status == 0
            assert "ollama" in results.stdout

            results = run_subcommand("config", ["get", "suggest.mode"], mcli_args=["--config", f.name])
            assert results.status == 0
            assert "unsafe-execution" in results.stdout

            results = run_subcommand("config", ["get", "suggest.add_to_history"], mcli_args=["--config", f.name])
            assert results.status == 0
            assert "false" in results.stdout

    def test_config_reset(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "openai"}}))

            results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
            assert results.status == 0
            assert "openai" in results.stdout

            results = run_subcommand("config", ["reset"], mcli_args=["--config", f.name])
            assert results.status == 0

            results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
            assert results.status == 0
            assert "ollama" in results.stdout

    def test_custom_config_get(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "ollama"}}))

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "ollama" in results.stdout

            with open(tmp.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "openai"}}))

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "openai" in results.stdout

    def test_custom_config_set(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write(json.dumps({"general": {"llm": "ollama"}}))

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["set", "--key", "general.llm", "--value", "openai"], mcli_args=["--config", f.name])
                assert results.status == 0

            with open(tmp.name, "r") as f:
                results = run_subcommand("config", ["get", "general.llm"], mcli_args=["--config", f.name])
                assert results.status == 0
                assert "openai" in results.stdout
