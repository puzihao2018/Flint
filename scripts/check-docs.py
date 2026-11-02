"""Check translation pairs and repository links from documentation entry points."""
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

root = Path(__file__).resolve().parents[1]
docs = root / "docs"
english = {p.name for p in (docs / "EN").glob("*.md")}
chinese = {p.name for p in (docs / "CN").glob("*.md")}
if english != chinese:
    raise SystemExit(f"Missing translation for {sorted(english ^ chinese)}")

files = [*root.glob("*.md"), *docs.glob("*/*.md"), *root.glob("examples/*/README.md")]
for page in files:
    body = re.sub(r"^```.*?^```", "", page.read_text(), flags=re.M | re.S)
    for match in re.finditer(r'\]\(([^)\s]+)\)|(?:src|href)="([^"]+)"', body):
        link = urlsplit(match[1] or match[2])
        if link.scheme or link.netloc or not link.path:
            continue
        target = (page.parent / unquote(link.path)).resolve()
        if not target.exists():
            raise SystemExit(f"{page.relative_to(root)}: broken link {link.path}")
        if page == root / "README.md" and target.is_relative_to(docs / "CN"):
            if target != docs / "CN/README.md":
                raise SystemExit(f"English README links to Chinese content: {link.path}")

for name in re.findall(r'data-doc="([^"]+)"', (root / "website/index.html").read_text()):
    if name not in english:
        raise SystemExit(f"Unknown website document: {name}")
print(f"Verified {len(english)} translation pairs and links in {len(files)} Markdown files.")
