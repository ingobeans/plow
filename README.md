# plow

![screenshot](https://github.com/user-attachments/assets/8377574f-cd10-440c-979e-d1e3a3324d99)

plow is an image editor made in rust with macroquad and egui. it can run in the browser with wasm or standalone cross platform.


## build

for standalone: `cargo run`

for wasm, with `basic-http-server`, do: `cargo build --target wasm32-unknown-unknown; basic-http-server .\web\`
