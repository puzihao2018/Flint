#!/usr/bin/env python3
"""Render the canonical vector F mark into app/tray assets. Requires resvg-py."""
from pathlib import Path
import json
import resvg_py
root=Path(__file__).resolve().parents[1]
mark=json.loads((root/'branding/mark.json').read_text())
def bars(color=None):
    return ''.join('<rect '+ ' '.join(f'{k}="{color if k=="fill" and color else v}"' for k,v in b.items())+'/>' for b in mark['bars'])
def svg(content,size=72,view='0 0 64 64'):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="{view}">{content}</svg>'
app=svg(f'<rect x="8" y="8" width="112" height="112" rx="26" fill="{mark["background"]}"/><g transform="translate(25 25) scale(1.21875)">{bars()}</g>',1024,'0 0 128 128')
tray=svg(bars('#000'))
(root/'branding/app-icon.svg').write_text(app+'\n')
(root/'branding/app-icon.png').write_bytes(resvg_py.svg_to_bytes(svg_string=app))
(root/'src-tauri/icons/tray-flint.svg').write_text(tray+'\n')
(root/'src-tauri/icons/tray-flint.png').write_bytes(resvg_py.svg_to_bytes(svg_string=tray))
print('Rendered the same F geometry for app, UI, and template tray icon.')
