import tempfile
from tests.utils import run_subcommand, set_config


class TestSuggestSubcommand:
    # NOTE: Skipping this test for now since SLMs provide a lot of flaky tests.
    # TODO: Test with larger foundation models.
    # def test_basic_ollama(self):
    #     with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
    #         with open(tmp.name, "w") as f:
    #             f.write("{}")

    #         set_config("llm", "ollama", config_path=tmp.name)
    #         set_config("ollama.model", "llama3", config_path=tmp.name)

    #         with open(tmp.name, "r") as f:
    #             print(f.read())

    #         results = run_subcommand(
    #             "suggest", ["'Print the current directory using `ls`. Use only `ls`'", "--output-only"], mcli_args=["--config", tmp.name]
    #         )
    #         assert results.status == 0, "Suggest subcommand failed with status code {}".format(results.status)
    #         assert "ls" in results.stdout, "Suggest subcommand did not return the expected command"

    def test_fail_if_llm_misconfigured(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tmp:
            with open(tmp.name, "w") as f:
                f.write("{}")

            set_config("llm", "openai", config_path=tmp.name)
            results = run_subcommand(
                "suggest", ["'Print the current directory using `ls`. Use only `ls`'"], mcli_args=["--config", tmp.name]
            )
            assert results.status == 1, "Suggest subcommand did not fail with status code 1"
