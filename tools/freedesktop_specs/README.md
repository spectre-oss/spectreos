# Freedesktop Specifications Downloader

Zero-dependency Python 3 tool (`download.py` + `html2md.py`) to download specifications from https://specifications.freedesktop.org/ and convert HTML pages cleanly into Markdown under [`docs/freedesktop/`](../../docs/freedesktop/).

## Usage

```sh
# Download all latest Freedesktop.org specifications as Markdown into docs/freedesktop/
python3 tools/freedesktop_specs/download.py

# Preview what will be downloaded without writing files
python3 tools/freedesktop_specs/download.py --dry-run

# Download only matching specifications (e.g., basedir, desktop-entry)
python3 tools/freedesktop_specs/download.py --filter desktop

# Also keep raw HTML alongside the converted Markdown files
python3 tools/freedesktop_specs/download.py --keep-html --force
```

Generates `docs/freedesktop/INDEX.md` and `docs/freedesktop/manifest.json` alongside the converted `.md` specifications and `.dtd`/`.xml` schemas.
