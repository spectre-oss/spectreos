SpectreOS is an open-source mobile Linux operating system, Wayland compositor/shell, UI toolkit, native application suite and Android compatibility stack, organised as one monorepo. These pages are generated from the documentation that lives next to the code, so they follow the repository rather than a separate copy of it.

## How the pieces fit

- **Spectre NixOS · Poco F1** is the operating-system layer: a Mobile NixOS configuration with a mainline SDM845 kernel for the Xiaomi Pocophone F1 (`beryllium`).
- **Wisp** is the Wayland compositor, system UI, lock screen and launcher, built on Smithay (DRM/KMS, GBM, EGL/GLES3, libinput) and on libbhoo.
- **libbhoo** is the Rust UI toolkit (`bhoo` in code) that draws Wisp and every native app, with GLES 3.0, Wayland and WebAssembly backends.
- **Native apps** (Keyboard, Settings, Files, Calendar) are Rust programs written against libbhoo.
- **Shiraz** is the Android compatibility layer. It lives in a separate repository, [spectre-oss/shiraz](https://github.com/spectre-oss/shiraz), and is mounted at `codes/shiraz-core` as a Git submodule. Wisp and Spectre NixOS integrate it; its own documentation is maintained in that repository.
