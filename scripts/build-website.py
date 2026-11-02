#!/usr/bin/env python3
"""Validate and copy the dependency-free GitHub Pages site into dist-site/."""
from hashlib import sha256
from html.parser import HTMLParser
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'website'
DESTINATION = ROOT / 'dist-site'

class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.links = []
        self.keys = []
        self.in_manifest = False
        self.manifest = ''

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            assert attrs['id'] not in self.ids, f"Duplicate ID: {attrs['id']}"
            self.ids.add(attrs['id'])
        for key in ('href', 'src'):
            if key in attrs:
                self.links.append(attrs[key])
        for key in ('data-i18n', 'data-i18n-html'):
            if key in attrs:
                self.keys.append(attrs[key])
        if tag == 'code' and attrs.get('id') == 'manifest-code':
            self.in_manifest = True

    def handle_endtag(self, tag):
        if tag == 'code':
            self.in_manifest = False

    def handle_data(self, data):
        if self.in_manifest:
            self.manifest += data

page = Page()
page.feed((SOURCE / 'index.html').read_text())
for link in page.links:
    if link.startswith('#'):
        assert link == '#' or link[1:] in page.ids, f'Missing anchor: {link}'
    elif not re.match(r'\w+://', link):
        assert (SOURCE / link).is_file(), f'Missing asset: {link}'
script = (SOURCE / 'site.js').read_text()
zh_keys = set(re.findall(r'(\w+):\s*[\x27"]', script.split('const nodes')[0]))
assert set(page.keys) <= zh_keys, f'Missing Chinese strings: {set(page.keys) - zh_keys}'
# Publish an explicit allowlist. Repository files, local data and build tools stay out.
if DESTINATION.exists():
    shutil.rmtree(DESTINATION)
DESTINATION.mkdir()
html = (SOURCE / 'index.html').read_text()
for name in ('style.css', 'site.js'):
    shutil.copy2(SOURCE / name, DESTINATION / name)
    digest = sha256((SOURCE / name).read_bytes()).hexdigest()[:12]
    html = html.replace(f'"{name}"', f'"{name}?v={digest}"')
(DESTINATION / 'index.html').write_text(html)
shutil.copytree(SOURCE / 'assets', DESTINATION / 'assets')
(DESTINATION / '.nojekyll').touch()
(DESTINATION / 'launcher-example.yaml').write_text(page.manifest + '\n')
(DESTINATION / 'robots.txt').write_text('User-agent: *\nAllow: /\nSitemap: https://zihaopu.cn/Flint/sitemap.xml\n')
(DESTINATION / 'sitemap.xml').write_text('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://zihaopu.cn/Flint/</loc></url></urlset>\n')
print(f'Website built: {len(page.ids)} unique IDs, {len(page.keys)} translated elements, local links valid.')
print(DESTINATION)
