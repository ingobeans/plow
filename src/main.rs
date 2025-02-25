use canvas::*;
use egui_macroquad::egui::{self, Layout, WidgetText};
use macroquad::prelude::*;
mod consts;
use consts::*;
use quad_files::{FileInputResult, FilePicker};
use tools::*;
mod canvas;
mod tools;

/// Draw line with triple width, where center is white and edges are black.
///
/// Used to draw the cursor
fn draw_bold_line(x1: f32, y1: f32, x2: f32, y2: f32) {
    draw_line(x1, y1, x2, y2, 1., WHITE);

    let sides = if x1 == x2 {
        // if line is vertical, draw edges to the left and right
        [(-1., 0.), (1., 0.)]
    } else {
        // if line is horizontal, draw edges above and below
        [(0., -1.), (0., 1.)]
    };
    for (x_offset, y_offset) in sides {
        draw_line(
            x1 + x_offset,
            y1 + y_offset,
            x2 + x_offset,
            y2 + y_offset,
            1.,
            BLACK,
        );
    }
}

fn new_general_window(title: impl Into<WidgetText>, open: &mut bool) -> egui::Window<'_> {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .open(open)
}

fn get_new_canvas_name(canvases: &[Canvas]) -> String {
    // get a name for the new canvas (that isnt already used!!!!!)
    let mut canvas_name_index = canvases.len() + 1;
    let mut name = format!("{} {}", UNTITLED_NAME, canvas_name_index);
    let names = canvases.iter().map(|f| &f.name).collect::<Vec<&String>>();
    while names.contains(&&name) {
        canvas_name_index += 1;
        name = format!("{} {}", UNTITLED_NAME, canvas_name_index);
    }
    name
}

/// Works like macroquad's [Image::from_file_with_format], but uses our version of the `image` crate which has support enabled for image formats other than PNG
///
/// Also returns which [ImageFormat] was used
fn image_from_bytes(bytes: &[u8]) -> Result<(Image, ImageFormat), image::ImageError> {
    let format_guess = image::guess_format(bytes)?;
    let dynamic_image = image::load_from_memory_with_format(bytes, format_guess)?;
    let width = dynamic_image.width() as u16;
    let height = dynamic_image.height() as u16;
    let image_bytes = dynamic_image.to_rgba8().into_raw();
    Ok((
        Image {
            bytes: image_bytes,
            width,
            height,
        },
        format_guess,
    ))
}

