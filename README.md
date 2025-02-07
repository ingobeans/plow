# plow

![image](https://github.com/user-attachments/assets/15997c8c-afba-460c-ace4-736ddda708c0)

plow is an image editor made in rust with macroquad and egui. it can run in the browser with wasm or standalone cross platform.


## build

for standalone: `cargo run`

for wasm, with `basic-http-server`, do: `cargo build --target wasm32-unknown-unknown; basic-http-server .\web\`
