"""Typed Python helpers for the ContextClean CLI.

The wrapper intentionally shells out to the local ``ctxclean`` binary so Python
callers use the same Rust engine, token counting, redaction, and output schemas
as the command line.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any, Literal, Mapping, Sequence, overload

Mode = Literal["light", "standard", "aggressive"]
Format = Literal["text", "markdown", "json"]
Fit = Literal["gpt-4.1", "claude-sonnet", "gemini-pro"]


class ContextCleanError(RuntimeError):
    """Raised when the ctxclean process exits unsuccessfully."""

    def __init__(self, command: Sequence[str], returncode: int, stderr: str) -> None:
        self.command = tuple(command)
        self.returncode = returncode
        self.stderr = stderr
        super().__init__(
            f"ctxclean failed with exit code {returncode}: {stderr.strip() or command[0]}"
        )


def _base_args(
    *,
    ctxclean_bin: str,
    mode: Mode,
    output_format: Format,
    max_tokens: int | None,
    fit: Fit | None,
    strip_comments: bool,
    redact_secrets: bool,
    include_sensitive: bool,
) -> list[str]:
    args = [
        ctxclean_bin,
        "--mode",
        mode,
        "--format",
        output_format,
        "--quiet",
    ]
    if max_tokens is not None:
        args.extend(["--max-tokens", str(max_tokens)])
    if fit is not None:
        args.extend(["--fit", fit])
    if strip_comments:
        args.append("--strip-comments")
    if not redact_secrets:
        args.append("--no-redact-secrets")
    if include_sensitive:
        args.append("--include-sensitive")
    return args


def _run(
    args: Sequence[str],
    *,
    stdin: str | None = None,
    cwd: str | Path | None = None,
) -> str:
    command = list(args)
    command[0] = _resolve_ctxclean_bin(command[0])
    completed = subprocess.run(
        command,
        input=stdin,
        capture_output=True,
        text=True,
        cwd=None if cwd is None else str(cwd),
        check=False,
    )
    if completed.returncode != 0:
        raise ContextCleanError(command, completed.returncode, completed.stderr)
    return completed.stdout


def _resolve_ctxclean_bin(ctxclean_bin: str) -> str:
    path = Path(ctxclean_bin)
    if path.parent != Path(".") or path.is_absolute():
        return ctxclean_bin
    resolved = shutil.which(ctxclean_bin)
    if resolved:
        return resolved
    cargo_candidate = Path.home() / ".cargo" / "bin" / (
        "ctxclean.exe" if os.name == "nt" else "ctxclean"
    )
    if cargo_candidate.exists():
        return str(cargo_candidate)
    return ctxclean_bin


def _parse(output: str, output_format: Format) -> str | dict[str, Any]:
    if output_format == "json":
        parsed = json.loads(output)
        if not isinstance(parsed, dict):
            raise ValueError("ctxclean JSON output was not an object")
        return parsed
    return output


@overload
def clean_text(
    text: str,
    *,
    output_format: Literal["json"],
    ctxclean_bin: str = "ctxclean",
    mode: Mode = "standard",
    max_tokens: int | None = None,
    fit: Fit | None = None,
    strip_comments: bool = False,
    redact_secrets: bool = True,
) -> dict[str, Any]:
    ...


@overload
def clean_text(
    text: str,
    *,
    output_format: Literal["text", "markdown"] = "markdown",
    ctxclean_bin: str = "ctxclean",
    mode: Mode = "standard",
    max_tokens: int | None = None,
    fit: Fit | None = None,
    strip_comments: bool = False,
    redact_secrets: bool = True,
) -> str:
    ...


def clean_text(
    text: str,
    *,
    output_format: Format = "markdown",
    ctxclean_bin: str = "ctxclean",
    mode: Mode = "standard",
    max_tokens: int | None = None,
    fit: Fit | None = None,
    strip_comments: bool = False,
    redact_secrets: bool = True,
) -> str | dict[str, Any]:
    """Clean an in-memory string by passing it to ``ctxclean -``."""

    args = _base_args(
        ctxclean_bin=ctxclean_bin,
        mode=mode,
        output_format=output_format,
        max_tokens=max_tokens,
        fit=fit,
        strip_comments=strip_comments,
        redact_secrets=redact_secrets,
        include_sensitive=False,
    )
    args.append("-")
    return _parse(_run(args, stdin=text), output_format)


def clean_file(
    path: str | Path,
    *,
    output_format: Format = "markdown",
    ctxclean_bin: str = "ctxclean",
    mode: Mode = "standard",
    max_tokens: int | None = None,
    fit: Fit | None = None,
    strip_comments: bool = False,
    redact_secrets: bool = True,
    include_sensitive: bool = False,
    cwd: str | Path | None = None,
) -> str | dict[str, Any]:
    """Clean a file or directory path with the default ctxclean command."""

    args = _base_args(
        ctxclean_bin=ctxclean_bin,
        mode=mode,
        output_format=output_format,
        max_tokens=max_tokens,
        fit=fit,
        strip_comments=strip_comments,
        redact_secrets=redact_secrets,
        include_sensitive=include_sensitive,
    )
    args.append(str(path))
    return _parse(_run(args, cwd=cwd), output_format)


def clean_github_actions_log(
    path: str | Path,
    *,
    output_format: Format = "markdown",
    ctxclean_bin: str = "ctxclean",
    max_tokens: int | None = 8000,
    redact_secrets: bool = True,
    cwd: str | Path | None = None,
) -> str | dict[str, Any]:
    """Clean a GitHub Actions or CI failure log using the ``gha`` alias."""

    args = _base_args(
        ctxclean_bin=ctxclean_bin,
        mode="aggressive",
        output_format=output_format,
        max_tokens=max_tokens,
        fit=None,
        strip_comments=False,
        redact_secrets=redact_secrets,
        include_sensitive=False,
    )
    args.insert(1, "gha")
    args.append(str(path))
    return _parse(_run(args, cwd=cwd), output_format)


def report(
    path: str | Path,
    *,
    output_format: Format = "json",
    ctxclean_bin: str = "ctxclean",
    mode: Mode = "standard",
    max_tokens: int | None = None,
    fit: Fit | None = None,
    include_sensitive: bool = False,
    cwd: str | Path | None = None,
) -> str | dict[str, Any]:
    """Return a ContextClean report for a path."""

    args = _base_args(
        ctxclean_bin=ctxclean_bin,
        mode=mode,
        output_format=output_format,
        max_tokens=max_tokens,
        fit=fit,
        strip_comments=False,
        redact_secrets=True,
        include_sensitive=include_sensitive,
    )
    args.insert(1, "report")
    args.append(str(path))
    return _parse(_run(args, cwd=cwd), output_format)


def output_text(result: str | Mapping[str, Any]) -> str:
    """Extract cleaned content from either text/markdown output or JSON output."""

    if isinstance(result, str):
        return result
    output = result.get("output")
    if isinstance(output, Mapping):
        content = output.get("content")
        if isinstance(content, str):
            return content
    raise ValueError("result does not contain output.content")


__all__ = [
    "ContextCleanError",
    "Fit",
    "Format",
    "Mode",
    "clean_file",
    "clean_github_actions_log",
    "clean_text",
    "output_text",
    "report",
]
