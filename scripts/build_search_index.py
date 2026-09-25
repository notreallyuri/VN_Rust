#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "docs/src/app/docs"
OUT = ROOT / "docs/public/search-index.json"


def slug(heading):
    text = re.sub(r"[^\w\s-]", "", heading.lower())
    return re.sub(r"\s+", "-", text.strip())


def clean(lines):
    out = []
    for line in lines:
        line = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", line)
        line = re.sub(r"</?[A-Za-z][^>]*>", " ", line)
        line = re.sub(r"^\s*[>*+-]\s+", "", line)
        line = re.sub(r"^\s*\d+\.\s+", "", line)
        line = re.sub(r"^\s*\|?\s*[-: ]+\|[-:| ]*$", "", line)
        line = line.replace("|", " ").replace("`", "").replace("*", "")
        out.append(line)
    return re.sub(r"\s+", " ", " ".join(out)).strip()


def route(page):
    rest = page.parent.relative_to(PAGES).as_posix()
    return "/docs" if rest == "." else f"/docs/{rest}"


def sections(page):
    title = ""
    heading = ""
    anchor = ""
    body = []
    fenced = False
    out = []

    def flush():
        text = clean(body)
        if text:
            out.append({"h": heading, "a": anchor, "t": text})
        body.clear()

    for line in page.read_text().splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
            continue
        if fenced:
            body.append(line)
            continue
        match = re.match(r"^(#{1,3})\s+(.*)", line)
        if not match:
            body.append(line)
            continue
        flush()
        text = match.group(2).strip()
        if len(match.group(1)) == 1:
            title = text
            heading, anchor = "", ""
        else:
            heading, anchor = text, slug(text)
    flush()
    return title, out


def main():
    index = []
    for page in sorted(PAGES.rglob("page.mdx")):
        title, found = sections(page)
        if not title:
            print(f"no heading in {page}", file=sys.stderr)
            return 1
        for section in found:
            index.append({"r": route(page), "p": title, **section})

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(index, separators=(",", ":")))
    size = OUT.stat().st_size
    print(f"search-index.json           {size // 1024} KiB, {len(index)} sections")
    return 0


if __name__ == "__main__":
    sys.exit(main())
