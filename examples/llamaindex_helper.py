"""Minimal LlamaIndex-style helper example for ContextClean.

This example returns cleaned text plus metrics metadata. A real package can
wrap the return value into a LlamaIndex Document.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


def clean_for_llamaindex(path: str | Path, max_tokens: int = 8000) -> dict[str, Any]:
    completed = subprocess.run(
        [
            "ctxclean",
            str(path),
            "--max-tokens",
            str(max_tokens),
            "--format",
            "json",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    parsed = json.loads(completed.stdout)
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
