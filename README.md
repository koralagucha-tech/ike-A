# ike_taunt

Skyline plugin for Super Smash Bros. Ultimate.
Ike's down taunt (left and right) shows the mesh `hanx123` and hides the mesh `shirt` at frame 20.

## Build locally

1. Install cargo-skyline: `cargo install cargo-skyline --locked`
2. Download the Skyline Rust toolchain: `cargo skyline update-std`
3. From this folder: `cargo skyline build --release`
4. The plugin is written to `target/aarch64-skyline-switch/release/libike_taunt.nro`

(Do not use `cargo skyline install` for setup: it sends the built plugin to a Switch over FTP.)

## Build on GitHub (no local setup)

1. Create a GitHub repository and upload the contents of this folder, including the `.github` folder.
2. Open the repository's **Actions** tab. The "Build plugin" workflow runs on every push, or run it by hand with **Run workflow**.
3. When it finishes, open the run and download the `ike_taunt` artifact. It contains `libike_taunt.nro`.

## Install

Copy `libike_taunt.nro` to:

`sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`

(on an emulator, the same path inside its SD/mod folder). Skyline and ARCropolis must be installed.

## Notes

- Mesh names are case-sensitive and must match your model exactly.
- The change applies to every Ike costume slot.
- To change the frame the swap happens on, edit `frame(agent.lua_state_agent, 20.0)` in `src/ike.rs`.
