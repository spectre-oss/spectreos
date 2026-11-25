SpectreOS is one monorepo, but its parts do **not** share a version. Every component below is versioned and released **independently**: a fix in Files does not bump Wisp, and a libbhoo release does not touch the Keyboard. Shiraz lives in [its own repository](https://github.com/spectre-oss/shiraz) and is not versioned here.

## How versions are decided

Releases are driven by [Conventional Commits](https://www.conventionalcommits.org/). A pipeline written in pure Python (`scripts/release.py`, standard library only) looks, per component, at the commits since that component's latest tag that touch the component's own paths, and picks the highest bump:

| Commit | Bump |
| --- | --- |
| `feat:` | minor |
| `fix:`, `perf:`, `revert:` | patch |
| `type!:` or a `BREAKING CHANGE:` footer | major, capped to minor unless a maintainer explicitly allows major |
| `chore:`, `docs:`, `ci:`, `style:`, `refactor:`, `test:`, `build:` | no release |

A component that has never been tagged starts from the version recorded in its version file (`.version`, or `VERSION` for Spectre NixOS). Each release is tagged `<component>-v<X.Y.Z>`, for example `wisp-v0.2.0`, and the release commit is titled `chore(release): … [skip ci]`.

> [!NOTE]
> The *Source version* column below is read from the repository when this site is built. It can be ahead of the latest published release until the pipeline tags the next one.

## What a release publishes

- **libbhoo** is published as a crate to crates.io and gets a GitHub Release with its changelog.
- **Wisp, Keyboard, Settings and Files** are *binary-only* releases. Each GitHub Release carries just the release binary, built with `--release --locked` and the component's Cargo feature, named `<binary>-linux-x86_64`.
- **Calendar** is embedded in Wisp and has no standalone binary, so its release is notes only.
- **Spectre NixOS · Poco F1** releases carry notes only. Phone images are built locally with `make` and kept under `dist/` (see the [Poco F1 guide](site:docs/spectref1/)).
- **This website** is deployed to GitHub Pages whenever something it is built from changes. It has no tag.

## Components
