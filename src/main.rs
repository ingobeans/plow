use egui_macroquad::egui::{self, Layout};
use macroquad::prelude::*;
mod consts;
use consts::*;
use quad_files::{FileInputResult, FilePicker};

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

/// Update texture from image. Region specifies which region of texture to update, if None, updates entire texture.
fn update_texture(texture: &mut Texture2D, image: &Image, region: Option<Rect>) {
    if let Some(region) = region {
        texture.update_part(
            &image.sub_image(region),
            region.x as i32,
            region.y as i32,
            region.w as i32,
            region.h as i32,
        );
    } else {
        *texture = Texture2D::from_image(image);
        texture.set_filter(FilterMode::Nearest);
    }
}

fn gen_empty_image(width: u16, height: u16) -> Image {
    let bytes = vec![0; width as usize * height as usize * 4];
    Image {
        width,
        bytes,
        height,
    }
}

fn validate_canvas_size(canvas_width: u16, canvas_height: u16) -> bool {
    if canvas_width.max(canvas_height) > 32768 {
        return false;
    }
    if canvas_width as u64 * canvas_height as u64 > 1073676289 {
        return false;
    }
    true
}

fn generate_camera_bounds_to_fit(canvas_width: u16, canvas_height: u16) -> (f32, f32, f32) {
    // make zoom to show entire canvas height
    let camera_grid_size: f32 = (screen_width() / canvas_height as f32 / 2.0).max(MIN_ZOOM);
    // make camera position default be at center of canvas
    let camera_x = canvas_width as f32 / 2. * camera_grid_size - screen_width() / 2.;
    let camera_y = canvas_height as f32 / 2. * camera_grid_size - screen_height() / 2.;
    (camera_grid_size, camera_x, camera_y)
}

#[macroquad::main("plow")]
async fn main() {
    let plow_header = format!("plow {}", env!("CARGO_PKG_VERSION"));
    println!("{}", plow_header);

    let mut canvas_width = 100;
    let mut canvas_height = 100;
    if !validate_canvas_size(canvas_width, canvas_height) {
        println!("image too big! no dimension may be greater than 32768, and the product of the width and height may not be greater than 1073676289");
        return;
    }
    println!("created {}x{} image!", canvas_width, canvas_height);

    // make zoom to show entire canvas height
    let (mut camera_grid_size, mut camera_x, mut camera_y) =
        generate_camera_bounds_to_fit(canvas_width, canvas_height);

    let grid_material = get_grid_material();
    let mut canvas = Canvas {
        image: gen_empty_image(canvas_width, canvas_height),
    };

    let mut canvas_texture = Texture2D::from_image(&canvas.image);
    canvas_texture.set_filter(FilterMode::Nearest);
    println!("created texture!");

    // set up file picker
    let mut file_picker = FilePicker::new();

    let mut new_file_window_open = false;
    let mut new_file_width = String::new();
    let mut new_file_height = String::new();

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
        // check if image has been loaded from file picker
        if let FileInputResult::Data(data) = file_picker.update() {
            println!("got data!");
            let image = Image::from_file_with_format(&data, None);
            if let Ok(image) = image {
                canvas_width = image.width() as u16;
                canvas_height = image.height() as u16;
                canvas.image = image;
                update_texture(&mut canvas_texture, &canvas.image, None);
                (camera_grid_size, camera_x, camera_y) =
                    generate_camera_bounds_to_fit(canvas_width, canvas_height);
            } else {
                println!("image failed to load");
            }
        }

        // define ui
        let mut mouse_over_ui = false;
        egui_macroquad::ui(|egui_ctx| {
            egui::TopBottomPanel::top("topbar").show(egui_ctx, |ui| {
                ui.with_layout(Layout::left_to_right(egui::Align::Max), |ui| {
                    ui.label(format!("[untitled - {}]", plow_header));
                    ui.menu_button("file", |ui| {
                        if ui.button("new").clicked() {
                            ui.close_menu();
                            new_file_window_open = !new_file_window_open;
                        };
                        if ui.button("open").clicked() {
                            ui.close_menu();
                            file_picker.open_dialog();
                        }
                    });
                    ui.label(format!("fps: {}", get_fps()));
                });
            });
            if new_file_window_open {
                egui::Window::new("new file")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        ui.label("enter canvas size");
                        egui::Grid::new("input").num_columns(2).show(ui, |ui| {
                            ui.label("width");
                            ui.text_edit_singleline(&mut new_file_width);
                            ui.end_row();
                            ui.label("height");
                            ui.text_edit_singleline(&mut new_file_height);
                            ui.end_row();
                            if ui.button("okay").clicked() {
                                if let Ok(width) = new_file_width.parse() {
                                    if let Ok(height) = new_file_height.parse() {
                                        canvas_width = width;
                                        canvas_height = height;
                                        canvas.image = gen_empty_image(width, height);
                                        update_texture(&mut canvas_texture, &canvas.image, None);
                                        (camera_grid_size, camera_x, camera_y) =
                                            generate_camera_bounds_to_fit(
                                                canvas_width,
                                                canvas_height,
                                            );
                                        new_file_window_open = false;
                                    }
                                }
                            };
                            if ui.button("cancel").clicked() {
                                new_file_window_open = false;
                            };
                        });
                    });
            }
            mouse_over_ui = egui_ctx.is_pointer_over_area();
        });

        // cursor is the mouse position in world/canvas coordinates
        let cursor_x = ((mouse.0 + camera_x) / camera_grid_size).floor();
        let cursor_y = ((mouse.1 + camera_y) / camera_grid_size).floor();
        let cursor_in_canvas = cursor_x >= 0.
            && (cursor_x as u16) < canvas_width
            && cursor_y >= 0.
            && (cursor_y as u16) < canvas_height
            && !mouse_over_ui;

        if cursor_in_canvas && is_mouse_button_down(MouseButton::Left) {
            // draw pixel if LMB is pressed
            canvas
                .image
                .set_pixel(cursor_x as u32, cursor_y as u32, WHITE);

            update_texture(
                &mut canvas_texture,
                &canvas.image,
                Some(Rect {
                    x: cursor_x,
                    y: cursor_y,
                    w: 1.,
                    h: 1.,
                }),
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

        // draw ui

        egui_macroquad::draw();

        next_frame().await
    }
}
