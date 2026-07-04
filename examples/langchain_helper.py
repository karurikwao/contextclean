"""Minimal LangChain-style helper example for ContextClean.

This example uses the thin Python wrapper scaffold in `packages/python`.
Install that package locally, or add `packages/python` to PYTHONPATH while
developing the integration.
"""

from __future__ import annotations

from pathlib import Path

from contextclean import clean_file, output_text


def clean_for_langchain(path: str | Path, max_tokens: int = 8000) -> str:
    result = clean_file(
        path,
        max_tokens=max_tokens,
        output_format="markdown",
    )
    return output_text(result)


if __name__ == "__main__":
    print(clean_for_langchain("fixtures/dirty_html_article.html", max_tokens=4000))
