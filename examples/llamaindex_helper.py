"""Minimal LlamaIndex-style helper example for ContextClean.

This example returns cleaned text plus metrics metadata. It uses the thin
Python wrapper scaffold in `packages/python`; install that package locally, or
add `packages/python` to PYTHONPATH while developing the integration.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from contextclean import clean_file


def clean_for_llamaindex(path: str | Path, max_tokens: int = 8000) -> dict[str, Any]:
    parsed = clean_file(
        path,
        max_tokens=max_tokens,
        output_format="json",
    )
    assert isinstance(parsed, dict)
    return {
        "text": parsed["output"]["content"],
        "metadata": {
            "source": parsed["source"],
            "input_tokens": parsed["metrics"]["input_tokens"],
            "output_tokens": parsed["metrics"]["output_tokens"],
            "tokens_saved": parsed["metrics"]["tokens_saved"],
            "warnings": parsed["warnings"],
        },
    }


if __name__ == "__main__":
    print(clean_for_llamaindex("fixtures/dirty_html_article.html", max_tokens=4000))
