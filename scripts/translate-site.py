#!/usr/bin/env python3
"""Translate documentation-site pages with DeepSeek (GEP-0000 policy).

For every English page under ``website/docs`` (``*.md`` without a locale
suffix) that has no up-to-date ``*.zh.md`` sibling, produce the Simplified
Chinese sibling next to it — the layout mkdocs-static-i18n expects.
Generated reference pages (``website/docs/reference``) are skipped: their
Chinese comes from the ``_trans`` metadata in the std sources.

Usage:
    uv run scripts/translate-site.py            # translate missing/stale pages
    uv run scripts/translate-site.py --force    # retranslate everything
"""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DOCS = REPO_ROOT / "website" / "docs"

spec = importlib.util.spec_from_file_location(
    "tg", REPO_ROOT / "scripts" / "translate-gep.py"
)
tg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tg)  # type: ignore[union-attr]


def pages() -> list[Path]:
    out = []
    for p in sorted(DOCS.rglob("*.md")):
        if p.name.endswith(".zh.md"):
            continue
        if "reference" in p.parts:
            continue
        out.append(p)
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true")
    parser.add_argument("pages", nargs="*", help="specific pages to translate")
    args = parser.parse_args()
    env = tg.load_env(REPO_ROOT / ".env")
    system = tg.SYSTEM_PROMPT.format(language="Simplified Chinese (简体中文)")
    targets = [Path(p) for p in args.pages] if args.pages else pages()
    for page in targets:
        out = page.with_name(page.name[: -len(".md")] + ".zh.md")
        if out.exists() and not args.force:
            if out.stat().st_mtime >= page.stat().st_mtime:
                continue
        print(f"translating {page.relative_to(REPO_ROOT)} ...", flush=True)
        sections = tg.split_sections(page.read_text(encoding="utf-8"))
        translated = []
        for i, section in enumerate(sections, start=1):
            print(f"  section {i}/{len(sections)}", flush=True)
            translated.append(tg.chat(env, system, section).rstrip("\n") + "\n")
        out.write_text("\n".join(translated), encoding="utf-8")
        print(f"  -> {out.relative_to(REPO_ROOT)}", flush=True)


if __name__ == "__main__":
    sys.exit(main())
