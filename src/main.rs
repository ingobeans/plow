use canvas::*;
use egui_macroquad::egui::{self, Layout};
use macroquad::prelude::*;
mod consts;
use consts::*;
use quad_files::{FileInputResult, FilePicker};
use tools::*;
mod canvas;
mod tools;

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

    let mut canvas = Canvas::new(DEFAULT_CANVAS_WIDTH, DEFAULT_CANVAS_HEIGHT).unwrap();
    let tools = get_tools();
    let mut active_tool = tools.first().unwrap();

    // make zoom to show entire canvas height
    let (mut camera_grid_size, mut camera_x, mut camera_y) =
        generate_camera_bounds_to_fit(canvas.width, canvas.height);

    let grid_material = get_grid_material();
    // set up file picker
    let mut file_picker = FilePicker::new();

    let mut primary_color = DEFAULT_PRIMARY_COLOR;
    let mut secondary_color = DEFAULT_SECONDARY_COLOR;

    let mut new_file_window_open = false;
    let mut new_file_width = String::new();
    let mut new_file_height = String::new();

    let mut rename_layer_window_open = false;
    let mut rename_layer_text = String::new();

    let mut last_cursor_x: Option<i16> = None;
    let mut last_cursor_y: Option<i16> = None;

    loop {
        clear_background(BG_COLOR);

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
        // store state of text inputs to compare if theyve been edited
        let pre_new_file_width = new_file_width.clone();
        let pre_new_file_height = new_file_height.clone();
        let pre_rename_layer_text = rename_layer_text.clone();

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
                            new_file_window_open = true;
                            new_file_width = DEFAULT_CANVAS_WIDTH.to_string();
                            new_file_height = DEFAULT_CANVAS_HEIGHT.to_string();
                        };
                        if ui.button("open").clicked() {
                            ui.close_menu();
                            file_picker.open_dialog();
                        }
                    });
                    ui.label(format!("fps: {}", get_fps()));
                });
            });
            // draw tools window
            egui::Window::new("tools").show(egui_ctx, |ui| {
                egui::Grid::new("tools grid").num_columns(2).show(ui, |ui| {
                    for tool in &tools {
                        let tool_name = tool.name();
                        let mut button = ui.button(&tool_name);
                        if tool_name == active_tool.name() {
                            button = button.highlight();
                        }
                        // make active if clicked
                        if button.clicked() {
                            active_tool = tool;
                        }
                    }
                });
            });
            // color picker
            egui::Window::new("colors").show(egui_ctx, |ui| {
                ui.color_edit_button_rgba_unmultiplied(&mut primary_color);
                ui.color_edit_button_rgba_unmultiplied(&mut secondary_color);
            });

            // draw layers window
            egui::Window::new("layers")
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0., 0.))
                .show(egui_ctx, |ui| {
                    let mut dragging_layer = None;
                    let response = quad_egui_dnd::dnd(ui, "layers").show(
                        canvas.layers.iter_mut().enumerate(),
                        |ui, (index, item), handle, state| {
                            ui.horizontal(|ui| {
                                handle.ui(ui, |ui| {
                                    ui.checkbox(&mut item.visible, "");
                                    let label = ui
                                        .button(&item.name)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if label.clicked() {
                                        canvas.current_layer = index;
                                    }
                                    if state.dragged {
                                        dragging_layer = Some(item.name.clone());
                                    }
                                    if label.double_clicked() {
                                        rename_layer_text = item.name.clone();
                                        rename_layer_window_open = true;
                                    }
                                    if index == canvas.current_layer {
                                        label.highlight();
                                    }
                                });
                            });
                        },
                    );
                    if response.is_drag_finished() {
                        response.update_vec(&mut canvas.layers);

                        // update the canvas.current_layer to the new position of the dragged item
                        if let Some(dragging_layer) = dragging_layer {
                            for (index, layer) in canvas.layers.iter().enumerate() {
                                if layer.name == dragging_layer {
                                    canvas.current_layer = index;
                                    break;
                                }
                            }
                        }
                    }
                    ui.separator();
                    ui.end_row();
                    if ui.button("new layer").clicked() {
                        canvas.new_layer();
                    }
                    if ui.button("delete layer").clicked() {
                        canvas.delete_layer();
                    }
                });
            // draw rename layer window
            if rename_layer_window_open {
                egui::Window::new("rename layer")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        egui::Grid::new("new file input")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("name");
                                ui.text_edit_singleline(&mut rename_layer_text);
                                ui.end_row();
                                if ui.button("okay").clicked() {
                                    let names = canvas
                                        .layers
                                        .iter()
                                        .map(|f| &f.name)
                                        .collect::<Vec<&String>>();
                                    if !names.contains(&&rename_layer_text) {
                                        rename_layer_window_open = false;
                                        canvas.layers[canvas.current_layer].name =
                                            rename_layer_text.clone();
                                    }
                                }
                                if ui.button("cancel").clicked() {
                                    rename_layer_window_open = false;
                                }
                            });
                    });
            }
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

            mouse_over_ui = egui_ctx.is_pointer_over_area() || egui_ctx.is_using_pointer();
        });
        let typing_in_text_box = {
            pre_new_file_height != new_file_height
                || pre_new_file_width != new_file_width
                || pre_rename_layer_text != rename_layer_text
        };

        // check for pressed keybinds (when the user isnt typing in a text box)
        if !typing_in_text_box {
            // check if a tool's keybind has been pressed and if so make it active
            for tool in &tools {
                if tool.keybind().is_some() && is_key_pressed(tool.keybind().unwrap()) {
                    active_tool = tool;
                    break;
                }
            }
            // and also check if x pressed, if so swap primary and secondary colors
            if is_key_pressed(KeyCode::X) {
                (primary_color, secondary_color) = (secondary_color, primary_color)
            }
        }

        let scroll = mouse_wheel();
        let mouse = mouse_position();

        // cursor is the mouse position in world/canvas coordinates
        let cursor_x = ((mouse.0 + camera_x) / camera_grid_size).floor() as i16;
        let cursor_y = ((mouse.1 + camera_y) / camera_grid_size).floor() as i16;
        let cursor_in_canvas = cursor_x >= 0
            && (cursor_x) < canvas.width as i16
            && cursor_y >= 0
            && (cursor_y) < canvas.height as i16;

        // handle input
        if !mouse_over_ui && is_mouse_button_down(MouseButton::Middle) {
            let mouse_delta = mouse_delta_position();
            camera_x += mouse_delta.x as f32 * screen_width() / 2.;
            camera_y += mouse_delta.y as f32 * screen_height() / 2.;
        }
        if !mouse_over_ui && scroll.1 != 0. {
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

        if !mouse_over_ui {
            active_tool.draw(ToolContext {
                layer: &mut canvas.layers[canvas.current_layer],
                cursor_x,
                cursor_y,
                last_cursor_x,
                last_cursor_y,
                primary_color,
                secondary_color,
            });
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
        for layer in canvas.layers.iter().rev() {
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
                (cursor_x as f32 * camera_grid_size - camera_x).floor(),
                (cursor_y as f32 * camera_grid_size - camera_y).floor(),
                camera_grid_size.floor(),
            );
        }

        // draw ui

        egui_macroquad::draw();
        last_cursor_x = Some(cursor_x);
        last_cursor_y = Some(cursor_y);
        next_frame().await
    }
}
