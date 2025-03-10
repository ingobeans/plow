# plow

![gif](https://github.com/user-attachments/assets/3e1b3fff-22e3-4927-860e-8db6337ff26d)


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
