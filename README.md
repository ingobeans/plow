# plow

![screenshot](https://github.com/user-attachments/assets/7e66ce0f-23fd-40a4-8054-cb1a0617ebe8)

plow is an image editor made in rust with macroquad and egui. it can run in the browser with wasm or standalone cross platform.


## build

for standalone: `cargo run`

for wasm, with `basic-http-server`, do: `cargo build --target wasm32-unknown-unknown; basic-http-server .\web\`
