#!/usr/bin/env python3
"""Translate Gandora Enhancement Proposals with an LLM.

Reads credentials from the repository ``.env`` file (``GAN_API_KEY``,
``GAN_BASE_URL``, ``GAN_MODEL``) and translates English source GEPs from
``geps/`` into ``geps/local/<lang>/`` following GEP-0000's translation policy:
the translation copies the source front matter, translates ``title`` and
``description``, and adds ``language``, ``source``, ``source-revision``, and
``translation-status`` fields.

Usage:
    uv run scripts/translate-gep.py geps/0000-gep-process.md
    uv run scripts/translate-gep.py --all
    uv run scripts/translate-gep.py --all --lang zh --status Current

Only the Python standard library is required.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
GEPS_DIR = REPO_ROOT / "geps"

LANG_NAMES = {
    "zh": ("zh-CN", "Simplified Chinese (简体中文)"),
    "ja": ("ja", "Japanese (日本語)"),
    "ko": ("ko", "Korean (한국어)"),
}

SYSTEM_PROMPT = """\
You are translating a normative software-specification document (a Gandora
Enhancement Proposal) from English to {language}. Rules:

- Preserve the Markdown structure exactly: heading levels, lists, tables,
  emphasis, and blank lines.
- Do NOT translate: code blocks, inline code spans, requirement identifiers
  such as GEP-0000-R001, file paths, URLs, YAML keys, and proper names such as
  Gandora, GEP, Python, Rust, uv, Elixir.
- Keep the RFC 2119 key words MUST, MUST NOT, SHOULD, SHOULD NOT, MAY in
  uppercase English, followed by no gloss.
- Translate prose faithfully and idiomatically; this is a specification, so
  precision beats fluency.
- Output only the translated Markdown, with no surrounding commentary and no
  code fence around the whole document.
"""


def load_env(path: Path) -> dict[str, str]:
    env: dict[str, str] = {}
    if not path.exists():
        sys.exit(f"error: {path} not found; expected GAN_API_KEY etc. in it")
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        env[key.strip()] = value.strip()
    return env


def chat(env: dict[str, str], system: str, user: str) -> str:
    url = env["GAN_BASE_URL"].rstrip("/") + "/chat/completions"
    payload = {
        "model": env["GAN_MODEL"],
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
    }
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {env['GAN_API_KEY']}",
        },
    )
    with urllib.request.urlopen(request, timeout=600) as response:
        body = json.load(response)
    return body["choices"][0]["message"]["content"]


def split_front_matter(text: str) -> tuple[str, str]:
    match = re.match(r"\A---\n(.*?)\n---\n", text, re.DOTALL)
    if not match:
        sys.exit("error: source GEP has no YAML front matter")
    return match.group(1), text[match.end() :]


def split_sections(body: str) -> list[str]:
    """Split on top-level ## headings so each request stays small."""
    parts: list[str] = []
    current: list[str] = []
    in_code = False
    for line in body.splitlines(keepends=True):
        if line.startswith("```"):
            in_code = not in_code
        if not in_code and line.startswith("## ") and current:
            parts.append("".join(current))
            current = []
        current.append(line)
    if current:
        parts.append("".join(current))
    return parts


def front_matter_value(front: str, key: str) -> str:
    match = re.search(rf"^{key}:\s*(.*)$", front, re.MULTILINE)
    if not match:
        sys.exit(f"error: front matter is missing '{key}'")
    return match.group(1).strip()


def translate_field(env: dict[str, str], language: str, text: str) -> str:
    prompt = (
        f"Translate this one-line specification metadata value to {language}. "
        f"Keep proper names (Gandora, GEP, Python, uv) untranslated. "
        f"Output only the translation.\n\n{text}"
    )
    return chat(env, "You translate technical metadata precisely.", prompt).strip()