#[macroquad::main("plow")]
async fn main() {
    let plow_header = format!("[plow {}]", env!("CARGO_PKG_VERSION"));
    println!("{}", plow_header);

    let mut canvases = vec![Canvas::new(
        DEFAULT_CANVAS_WIDTH,
        DEFAULT_CANVAS_HEIGHT,
        String::from(UNTITLED_NAME),
    )
    .unwrap()];
    let mut active_canvas = 0;

    let tools = get_tools();
    let mut tools_settings = ToolsSettings::new();
    let mut active_tool = tools.first().unwrap();

    let grid_material = get_grid_material();
    // set up file picker
    let mut file_picker = FilePicker::new();

    let mut primary_color = DEFAULT_PRIMARY_COLOR;
    let mut secondary_color = DEFAULT_SECONDARY_COLOR;

    let mut last_cursor_x: Option<i16> = None;
    let mut last_cursor_y: Option<i16> = None;

    let mut tool_before_holding_alt: Option<&Box<dyn Tool>> = None;

    // stroke to display for [CursorType::Point]
    let point_stroke = Stroke::new(1);

    // window states
    // ugly code, ui window problem x1
    let mut new_file_window_open = false;
    let mut new_file_width = String::new();
    let mut new_file_height = String::new();

    let mut rename_layer_window_open = false;
    let mut rename_layer_text = String::new();

    let mut colors_window_open = true;
    let mut tools_window_open = true;
    let mut layers_window_open = true;

    let mut typing_in_text_box = false;

    loop {
        clear_background(BG_COLOR);

        // check if image has been loaded from file picker
        if let FileInputResult::Data(data) = file_picker.update() {
            println!("got data!");
            let result = image_from_bytes(&data.bytes);

            let (name_without_extension, _) = data
                .name
                .rsplit_once('.')
                .unwrap_or((UNTITLED_NAME, UNTITLED_NAME));

            if let Ok((image, format)) = result {
                active_canvas = canvases.len();
                canvases.push(
                    Canvas::from_image(image, name_without_extension.to_string(), format).unwrap(),
                );
            } else {
                let err = result.unwrap_err();
                println!("image failed to load {}", err);
            }
        }
        // if alt was released restore previous tool from color picker'
        if is_key_released(KeyCode::LeftAlt) {
            if let Some(tool_before_holding_alt) = tool_before_holding_alt {
                active_tool = tool_before_holding_alt;
            }
            tool_before_holding_alt = None;
        }
        // if alt pressed, change tool to color picker
        if is_key_pressed(KeyCode::LeftAlt) && tool_before_holding_alt.is_none() {
            tool_before_holding_alt = Some(active_tool);
            active_tool = &tools[3];
        }

        // define ui
        let mut mouse_over_ui = false;
        egui_macroquad::ui(|egui_ctx| {
            // draw first topbar (info, canvases)
            egui::TopBottomPanel::top("topbar").show(egui_ctx, |ui| {
                ui.with_layout(Layout::left_to_right(egui::Align::Max), |ui| {
                    ui.label(plow_header.clone());
                    ui.separator();
                    for (index, canvas) in canvases.iter().enumerate() {
                        let mut text = canvas.name.clone();
                        if canvas.is_modified() {
                            text += "*"
                        }
                        let mut button = ui.button(text);
                        if index == active_canvas {
                            button = button.highlight();
                        }
                        if button.clicked() {
                            // change active canvas
                            active_canvas = index;
                            // reset vars
                            last_cursor_x = None;
                            last_cursor_y = None;
                        }
                    }
                });
            });
            // draw secondary topbar (navigation, tool settings)
            egui::TopBottomPanel::top("topbar2").show(egui_ctx, |ui| {
                ui.with_layout(Layout::left_to_right(egui::Align::Max), |ui| {
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
                        if ui.button("save").clicked() {
                            ui.close_menu();
                            canvases[active_canvas].export();
                        }
                    });
                    ui.menu_button("view", |ui| {
                        // ugly code, ui window problem x2
                        ui.checkbox(&mut tools_window_open, "tools");
                        ui.checkbox(&mut colors_window_open, "colors");
                        ui.checkbox(&mut layers_window_open, "layers");
                    });
                    ui.separator();
                    active_tool.draw_buttons(ui, &mut tools_settings);
                });
            });
            // ugly code for all windows, ui window problem x3
            if tools_window_open {
                // tools window
                new_general_window("tools", &mut tools_window_open).show(egui_ctx, |ui| {
                    egui::Grid::new("tools grid").num_columns(2).show(ui, |ui| {
                        for (index, tool) in tools.iter().enumerate() {
                            let tool_name = tool.name();
                            let mut button = ui.button(&tool_name);
                            if tool_name == active_tool.name() {
                                button = button.highlight();
                            }
                            // make active if clicked
                            if button.clicked() {
                                active_tool = tool;
                            }

                            // make every other tool break new line
                            if index % 2 != 0 {
                                ui.end_row();
                            }
                        }
                    });
                });
            }
            if colors_window_open {
                // color picker
                new_general_window("colors", &mut colors_window_open).show(egui_ctx, |ui| {
                    ui.color_edit_button_rgba_unmultiplied(&mut primary_color);
                    ui.color_edit_button_rgba_unmultiplied(&mut secondary_color);
                });
            }

            if layers_window_open {
                // layers window
                new_general_window("layers", &mut layers_window_open)
                    .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        let mut dragging_layer = None;
                        let canvas = &mut canvases[active_canvas];
                        let items = canvas.layers.iter_mut().enumerate();
                        let response = quad_egui_dnd::dnd(ui, "layers").show(
                            items,
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
                            response.update_vec(&mut canvases[active_canvas].layers);

                            // update the canvases[active_canvas].current_layer to the new position of the dragged item
                            if let Some(dragging_layer) = dragging_layer {
                                for (index, layer) in
                                    canvases[active_canvas].layers.iter().enumerate()
                                {
                                    if layer.name == dragging_layer {
                                        canvases[active_canvas].current_layer = index;
                                        break;
                                    }
                                }
                            }
                        }
                        ui.separator();
                        ui.end_row();
                        ui.horizontal(|ui| {
                            if ui
                                .button("new layer")
                                .on_hover_text("ctrl+shift+n")
                                .clicked()
                            {
                                canvases[active_canvas].new_layer();
                            }
                            let delete_layer_button = egui::Button::new("delete layer");
                            // make delete layer button disabled if only 1 layer
                            if ui
                                .add_enabled(
                                    canvases[active_canvas].layers.len() > 1,
                                    delete_layer_button,
                                )
                                .on_hover_text("ctrl+shift+delete")
                                .clicked()
                            {
                                canvases[active_canvas].delete_layer();
                            }
                            if ui
                                .button("duplicate layer")
                                .on_hover_text("ctrl+shift+d")
                                .clicked()
                            {
                                canvases[active_canvas].duplicate_layer();
                            }
                            let merge_down_button = egui::Button::new("merge down");
                            // make merge down button disabled if at bottom layer
                            if ui
                                .add_enabled(
                                    canvases[active_canvas].current_layer
                                        != canvases[active_canvas].layers.len() - 1,
                                    merge_down_button,
                                )
                                .on_hover_text("ctrl+m")
                                .clicked()
                            {
                                canvases[active_canvas].merge_layers_down();
                            }
                        });
                    });
            }
            // draw rename layer window
            if rename_layer_window_open {
                egui::Window::new("rename layer")
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        egui::Grid::new("new file input")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("name");
                                typing_in_text_box |=
                                    ui.text_edit_singleline(&mut rename_layer_text).changed();
                                ui.end_row();
                                if ui.button("okay").clicked() {
                                    let names = canvases[active_canvas]
                                        .layers
                                        .iter()
                                        .map(|f| &f.name)
                                        .collect::<Vec<&String>>();
                                    if !names.contains(&&rename_layer_text) {
                                        rename_layer_window_open = false;
                                        let active_layer = canvases[active_canvas].current_layer;
                                        canvases[active_canvas].layers[active_layer].name =
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
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                    .show(egui_ctx, |ui| {
                        ui.label("enter canvas size");
                        egui::Grid::new("new file input")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("width");
                                typing_in_text_box |=
                                    ui.text_edit_singleline(&mut new_file_width).changed();
                                ui.end_row();
                                ui.label("height");
                                typing_in_text_box |=
                                    ui.text_edit_singleline(&mut new_file_height).changed();
                                ui.end_row();
                                if ui.button("okay").clicked() {
                                    if let Ok(width) = new_file_width.parse() {
                                        if let Ok(height) = new_file_height.parse() {
                                            let new_name = get_new_canvas_name(&canvases);
                                            active_canvas = canvases.len();
                                            canvases.push(
                                                Canvas::new(width, height, new_name).unwrap(),
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

        // check for pressed keybinds (when the user isnt typing in a text box)
        if !typing_in_text_box {
            // check if a tool's keybind has been pressed and if so make it active
            for tool in &tools {
                if tool.keybind().is_some() && is_key_pressed(tool.keybind().unwrap()) {
                    active_tool = tool;
                    break;
                }
            }
            // x => swap primary and secondary color
            if is_key_pressed(KeyCode::X) {
                (primary_color, secondary_color) = (secondary_color, primary_color)
            } else if is_key_down(KeyCode::LeftControl) {
                // ctrl + m => merge layers down
                if is_key_pressed(KeyCode::M) {
                    canvases[active_canvas].merge_layers_down();
                }
                // ctrl + s => save/export
                else if is_key_pressed(KeyCode::S) {
                    canvases[active_canvas].export();
                }
                // ctrl + o => open file
                else if is_key_pressed(KeyCode::O) {
                    file_picker.open_dialog();
                }
                // otherwise check the ctrl+shift keybinds
                else if is_key_down(KeyCode::LeftShift) {
                    // ctrl+shift+n => new layer
                    if is_key_pressed(KeyCode::N) {
                        canvases[active_canvas].new_layer();
                    }
                    // ctrl+shift+delete => delete layer
                    else if is_key_pressed(KeyCode::Delete) {
                        canvases[active_canvas].delete_layer();
                    }
                    // ctrl+shift+d => duplicate layer
                    else if is_key_pressed(KeyCode::D) {
                        canvases[active_canvas].duplicate_layer();
                    }
                }
            }
        }

        let scroll = mouse_wheel();
        let mouse = mouse_position();

        // cursor is the mouse position in world/canvas coordinates
        let cursor_x = ((mouse.0 + canvases[active_canvas].camera_x)
            / canvases[active_canvas].camera_grid_size)
            .floor() as i16;
        let cursor_y = ((mouse.1 + canvases[active_canvas].camera_y)
            / canvases[active_canvas].camera_grid_size)
            .floor() as i16;
        let cursor_in_canvas = cursor_x >= 0
            && (cursor_x) < canvases[active_canvas].width as i16
            && cursor_y >= 0
            && (cursor_y) < canvases[active_canvas].height as i16;

        // handle input
        if !mouse_over_ui && is_mouse_button_down(MouseButton::Middle) {
            let mouse_delta = mouse_delta_position();
            canvases[active_canvas].camera_x += mouse_delta.x as f32 * screen_width() / 2.;
            canvases[active_canvas].camera_y += mouse_delta.y as f32 * screen_height() / 2.;
        }
        if !mouse_over_ui && scroll.1 != 0. {
            let amt = if scroll.1 > 0. {
                1. / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x = (mouse.0 + canvases[active_canvas].camera_x)
                / canvases[active_canvas].camera_grid_size;
            let old_mouse_world_y = (mouse.1 + canvases[active_canvas].camera_y)
                / canvases[active_canvas].camera_grid_size;
            // update grid size
            canvases[active_canvas].camera_grid_size /= amt;
            canvases[active_canvas].camera_grid_size =
                canvases[active_canvas].camera_grid_size.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            canvases[active_canvas].camera_x =
                old_mouse_world_x * canvases[active_canvas].camera_grid_size - mouse.0;
            canvases[active_canvas].camera_y =
                old_mouse_world_y * canvases[active_canvas].camera_grid_size - mouse.1;
        }

        if !mouse_over_ui {
            let active_layer = canvases[active_canvas].current_layer;
            let layer = &mut canvases[active_canvas].layers[active_layer];
            let cursor_in_bounds = !(cursor_x < 0
                || cursor_y < 0
                || cursor_x as usize >= layer.width()
                || cursor_y as usize >= layer.height());
            active_tool.update(ToolContext {
                layer,
                cursor_x,
                cursor_y,
                cursor_in_bounds,
                last_cursor_x,
                last_cursor_y,
                primary_color: &mut primary_color,
                secondary_color: &mut secondary_color,
                settings: &mut tools_settings,
            });
        }

        // draw grid background behind canvas
        gl_use_material(&grid_material);
        draw_rectangle(
            -canvases[active_canvas].camera_x,
            -canvases[active_canvas].camera_y,
            canvases[active_canvas].width as f32 * canvases[active_canvas].camera_grid_size,
            canvases[active_canvas].height as f32 * canvases[active_canvas].camera_grid_size,
            WHITE,
        );
        gl_use_default_material();
        // draw canvas
        let draw_params = DrawTextureParams {
            dest_size: Some(vec2(
                (canvases[active_canvas].width as f32 * canvases[active_canvas].camera_grid_size)
                    .floor(),
                (canvases[active_canvas].height as f32 * canvases[active_canvas].camera_grid_size)
                    .floor(),
            )),
            ..Default::default()
        };
        for layer in canvases[active_canvas].layers.iter().rev() {
            if layer.visible {
                draw_texture_ex(
                    &layer.texture,
                    -canvases[active_canvas].camera_x,
                    -canvases[active_canvas].camera_y,
                    WHITE,
                    draw_params.clone(),
                );
            }
        }

        // draw cursor (if in bounds)
        if cursor_in_canvas && !mouse_over_ui {
            let stroke = match active_tool.cursor_type() {
                CursorType::Stroke => &tools_settings.stroke,
                CursorType::Point => &point_stroke,
            };
            for ((x1, y1), (x2, y2)) in &stroke.borders {
                let x1 = (cursor_x + *x1 as i16 + stroke.pixels_offset) as f32
                    * canvases[active_canvas].camera_grid_size
                    - canvases[active_canvas].camera_x;
                let y1 = (cursor_y + *y1 as i16 + stroke.pixels_offset) as f32
                    * canvases[active_canvas].camera_grid_size
                    - canvases[active_canvas].camera_y;
                let x2 = (cursor_x + *x2 as i16 + stroke.pixels_offset) as f32
                    * canvases[active_canvas].camera_grid_size
                    - canvases[active_canvas].camera_x;
                let y2 = (cursor_y + *y2 as i16 + stroke.pixels_offset) as f32
                    * canvases[active_canvas].camera_grid_size
                    - canvases[active_canvas].camera_y;
                draw_bold_line(x1, y1, x2, y2);
            }
        }

        // draw ui

        egui_macroquad::draw();
        last_cursor_x = Some(cursor_x);
        last_cursor_y = Some(cursor_y);
        next_frame().await
    }
}
