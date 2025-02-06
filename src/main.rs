use macroquad::prelude::*;
mod consts;
use consts::*;

struct Canvas {
    pixels: Vec<Vec<Color>>,
}

#[macroquad::main("plow")]
async fn main() {
    let canvas_width = 100;
    let canvas_height = 100;

    let mut camera_grid_size: f32 = 35.;
    // make camera position default be at center of canvas
    let mut camera_x = canvas_width as f32 / 2. * camera_grid_size - screen_width() / 2.;
    let mut camera_y = canvas_height as f32 / 2. * camera_grid_size - screen_height() / 2.;

    let mut line = Vec::new();
    line.resize(canvas_width, WHITE);
    let mut rows = Vec::new();
    rows.resize(canvas_height, line);
    let canvas = Canvas { pixels: rows };

    loop {
        clear_background(BG_COLOR);

        // handle input
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_delta = mouse_delta_position();
            camera_x += mouse_delta.x as f32 * screen_width() / 2.;
            camera_y += mouse_delta.y as f32 * screen_height() / 2.;
        }
        let scroll = mouse_wheel();
        let mouse = mouse_position();
        if scroll.1 != 0. {
            // i dont know how to actually do this scroll stuff
            // but i made this curve in geogebra and i mean it looks cool
            // i assumed it should grow exponentially, since it shant ever reach 0, only ever approach it as x decreases
            let amt = 5_f32.powf(-0.001 * (scroll.1));
            // store old mouse position (in world position)
            let old_mouse_world_x = (mouse.0 + camera_x) / camera_grid_size;
            let old_mouse_world_y = (mouse.1 + camera_y) / camera_grid_size;
            // update grid size
            camera_grid_size /= amt;
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            camera_x = old_mouse_world_x * camera_grid_size - mouse.0;
            camera_y = old_mouse_world_y * camera_grid_size - mouse.1;
        }
        let mouse_world_x = ((mouse.0 + camera_x) / camera_grid_size).floor();
        let mouse_world_y = ((mouse.1 + camera_y) / camera_grid_size).floor();

        // draw canvas
        for (y, line) in canvas.pixels.iter().enumerate() {
            for (x, color) in line.iter().enumerate() {
                draw_rectangle(
                    x as f32 * camera_grid_size - camera_x,
                    y as f32 * camera_grid_size - camera_y,
                    camera_grid_size as f32,
                    camera_grid_size as f32,
                    *color,
                );
            }
        }

        // draw cursor
        draw_rectangle(
            mouse_world_x * camera_grid_size - camera_x,
            mouse_world_y * camera_grid_size - camera_y,
            camera_grid_size as f32,
            camera_grid_size as f32,
            CURSOR_COLOR,
        );

        // draw fps
        draw_rectangle(10., 10., 40., 20., WHITE);
        draw_text(&get_fps().to_string(), 20., 20., 16., BLACK);

        next_frame().await
    }
}
