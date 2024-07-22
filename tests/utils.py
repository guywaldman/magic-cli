from dataclasses import dataclass
import os
import subprocess


def run_subcommand(subcommand: str, subcommand_args: list[str], mcli_args: list[str] = []):
    """Helper utility which runs the subcommand with the given arguments and returns the results."""

    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    magic_cli_executable_path = os.path.join(root_dir, "target", "release", "magic-cli")

    command = f"{magic_cli_executable_path} {' '.join(mcli_args)} {subcommand} {' '.join(subcommand_args)}"

    print(f"Running '{command}'...")
    proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)

    stdout, stderr, status = proc.stdout, proc.stderr, proc.returncode

    print(f"Finished running '{command}' with status code {status}")

    return RunResults(
        stdout=stdout.decode("utf-8"),
        stderr=stderr.decode("utf-8"),
        status=status,
    )


def set_config(key: str, value: str, config_path: str | None = None):
    mcli_args = [] if config_path is None else ["--config", config_path]
    results = run_subcommand("config", ["set", "--key", key, "--value", value], mcli_args=mcli_args)
    assert results.status == 0


@dataclass
class RunResults:
    stdout: str
    stderr: str
    status: int
