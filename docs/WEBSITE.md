# Project website

The Flint website lives in `website/` and publishes to <https://zihaopu.cn/Flint/> through GitHub Pages. It uses plain HTML, CSS, and JavaScript with no runtime dependencies, tracking, remote fonts, or backend.

## Local preview

```sh
npm run site:build
python3 -m http.server 4173 --bind 127.0.0.1 --directory dist-site
```

Open `http://127.0.0.1:4173`. The build checks local assets, anchor targets, duplicate IDs, and Chinese translation coverage. It copies only the website files into `dist-site/`, which is ignored by Git. It also extracts the visible YAML example into `dist-site/launcher-example.yaml` for validation with `npm run manifest:check -- "$PWD/dist-site/launcher-example.yaml"`.

## Editing

- `index.html`: English copy, page structure, links, and accessible product demo markup.
- `site.js`: Chinese copy, persisted language choice, simulated service controls, menu bar preview, and clipboard interaction.
- `style.css`: responsive layouts, focus states, reduced-motion behavior, and Flint's existing green/cream/gold palette.
- `assets/`: existing Flint brand artwork; no third-party artwork.

The preview is explicitly a simulation. It does not connect to localhost services or present simulated status as a real service check. Keep supported features and planned work separate. No invented metrics, testimonials, star counts, or supported integrations.

The download button links to the DMG attached to the tagged GitHub release. When publishing a new version, update the version and asset URL in the website, both READMEs, and both user guides.

## Deployment

Enable **Settings → Pages → Source → GitHub Actions** for the repository. `.github/workflows/pages.yml` validates site changes in pull requests and deploys them on pushes to `main`. Only `dist-site/` is uploaded. Repository visibility and GitHub Pages availability must be resolved before deployment; private-repository Pages depends on the account plan. Do not change source-repository visibility solely to fix a failed deployment without the owner's authorization.

Website changes do not require bumping or rebuilding the desktop app. CI's desktop workflow remains independent. The project homepage should be set to the Pages URL once deployment succeeds.

Before shipping, check English and Chinese, a narrow mobile viewport, keyboard navigation, the demo switches/tabs, menu preview, clipboard button, and local asset requests. Supporting documents remain in the repository under `docs/`.

## Design references

[Quay](https://abhi.am/quay) informed the emphasis on concrete local-service workflows and an interactive product preview. [Pinokio](https://desktop.pinokio.co/) informed the clear product/download/developer entry points. The layout, wording, and implementation are original; Flint retains its existing identity.
