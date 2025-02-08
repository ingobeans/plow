use canvas::*;
use egui_macroquad::egui::{self, Layout};
use macroquad::prelude::*;
mod consts;
use consts::*;
use quad_files::{FileInputResult, FilePicker};
mod canvas;

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

    let mut canvas = Canvas::new(100, 100).unwrap();

    // make zoom to show entire canvas height
    let (mut camera_grid_size, mut camera_x, mut camera_y) =
        generate_camera_bounds_to_fit(canvas.width, canvas.height);

    let grid_material = get_grid_material();
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
                canvas = Canvas::from_image(image).unwrap();
                (camera_grid_size, camera_x, camera_y) =
                    generate_camera_bounds_to_fit(canvas.width, canvas.height);
            } else {
                println!("image failed to load");
            }
        }

        // define ui
        let mut mouse_over_ui = false;
        egui_macroquad::ui(|egui_ctx| {
            // draw topbar
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
            // draw layers window
            egui::Window::new("layers")
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0., 0.))
                .show(egui_ctx, |ui| {
                    egui::Grid::new("layers").num_columns(2).show(ui, |ui| {
                        for (index, layer) in canvas.layers.iter_mut().enumerate().rev() {
                            ui.checkbox(&mut layer.visible, "");
                            let label = ui
                                .label(&layer.name)
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if label.clicked() {
                                canvas.current_layer = index;
                            }
                            if index == canvas.current_layer {
                                label.highlight();
                            }
                            ui.end_row();
                        }
                    });
                    // draw bottom bar
                    ui.separator();
                    ui.end_row();
                    if ui.button("new layer").clicked() {
                        canvas.new_layer();
                    }
                    if ui.button("delete layer").clicked() {
                        canvas.delete_layer();
                    }
                });
            // draw new file window
            if new_file_window_open {
                egui::Window::new("new file")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        ui.label("enter canvas size");
                        egui::Grid::new("new file input")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("width");
                                ui.text_edit_singleline(&mut new_file_width);
                                ui.end_row();
                                ui.label("height");
                                ui.text_edit_singleline(&mut new_file_height);
                                ui.end_row();
                                if ui.button("okay").clicked() {
                                    if let Ok(width) = new_file_width.parse() {
                                        if let Ok(height) = new_file_height.parse() {
                                            canvas = Canvas::new(width, height).unwrap();
                                            (camera_grid_size, camera_x, camera_y) =
                                                generate_camera_bounds_to_fit(
                                                    canvas.width,
                                                    canvas.height,
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
            && (cursor_x as u16) < canvas.width
            && cursor_y >= 0.
            && (cursor_y as u16) < canvas.height
            && !mouse_over_ui;

        if cursor_in_canvas && is_mouse_button_down(MouseButton::Left) {
            // draw pixel if LMB is pressed
            canvas.layers[canvas.current_layer].image.set_pixel(
                cursor_x as u32,
                cursor_y as u32,
                WHITE,
            );

            canvas.layers[canvas.current_layer].update_texture(Some(Rect {
                x: cursor_x,
                y: cursor_y,
                w: 1.,
                h: 1.,
            }));
        }

        // draw grid background behind canvas
        gl_use_material(&grid_material);
        draw_rectangle(
            -camera_x,
            -camera_y,
            canvas.width as f32 * camera_grid_size,
            canvas.height as f32 * camera_grid_size,
            WHITE,
        );
        gl_use_default_material();
        // draw canvas
        let draw_params = DrawTextureParams {
            dest_size: Some(vec2(
                (canvas.width as f32 * camera_grid_size).floor(),
                (canvas.height as f32 * camera_grid_size).floor(),
            )),
            ..Default::default()
        };
        for layer in &canvas.layers {
            if layer.visible {
                draw_texture_ex(
                    &layer.texture,
                    -camera_x,
                    -camera_y,
                    WHITE,
                    draw_params.clone(),
                );
            }
        }

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
