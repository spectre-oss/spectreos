[Jump to content](#_content)[Jump to page navigation: previous page [access key p]/next page [access key n]](#_bottom-navigation)[AppStream](https://www.freedesktop.org/software/appstream/docs/)[AppStream](https://www.freedesktop.org/software/appstream/docs/index.html)Navigation←[→](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-About.html)[AppStream](https://www.freedesktop.org/software/appstream/docs/index.html)[Top](#)←[→](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-About.html)

###### AppStream 1.2

# AppStream

## Infrastructure for distro-agnostic software-centers and universal software component metadata

 AppStream is a collaborative effort for enhancing the way we interact with the software repositories provided by the distribution by standardizing sets of additional metadata.

 AppStream provides the foundation to build software-center applications. It additionally provides specifications for things like a unified software metadata database, screenshot services and various other things needed to create user-friendly application-centers for software distributions.

 You can find a C/GLib based reference implementation of the full specification at [github.com/ximion/appstream (https://github.com/ximion/appstream)](https://github.com/ximion/appstream) and report bugs about the implementation as well as the specification itself at the [issue tracker (https://github.com/ximion/appstream/issues)](https://github.com/ximion/appstream/issues). The project also provides a web form for the simple creation of metainfo XML files for upstream maintainers at [freedesktop.org/software/appstream/metainfocreator (https://www.freedesktop.org/software/appstream/metainfocreator/#/)](https://www.freedesktop.org/software/appstream/metainfocreator/#/).

Author: Matthias KlumppContributors: Michael Vogt, Richard Hughes, and Vincent UntzPublication Date: 2026-11-10

- **[1 About AppStream](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-About.html)**

- **[1.1 What is AppStream?](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-About.html#about-whatis)**

- **[2 Upstream Metadata](https://www.freedesktop.org/software/appstream/docs/chap-Metadata.html)**

- **[2.1 Generic Component](https://www.freedesktop.org/software/appstream/docs/chap-Metadata.html#sect-Metadata-GenericComponent)**
- **[2.2 Release Information](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Releases.html)**
- **[2.3 Desktop Applications](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Application.html)**
- **[2.4 Console Applications](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-ConsoleApplication.html)**
- **[2.5 Web Applications](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-WebApplication.html)**
- **[2.6 Services](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Service.html)**
- **[2.7 Addons](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Addon.html)**
- **[2.8 Fonts](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Fonts.html)**
- **[2.9 Icon Themes](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-IconTheme.html)**
- **[2.10 Codecs](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Codec.html)**
- **[2.11 Input Methods](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-InputMethod.html)**
- **[2.12 Firmware](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Firmware.html)**
- **[2.13 Driver](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Driver.html)**
- **[2.14 Localization](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Localization.html)**
- **[2.15 Repositories](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Repository.html)**
- **[2.16 Operating System](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-OS.html)**
- **[2.17 Runtime](https://www.freedesktop.org/software/appstream/docs/sect-Metadata-Runtime.html)**

- **[3 Catalog Metadata](https://www.freedesktop.org/software/appstream/docs/chap-CatalogData.html)**

- **[3.1 AppStream Catalog XML](https://www.freedesktop.org/software/appstream/docs/chap-CatalogData.html#sect-AppStream-XML)**
- **[3.2 AppStream Catalog YAML](https://www.freedesktop.org/software/appstream/docs/sect-AppStream-YAML.html)**
- **[3.3 Icon Cache](https://www.freedesktop.org/software/appstream/docs/sect-AppStream-IconCache.html)**

- **[4 Miscellaneous](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-Misc.html)**

- **[4.1 Version Comparison Algorithm](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-Misc.html#sect-AppStream-Misc-VerCmp)**
- **[4.2 URI Handler](https://www.freedesktop.org/software/appstream/docs/sect-AppStream-Misc-URIHandler.html)**

- **[5 Metadata Quickstart](https://www.freedesktop.org/software/appstream/docs/chap-Quickstart.html)**

- **[5.1 For GUI application upstream maintainers](https://www.freedesktop.org/software/appstream/docs/chap-Quickstart.html#sect-Quickstart-DesktopApps)**
- **[5.2 For upstream projects providing addons](https://www.freedesktop.org/software/appstream/docs/sect-Quickstart-Addons.html)**
- **[5.3 For distributors packaging Appstream metadata](https://www.freedesktop.org/software/appstream/docs/sect-Quickstart-Distros.html)**
- **[5.4 Maintaining release information](https://www.freedesktop.org/software/appstream/docs/sect-Quickstart-Releases.html)**
- **[5.5 Translating Metadata](https://www.freedesktop.org/software/appstream/docs/sect-Quickstart-Translation.html)**

- **[6 Data Validation](https://www.freedesktop.org/software/appstream/docs/chap-Validation.html)**

- **[6.1 Validator Issues](https://www.freedesktop.org/software/appstream/docs/chap-Validation.html#sect-ValidatorIssues)**

- **[7 Manual pages](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-ManualPages.html)**

- **[appstreamcli](https://www.freedesktop.org/software/appstream/docs/re01.html) — Handle AppStream metadata formats and query AppStream data**
- **[appstreamcli compose](https://www.freedesktop.org/software/appstream/docs/re02.html) — Compose AppStream metadata catalog from directory trees**

- **[8 AppStream API Reference](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-API.html)**
- **[Index](https://www.freedesktop.org/software/appstream/docs/ix01.html)**

 Copyright © 2012-2026 Matthias Klumpp. This documentation is dual-licensed under the [GNU Free Documentation License (GFDL), v1.3 (https://www.gnu.org/licenses/fdl.txt)](https://www.gnu.org/licenses/fdl.txt) with no invariant sections or later or the [CC Attribution-ShareAlike 4.0 International (https://creativecommons.org/licenses/by-sa/4.0/)](https://creativecommons.org/licenses/by-sa/4.0/) license (SPDX: CC-BY-SA-4.0 OR GFDL-1.3-or-later).

[→Chapter 1 About AppStream](https://www.freedesktop.org/software/appstream/docs/chap-AppStream-About.html)

© 2026 Freedesktop.org, the AppStream Project
