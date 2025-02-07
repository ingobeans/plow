use macroquad::prelude::*;
mod consts;
use consts::*;

struct Canvas {
    image: Image,
}

fn draw_cursor_at(cursor_x: f32, cursor_y: f32, camera_grid_size: f32) {
    draw_rectangle_lines(
        cursor_x - 3.,
        cursor_y - 3.,
        camera_grid_size + 6.,
        camera_grid_size + 6.,
        2.,
        BLACK,
    );
    draw_rectangle_lines(
        cursor_x + -2.,
        cursor_y + -2.,
        camera_grid_size + 4.,
        camera_grid_size + 4.,
        2.,
        WHITE,
    );
    draw_rectangle_lines(
        cursor_x - 1.,
        cursor_y - 1.,
        camera_grid_size + 2.,
        camera_grid_size + 2.,
        2.,
        BLACK,
    );
}

fn update_region(texture: &Texture2D, image: &Image, region: Rect) {
    texture.update_part(
        &image.sub_image(region),
        region.x as i32,
        region.y as i32,
        region.w as i32,
        region.h as i32,
    );
}

fn gen_empty_image(width: u16, height: u16) -> Image {
    let bytes = vec![0; width as usize * height as usize * 4];
    Image {
        width,
        bytes,
        height,
    }
}

#[macroquad::main("plow")]
async fn main() {
    println!("plow!");
    let canvas_width = 100;
    let canvas_height = 100;

    // make zoom to show entire canvas height
    let mut camera_grid_size: f32 = (screen_width() / canvas_height as f32 / 2.0).max(MIN_ZOOM);
    // make camera position default be at center of canvas
    let mut camera_x = canvas_width as f32 / 2. * camera_grid_size - screen_width() / 2.;
    let mut camera_y = canvas_height as f32 / 2. * camera_grid_size - screen_height() / 2.;

    let grid_material = get_grid_material();
    let mut canvas = Canvas {
        image: gen_empty_image(canvas_width, canvas_height),
    };

    let canvas_texture = Texture2D::from_image(&canvas.image);
    canvas_texture.set_filter(FilterMode::Nearest);

    loop {
        clear_background(BG_COLOR);

        // handle input
        if is_mouse_button_down(MouseButton::Middle) {
            let mouse_delta = mouse_delta_position();
            camera_x += mouse_delta.x as f32 * screen_width() / 2.;
            camera_y += mouse_delta.y as f32 * screen_height() / 2.;
        }
        let scroll = mouse_wheel();
        let mouse = mouse_position();
        if scroll.1 != 0. {
            let amt = if scroll.1 > 0. {
                1. / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x = (mouse.0 + camera_x) / camera_grid_size;
            let old_mouse_world_y = (mouse.1 + camera_y) / camera_grid_size;
            // update grid size
            camera_grid_size /= amt;
            camera_grid_size = camera_grid_size.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            camera_x = old_mouse_world_x * camera_grid_size - mouse.0;
            camera_y = old_mouse_world_y * camera_grid_size - mouse.1;
        }
        // cursor is the mouse position in world/canvas coordinates
        let cursor_x = ((mouse.0 + camera_x) / camera_grid_size).floor();
        let cursor_y = ((mouse.1 + camera_y) / camera_grid_size).floor();
        let cursor_in_canvas = cursor_x >= 0.
            && (cursor_x as u16) < canvas_width
            && cursor_y >= 0.
            && (cursor_y as u16) < canvas_height;

        if cursor_in_canvas && is_mouse_button_down(MouseButton::Left) {
            // draw pixel if LMB is pressed
            canvas
                .image
                .set_pixel(cursor_x as u32, cursor_y as u32, WHITE);

            update_region(
                &canvas_texture,
                &canvas.image,
                Rect {
                    x: cursor_x,
                    y: cursor_y,
                    w: 1.,
                    h: 1.,
                },
            );
        }

        // draw grid background behind canvas
        gl_use_material(&grid_material);
        draw_rectangle(
            -camera_x,
            -camera_y,
            canvas_width as f32 * camera_grid_size,
            canvas_height as f32 * camera_grid_size,
            WHITE,
        );
        gl_use_default_material();
        // draw canvas
        let draw_params = DrawTextureParams {
            dest_size: Some(vec2(
                (canvas_width as f32 * camera_grid_size).floor(),
                (canvas_height as f32 * camera_grid_size).floor(),
            )),
            ..Default::default()
        };
        draw_texture_ex(&canvas_texture, -camera_x, -camera_y, WHITE, draw_params);

        // draw cursor (if in bounds)
        if cursor_in_canvas {
            draw_cursor_at(
                (cursor_x * camera_grid_size - camera_x).floor(),
                (cursor_y * camera_grid_size - camera_y).floor(),
                camera_grid_size.floor(),
            );
        }

        // draw fps
        draw_rectangle(10., 10., 40., 20., WHITE);
        draw_text(&get_fps().to_string(), 20., 20., 16., BLACK);

        next_frame().await
    }
}
