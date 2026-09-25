#!/usr/bin/env python3
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "docs/src/app/docs"
EXPORT = ROOT / "docs/out"
SKIP = {"node_modules", "target", ".git", "out", ".next"}


def walk(base, pattern):
    for path in base.rglob(pattern):
        if not SKIP & set(path.parts):
            yield path


def headings(text):
    found = set()
    for line in text.splitlines():
        match = re.match(r"^(#{1,6})\s+(.*)", line)
        if match:
            slug = re.sub(r"[^\w\s-]", "", match.group(2).lower())
            found.add(re.sub(r"\s+", "-", slug.strip()))
    return found


def markdown():
    problems = []
    for md in sorted(walk(ROOT, "*.md")):
        for link in re.findall(r"\]\(([^)\s]+)", md.read_text()):
            if link.startswith(("http", "mailto:", "<")):
                continue
            path, _, anchor = link.partition("#")
            target = (md.parent / path).resolve() if path else md
            where = md.relative_to(ROOT)
            if not target.exists():
                problems.append(f"{where}: {link} points at no file")
            elif anchor and anchor not in headings(target.read_text()):
                problems.append(f"{where}: {link} has no such heading")
    return problems


def routes():
    if not EXPORT.is_dir():
        return [f"{EXPORT.relative_to(ROOT)} is missing; build the site first"]
    known = {
        "/" + str(page.parent.relative_to(EXPORT)).rstrip(".")
        for page in EXPORT.rglob("index.html")
    }
    known = {route.rstrip("/") or "/" for route in known}
    problems = []
    for mdx in sorted(walk(PAGES, "page.mdx")):
        for link in re.findall(r"\]\((/docs[^)#\s]*)", mdx.read_text()):
            if link.rstrip("/") not in known:
                problems.append(f"{mdx.relative_to(ROOT)}: {link} is not an exported route")
    return problems


def braces():
    problems = []
    for mdx in sorted(walk(PAGES, "page.mdx")):
        fenced = False
        for number, line in enumerate(mdx.read_text().splitlines(), 1):
            if line.lstrip().startswith("```"):
                fenced = not fenced
            elif not fenced:
                bare = re.sub(r"`[^`]*`", "", line)
                for match in re.finditer(r"\{[^}]*\}", bare):
                    problems.append(
                        f"{mdx.relative_to(ROOT)}:{number}: {match.group(0)} "
                        "is read as JSX; wrap it in backticks"
                    )
    return problems


def main():
    checks = [("markdown links", markdown), ("exported routes", routes)]
    if "--no-export" in sys.argv:
        checks = [checks[0]]
    checks.append(("unescaped braces", braces))

    failed = False
    for name, check in checks:
        problems = check()
        if problems:
            failed = True
            print(f"{name}:")
            for problem in problems:
                print(f"  {problem}")
        else:
            print(f"{name}: ok")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