def build_translation_front_matter(
    front: str,
    env: dict[str, str],
    language_name: str,
    locale: str,
    source_name: str,
    status: str,
) -> str:
    revision = front_matter_value(front, "revision")
    title = front_matter_value(front, "title")
    description = front_matter_value(front, "description")
    zh_title = translate_field(env, language_name, title)
    zh_description = translate_field(env, language_name, description)
    lines = []
    for line in front.splitlines():
        if line.startswith("title:"):
            lines.append(f"title: {zh_title}")
        elif line.startswith("description:"):
            lines.append(f"description: {zh_description}")
        elif line.startswith("translations:") or line.startswith("  "):
            # Drop the source's translations map from the translated copy.
            if line.startswith("translations:"):
                continue
            if re.match(r"  [\w-]+: local/", line):
                continue
            lines.append(line)
        else:
            lines.append(line)
    lines.extend(
        [
            f"language: {locale}",
            f"source: ../../{source_name}",
            f"source-revision: {revision}",
            f"translation-status: {status}",
        ]
    )
    return "\n".join(lines)


NOTICE = {
    "zh-CN": (
        "> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 "
        "[{source}](../../{source})。"
    ),
}


DOC_NOTICE = {
    "zh-CN": (
        "> 本文件是非规范性翻译，仅供参考；原文为英文版 "
        "[{source}](../../{source})。"
    ),
}


def translate_doc(env: dict[str, str], path: Path, lang: str) -> Path:
    """Translate a plain manual page from docs/ into docs/local/<lang>/."""
    locale, language_name = LANG_NAMES[lang]
    text = path.read_text(encoding="utf-8")
    system = SYSTEM_PROMPT.format(language=language_name)
    sections = split_sections(text)
    translated: list[str] = []
    for index, section in enumerate(sections, start=1):
        print(f"  translating section {index}/{len(sections)} ...", flush=True)
        translated.append(chat(env, system, section).rstrip("\n") + "\n")
    out_dir = REPO_ROOT / "docs" / "local" / lang
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / path.name
    notice = DOC_NOTICE.get(locale, "").format(source=path.name)
    body = "\n".join(translated)
    if notice:
        # place the notice right under the first heading
        lines = body.split("\n", 1)
        body = lines[0] + "\n\n" + notice + "\n" + (lines[1] if len(lines) > 1 else "")
    out_path.write_text(body, encoding="utf-8")
    return out_path


def translate_gep(env: dict[str, str], path: Path, lang: str, status: str) -> Path:
    locale, language_name = LANG_NAMES[lang]
    text = path.read_text(encoding="utf-8")
    front, body = split_front_matter(text)
    system = SYSTEM_PROMPT.format(language=language_name)
    sections = split_sections(body)
    translated: list[str] = []
    for index, section in enumerate(sections, start=1):
        print(f"  translating section {index}/{len(sections)} ...", flush=True)
        translated.append(chat(env, system, section).rstrip("\n") + "\n")
    new_front = build_translation_front_matter(
        front, env, language_name, locale, path.name, status
    )
    notice = NOTICE.get(locale, "").format(source=path.name)
    out_dir = GEPS_DIR / "local" / lang
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / path.name
    parts = [f"---\n{new_front}\n---\n"]
    if notice:
        parts.append(f"\n{notice}\n")
    parts.append("\n" + "\n".join(translated))
    out_path.write_text("".join(parts), encoding="utf-8")
    return out_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="*", help="source GEP files to translate")
    parser.add_argument("--all", action="store_true", help="translate every source GEP")
    parser.add_argument(
        "--doc",
        action="store_true",
        help="treat inputs as plain docs/ manual pages (no GEP front matter)",
    )
    parser.add_argument("--lang", default="zh", choices=sorted(LANG_NAMES))
    parser.add_argument(
        "--status",
        default="Current",
        choices=["Current", "Stale"],
        help="translation-status recorded in the output front matter",
    )
    args = parser.parse_args()

    env = load_env(REPO_ROOT / ".env")
    for key in ("GAN_API_KEY", "GAN_BASE_URL", "GAN_MODEL"):
        if key not in env:
            sys.exit(f"error: .env is missing {key}")

    if args.all:
        files = sorted(GEPS_DIR.glob("[0-9][0-9][0-9][0-9]-*.md"))
    else:
        files = [Path(f) for f in args.files]
    if not files:
        parser.error("pass GEP files or --all")

    for path in files:
        print(f"translating {path.name} -> local/{args.lang}/", flush=True)
        if args.doc:
            out = translate_doc(env, path.resolve(), args.lang)
        else:
            out = translate_gep(env, path.resolve(), args.lang, args.status)
        print(f"  wrote {out.relative_to(REPO_ROOT)}", flush=True)


if __name__ == "__main__":
    main()
