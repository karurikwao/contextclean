"""Minimal LangChain-style helper example for ContextClean.

This is an example, not a packaged integration. It shells out to `ctxclean`
and returns the cleaned text so callers can build their own Document objects.
"""

from __future__ import annotations

import subprocess
from pathlib import Path


def clean_for_langchain(path: str | Path, max_tokens: int = 8000) -> str:
    completed = subprocess.run(
        [
            "ctxclean",
            str(path),
            "--max-tokens",
            str(max_tokens),
            "--format",
            "markdown",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout


if __name__ == "__main__":
    print(clean_for_langchain("fixtures/dirty_html_article.html", max_tokens=4000))
