"""SpectreOS release component registry (standard library only).

``codes/shiraz-core/**`` is a separate project and is never versioned, scanned
or edited by the release tooling.
"""

import dataclasses
import re

REPOSITORY = "https://github.com/spectre-oss/spectreos"
EXCLUDED_PREFIXES = ("codes/shiraz-core/",)
VERSION = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")


@dataclasses.dataclass(frozen=True)
class Component:
    name: str
    directory: str
    version_file: str
    kind: str  # crate | app | embedded-app | os
    package: str = ""  # Cargo package name ("" for the OS)
    binary: str = ""
    features: str = ""

    @property
    def cargo(self):
        return bool(self.package)

    @property
    def manifest(self):
        return f"{self.directory}/Cargo.toml" if self.cargo else ""

    @property
    def lockfile(self):
        return f"{self.directory}/Cargo.lock" if self.cargo else ""

    @property
    def changelog(self):
        return f"{self.directory}/CHANGELOG.md"

    def tag(self, version):
        return f"{self.name}-v{version}"

    @property
    def tag_pattern(self):
        return re.compile(re.escape(self.name) + r"-v((?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*))\Z")


COMPONENTS = (
    Component("libbhoo", "codes/libs/ui/libbhoo", "codes/libs/ui/libbhoo/.version", "crate", package="libbhoo"),
    Component("wisp", "codes/apps/wisp", "codes/apps/wisp/.version", "app",
              package="wisp", binary="wisp", features="compositor"),
    Component("keyboard", "codes/apps/keyboard", "codes/apps/keyboard/.version", "app",
              package="spectre-keyboard", binary="spectre-keyboard", features="osk"),
    Component("settings", "codes/apps/settings", "codes/apps/settings/.version", "app",
              package="spectre-settings", binary="spectre-settings", features="wayland"),
    Component("files", "codes/apps/files", "codes/apps/files/.version", "app",
              package="spectre-files", binary="spectre-files", features="wayland"),
    Component("calendar", "codes/apps/calendar", "codes/apps/calendar/.version", "embedded-app",
              package="spectre-calendar"),
    Component("spectref1", "codes/os/specrte-nix-f1", "codes/os/specrte-nix-f1/VERSION", "os"),
)
BY_NAME = {c.name: c for c in COMPONENTS}

# Static landing-page inputs: Pages are rebuilt for any change to these.
LANDING_PREFIXES = ("landing/", "docs/")
LANDING_FILES = ("README.md",)
LANDING_BASENAMES_UNDER_CODES = ("README.md", ".version")
LANDING_VERSION_FILES = ("codes/os/specrte-nix-f1/VERSION",)

# PR check targets: name -> (directory, cargo package, apps that depend on it).
CHECK_APPS = {
    "wisp": ("codes/apps/wisp", "wisp"),
    "keyboard": ("codes/apps/keyboard", "spectre-keyboard"),
    "settings": ("codes/apps/settings", "spectre-settings"),
    "files": ("codes/apps/files", "spectre-files"),
    "calendar": ("codes/apps/calendar", "spectre-calendar"),
    "bhoo-web": ("codes/apps/bhoo-web", "bhoo-web-demo"),
    "wisp_remote_viewer": ("codes/utils/wisp_remote_viewer", "wisp_remote_viewer"),
}
# A change to the key must also re-check these dependents (path dependencies).
CHECK_DEPENDENTS = {
    "files": ("calendar", "wisp", "bhoo-web"),
    "settings": ("wisp", "bhoo-web"),
    "calendar": ("wisp", "bhoo-web"),
    "wisp": ("bhoo-web",),
}
LIBBHOO_DIR = "codes/libs/ui/libbhoo"
SPECTREF1_DIR = "codes/os/specrte-nix-f1"
# Workflow/root files that invalidate every check.
ROOT_PREFIXES = (".github/", ".cargo/")
ROOT_FILES = ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml")


def excluded(path):
    return any(path.startswith(prefix) for prefix in EXCLUDED_PREFIXES)


def under(path, directory):
    return path == directory or path.startswith(directory + "/")


def is_landing_path(path):
    if excluded(path):
        return False
    if path in LANDING_FILES or path in LANDING_VERSION_FILES or any(path.startswith(p) for p in LANDING_PREFIXES):
        return True
    return path.startswith("codes/") and path.rsplit("/", 1)[-1] in LANDING_BASENAMES_UNDER_CODES
