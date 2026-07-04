from __future__ import annotations

import os
import stat
import tempfile
import unittest
from pathlib import Path

from contextclean import clean_text, output_text


def _fake_ctxclean(tmp_path: Path) -> Path:
    script = tmp_path / ("ctxclean.cmd" if os.name == "nt" else "ctxclean")
    if os.name == "nt":
        script.write_text(
            '@echo off\r\necho {"output":{"content":"wrapped json"}}\r\n',
            encoding="utf-8",
        )
    else:
        script.write_text(
            '#!/bin/sh\necho \'{"output":{"content":"wrapped json"}}\'\n',
            encoding="utf-8",
        )
        script.chmod(script.stat().st_mode | stat.S_IEXEC)
    return script


class WrapperTests(unittest.TestCase):
    def test_clean_text_parses_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = clean_text(
                "<main><h1>Keep</h1></main>",
                output_format="json",
                ctxclean_bin=str(_fake_ctxclean(Path(temp_dir))),
            )

        self.assertEqual(output_text(result), "wrapped json")


if __name__ == "__main__":
    unittest.main()
