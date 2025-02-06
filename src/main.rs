use macroquad::prelude::*;
mod consts;
use consts::*;

#[macroquad::main("plow")]
async fn main() {
    let mut camera_x = -screen_width() / 2.;
    let mut camera_y = -screen_height() / 2.;
    let mut camera_grid_size: f32 = 35.;
    loop {
        clear_background(BG_COLOR);
        // handle input
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_delta = mouse_delta_position();
            camera_x += mouse_delta.x as f32 * screen_width() / 2.;
            camera_y += mouse_delta.y as f32 * screen_height() / 2.;
        }
        let scroll = mouse_wheel();
        if scroll.1 != 0. {
            let mouse = mouse_position();
            // i dont know how to actually do this scroll stuff
            // but i made this curve in geogebra and i mean it looks cool
            // i assumed it should grow exponentially, since it shant ever reach 0, only ever approach it as x decreases
            let amt = 5_f32.powf(-0.001 * (scroll.1));
            // store old mouse position (in world position)
            let x = (mouse.0 + camera_x) / camera_grid_size;
            let y = (mouse.1 + camera_y) / camera_grid_size;
            // update grid size
            camera_grid_size /= amt;
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            camera_x = x * camera_grid_size - mouse.0;
            camera_y = y * camera_grid_size - mouse.1;
        }
        draw_rectangle(
            5. * camera_grid_size - camera_x,
            2. * camera_grid_size - camera_y,
            camera_grid_size as f32,
            camera_grid_size as f32,
            GREEN,
        );

        next_frame().await
    }
}
