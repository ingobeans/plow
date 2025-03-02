# plow

![screenshot](https://github.com/user-attachments/assets/8377574f-cd10-440c-979e-d1e3a3324d99)

plow is an image editor made in rust with macroquad and egui. it can run in the browser with wasm or standalone cross platform.

## features

* 4 tools
* layers
* opening & saving files
* ctrl + z

## tools

* brush (B)
* eraser (E)
* bucket (F)
* color picker (K, or hold ALT)

## build

for standalone: `cargo run`

for wasm, with `basic-http-server`, do: `cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/plow.wasm web/ && basic-http-server web/`
