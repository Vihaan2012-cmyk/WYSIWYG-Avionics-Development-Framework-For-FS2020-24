# WYSIWYG Avionics Builder

A visual editor for Microsoft Flight Simulator 2020/2024 glass-cockpit displays.

Here is the workflow:

1. Define displays in `avdev.json` in your aircraft repo.
2. Let the app scan the aircraft for every LVAR, SimVar and event.
3. Draw the displays on a canvas with nested layers.
4. Bind properties to variables. Bindings can be presets, expressions or a node graph.
5. Add states, state machines and pages.

Every edit is written live to readable MSFS Avionics Framework TSX that you can also edit by hand.

Compiling the project does three things. It runs a strict, rustc-style checker. It bundles the result with [Mach](https://github.com/synapticsim/mach). It produces a ready-to-open [FlyByWire ACE](https://github.com/flybywiresim/ace) project.

- Design spec: [`docs/superpowers/specs/2026-10-08-wysiwyg-avionics-builder-design.md`](docs/superpowers/specs/2026-10-08-wysiwyg-avionics-builder-design.md)
- Platforms: Windows and Linux. The app is built with Tauri 2 (Rust core, React UI).

## Developing

**Requirements:**
- Rust stable. On Windows use the MSVC toolchain: `rustup override set stable-x86_64-pc-windows-msvc`.
- Node 22 or later.
- On Linux, the Tauri system dependencies (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `patchelf`).

```sh
npm install
cargo test --workspace
npm test
npm run tauri dev
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
