use macroquad::prelude::*;
mod consts;
use consts::*;

struct Canvas {
    image: Image,
}

fn draw_cursor_at(cursor_x: f32, cursor_y: f32, camera_grid_size: f32) {
    draw_rectangle_lines(
        cursor_x,
        cursor_y,
        camera_grid_size,
        camera_grid_size,
        BORDER_WIDTH,
        BLACK,
    );
    draw_rectangle_lines(
        cursor_x + BORDER_WIDTH / 2.,
        cursor_y + BORDER_WIDTH / 2.,
        camera_grid_size - BORDER_WIDTH,
        camera_grid_size - BORDER_WIDTH,
        BORDER_WIDTH,
        WHITE,
    );
    draw_rectangle_lines(
        cursor_x + BORDER_WIDTH,
        cursor_y + BORDER_WIDTH,
        camera_grid_size - BORDER_WIDTH * 2.,
        camera_grid_size - BORDER_WIDTH * 2.,
        BORDER_WIDTH,
        BLACK,
    );
}

fn update_region(texture: &Texture2D, image: &Image, region: Rect) {
    texture.update_part(&image.sub_image(region), region.x as i32, region.y as i32, region.w as i32, region.h as i32);
}

fn gen_empty_image(width: u16, height: u16) -> Image {
    let bytes = vec![0; width as usize*height as usize*4];
    Image {
        width,bytes,height
    }
}

#[macroquad::main("plow")]
async fn main() {
    println!("plow!");
    let canvas_width = 100;
    let canvas_height = 100;

    let mut camera_grid_size: f32 = 35.;
    // make camera position default be at center of canvas
    let mut camera_x = canvas_width as f32 / 2. * camera_grid_size - screen_width() / 2.;
    let mut camera_y = canvas_height as f32 / 2. * camera_grid_size - screen_height() / 2.;

    let grid_material = get_grid_material();
    let mut canvas = Canvas {
        image: gen_empty_image(canvas_width, canvas_height)
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
            camera_grid_size = camera_grid_size.max(0.1);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            camera_x = old_mouse_world_x * camera_grid_size - mouse.0;
            camera_y = old_mouse_world_y * camera_grid_size - mouse.1;
        }
        let mouse_world_x = ((mouse.0 + camera_x) / camera_grid_size).floor();
        let mouse_world_y = ((mouse.1 + camera_y) / camera_grid_size).floor();
        let cursor_in_canvas = mouse_world_x >= 0.
            && (mouse_world_x as u16) < canvas_width
            && mouse_world_y >= 0.
            && (mouse_world_y as u16) < canvas_height;

        if cursor_in_canvas && is_mouse_button_down(MouseButton::Left) {
            // draw pixel if LMB is pressed
            canvas
                .image
                .set_pixel(mouse_world_x as u32, mouse_world_y as u32, WHITE);
            
            update_region(&canvas_texture, &canvas.image, Rect {
                x: mouse_world_x,
                y: mouse_world_y,
                w: 1.,
                h: 1.
            });
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
            let cursor_x = mouse_world_x * camera_grid_size - camera_x;
            let cursor_y = mouse_world_y * camera_grid_size - camera_y;
            draw_cursor_at(cursor_x.floor(), cursor_y.floor(), camera_grid_size);
        }

        // draw fps
        draw_rectangle(10., 10., 40., 20., WHITE);
        draw_text(&get_fps().to_string(), 20., 20., 16., BLACK);

        next_frame().await
    }
}
