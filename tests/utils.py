from dataclasses import dataclass
import os
import subprocess
import dotenv


def run_subcommand(subcommand: str, subcommand_args: list[str] = [], log_args: bool = False, mcli_args: list[str] = []):
    """Helper utility which runs the subcommand with the given arguments and returns the results."""

    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    magic_cli_executable_path = os.path.join(root_dir, "target", "release", "magic-cli")

    command = f"{magic_cli_executable_path} {' '.join(mcli_args)} {subcommand} {' '.join(subcommand_args)}"

    command_log = command if log_args else f"{magic_cli_executable_path} (...arguments omitted...)"

    print(f"Running command '{command_log}'...")
    proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)

    stdout, stderr, status = proc.stdout, proc.stderr, proc.returncode

    print(f"Finished running command '{command_log}' with status code {status}")

    return RunResults(
        stdout=stdout.decode("utf-8"),
        stderr=stderr.decode("utf-8"),
        status=status,
    )


def run_shell_command(command: str, log_args: bool = False, args: list[str] = []):
    """Helper utility which runs a shell command and returns the results."""

    print(f"Running shell command '{command}'...")


def set_config(key: str, value: str, log_args: bool = False, config_path: str | None = None):
    mcli_args = [] if config_path is None else ["--config", config_path]
    results = run_subcommand("config", ["set", "--key", key, "--value", value], log_args=log_args, mcli_args=mcli_args)
    assert results.status == 0


@dataclass
class RunResults:
    stdout: str
    stderr: str
    status: int


@dataclass
class Env:
    openai_api_key: str

    @classmethod
    def from_env(cls):
        ci = os.environ.get("CI")
        if ci is None or ci.lower() == "false":
            # If running locally, load the .env.local file.
            script_dir = os.path.dirname(os.path.abspath(__file__))
            env_path = os.path.join(script_dir, "..", ".env.local")
            print(f"Loading environment variables from '{env_path}'")
            dotenv.load_dotenv(dotenv_path=env_path)

        openai_api_key = os.environ.get("OPENAI_API_KEY")
        if openai_api_key is None:
            raise Exception("OPENAI_API_KEY environment variable not set")

        return cls(openai_api_key=openai_api_key)
