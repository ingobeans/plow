use std::ops::RangeInclusive;

use egui_macroquad::egui::{DragValue, Slider, Ui};
use macroquad::prelude::*;

use crate::canvas::*;

pub fn get_tools() -> Vec<Box<dyn Tool>> {
    vec![
        // all tools
        Box::new(Brush),
        Box::new(Eraser {
            internal_brush: Brush,
        }),
        Box::new(Bucket),
        Box::new(ColorPicker),
    ]
}

fn rgb_array_to_color(rgb: &[f32; 4]) -> Color {
    Color::from_rgba(
        (rgb[0] * 255.).floor() as u8,
        (rgb[1] * 255.).floor() as u8,
        (rgb[2] * 255.).floor() as u8,
        (rgb[3] * 255.).floor() as u8,
    )
}
pub struct ToolsSettings {
    color_tolerance: u8,
    flood_mode_continuous: bool,
    brush_size: u16,
}

impl ToolsSettings {
    pub fn new() -> Self {
        ToolsSettings {
            color_tolerance: 0,
            flood_mode_continuous: true,
            brush_size: 1,
        }
    }
}

pub struct ToolContext<'a> {
    pub layer: &'a mut Layer,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub last_cursor_x: Option<i16>,
    pub last_cursor_y: Option<i16>,
    pub primary_color: &'a mut [f32; 4],
    pub secondary_color: &'a mut [f32; 4],
    pub settings: &'a mut ToolsSettings,
}

#[allow(unused)]
pub trait Tool {
    fn name(&self) -> String;
    fn update(&self, ctx: ToolContext) {}
    fn draw_buttons(&self, ui: &mut Ui, settings: &mut ToolsSettings) {}
    fn keybind(&self) -> Option<KeyCode> {
        None
    }
}
pub struct Brush;
impl Tool for Brush {
    fn name(&self) -> String {
        String::from("brush")
    }
    fn keybind(&self) -> Option<KeyCode> {
        Some(KeyCode::B)
    }
    fn draw_buttons(&self, ui: &mut Ui, settings: &mut ToolsSettings) {
        let brush_size_label = ui.label("brush size");
        let drag_value = DragValue::new(&mut settings.brush_size);
        ui.add(drag_value).labelled_by(brush_size_label.id);
    }
    fn update(&self, ctx: ToolContext) {
        // draw pixel if LMB is pressed

        let draw_color = if is_mouse_button_down(MouseButton::Left) {
            Some(rgb_array_to_color(&ctx.primary_color))
        } else if is_mouse_button_down(MouseButton::Right) {
            Some(rgb_array_to_color(&ctx.secondary_color))
        } else {
            None
        };
        if let Some(draw_color) = draw_color {
            if let Some(last_cursor_x) = ctx.last_cursor_x {
                if let Some(last_cursor_y) = ctx.last_cursor_y {
                    draw_line_image(
                        ctx.layer,
                        draw_color,
                        last_cursor_x,
                        last_cursor_y,
                        ctx.cursor_x,
                        ctx.cursor_y,
                        ctx.settings.brush_size,
                    );
                    ctx.layer.flush_texture();
                }
            } else {
                ctx.layer
                    .image
                    .set_pixel(ctx.cursor_x as u32, ctx.cursor_y as u32, draw_color);

                ctx.layer.flush_texture();
            }
        }
    }
}
pub struct Eraser {
    // the eraser actually just delegates all its tasks to an internal brush
    // but with color set to transparent
    internal_brush: Brush,
}
impl Tool for Eraser {
    fn name(&self) -> String {
        String::from("eraser")
    }
    fn keybind(&self) -> Option<KeyCode> {
        Some(KeyCode::E)
    }
    fn update(&self, ctx: ToolContext) {
        let mut color = [0., 0., 0., 0.];
        // hmm will have to make this less tedious
        let new_ctx = ToolContext {
            layer: ctx.layer,
            cursor_x: ctx.cursor_x,
            cursor_y: ctx.cursor_y,
            last_cursor_x: ctx.last_cursor_x,
            last_cursor_y: ctx.last_cursor_y,
            primary_color: &mut color.clone(),
            secondary_color: &mut color,
            settings: ctx.settings,
        };
        self.internal_brush.update(new_ctx);
    }
    fn draw_buttons(&self, ui: &mut Ui, settings: &mut ToolsSettings) {
        self.internal_brush.draw_buttons(ui, settings);
    }
}

pub struct ColorPicker;
impl Tool for ColorPicker {
    fn name(&self) -> String {
        String::from("color picker")
    }
    fn keybind(&self) -> Option<KeyCode> {
        Some(KeyCode::K)
    }
    fn update(&self, ctx: ToolContext) {
        let color_slot = if is_mouse_button_pressed(MouseButton::Left) {
            Some(ctx.primary_color)
        } else if is_mouse_button_pressed(MouseButton::Right) {
            Some(ctx.secondary_color)
        } else {
            None
        };
        if let Some(color_slot) = color_slot {
            let color = ctx
                .layer
                .image
                .get_pixel(ctx.cursor_x as u32, ctx.cursor_y as u32);
            let color = [color.r, color.g, color.b, color.a];
            *color_slot = color;
        }
    }
}

fn compare_colors(color_a: [u8; 4], color_b: [u8; 4]) -> u16 {
    let mut diffs = 0;
    for part in 0..4 {
        diffs += (color_a[part] as i16 - color_b[part] as i16).unsigned_abs()
    }
    diffs
}

fn flood_fill(
    width: usize,
    height: usize,
    pixels: &mut [[u8; 4]],
    x: usize,
    y: usize,
    target_color: [f32; 4],
    tolerance: u16,
) -> BoundsTracker {
    // convert target_color to u8
    let target_color: [u8; 4] = [
        (target_color[0] * 255.).floor() as u8,
        (target_color[1] * 255.).floor() as u8,
        (target_color[2] * 255.).floor() as u8,
        (target_color[3] * 255.).floor() as u8,
    ];

    // directions to check for pixels (up, down, right, left)
    let dirs = [[0, 1], [0, -1], [1, 0], [-1, 0]];

    let mut bounds_tracker = BoundsTracker::new();
    let row = vec![false; height];
    let mut visited: Vec<Vec<bool>> = vec![row.clone(); width];
    let mut buf: Vec<(usize, usize)> = vec![(x, y)];
    visited[x][y] = true;
    while let Some(item) = buf.pop() {
        let x = item.0;
        let y = item.1;
        let old_color = pixels[x + y * width];
        pixels[x + y * width] = target_color;
        bounds_tracker.track(x as u32, y as u32);
        for dir in dirs {
            let x = (x as isize + dir[0]).try_into();
            let y = (y as isize + dir[1]).try_into();
            let valid = x.is_ok() && y.is_ok() && x.unwrap() < width && y.unwrap() < height;
            if valid {
                let x: usize = x.unwrap();
                let y: usize = y.unwrap();
                let has_been_visited = visited[x][y];
                if !has_been_visited
                    && compare_colors(pixels[x + y * width], old_color) <= tolerance
                {
                    buf.push((x, y));
                    visited[x][y] = true;
                }
            }
        }
    }
    bounds_tracker
}

fn global_fill(
    width: usize,
    pixels: &mut [[u8; 4]],
    x: usize,
    y: usize,
    target_color: [f32; 4],
    tolerance: u16,
) -> BoundsTracker {
    // start color
    let start_color = pixels[x + y * width];

    // convert target_color to u8
    let target_color: [u8; 4] = [
        (target_color[0] * 255.).floor() as u8,
        (target_color[1] * 255.).floor() as u8,
        (target_color[2] * 255.).floor() as u8,
        (target_color[3] * 255.).floor() as u8,
    ];
    let mut bounds_tracker = BoundsTracker::new();
    for (index, pixel) in pixels.into_iter().enumerate() {
        if compare_colors(*pixel, start_color) <= tolerance {
            *pixel = target_color;
            bounds_tracker.track((index % width) as u32, (index / width) as u32);
        }
    }
    bounds_tracker
}
pub struct Bucket;
impl Tool for Bucket {
    fn name(&self) -> String {
        String::from("bucket")
    }
    fn keybind(&self) -> Option<KeyCode> {
        Some(KeyCode::F)
    }
    fn draw_buttons(&self, ui: &mut Ui, settings: &mut ToolsSettings) {
        ui.checkbox(&mut settings.flood_mode_continuous, "continuous");
        let tolerance_label = ui.label("tolerance");
        let slider = Slider::new(&mut settings.color_tolerance, RangeInclusive::new(0, 100));
        ui.add(slider)
            .labelled_by(tolerance_label.id)
            .on_hover_text(
                "color tolerance. 0 means bucket will only fill pixels that are exactly equal",
            );
    }
    fn update(&self, ctx: ToolContext) {
        let draw_color = if is_mouse_button_pressed(MouseButton::Left) {
            Some(ctx.primary_color)
        } else if is_mouse_button_pressed(MouseButton::Right) {
            Some(ctx.secondary_color)
        } else {
            None
        };
        if let Some(draw_color) = draw_color {
            let width = ctx.layer.width();
            let height = ctx.layer.height();
            let pixels: &mut [[u8; 4]] = ctx.layer.image.get_image_data_mut();

            // idek
            // convert the 0-100 scale tolerance to a 0-1020 scale tolerance (1020=255*4)
            // but not in a linear function
            // i hate this but idk how to actually do this sort of color comparison
            let tolerance = (1.04_f32.powf(ctx.settings.color_tolerance as f32)
                / (4. / ctx.settings.color_tolerance as f32)) as u16;

            let mut bounds = if ctx.settings.flood_mode_continuous {
                flood_fill(
                    width,
                    height,
                    pixels,
                    ctx.cursor_x as usize,
                    ctx.cursor_y as usize,
                    *draw_color,
                    tolerance,
                )
            } else {
                global_fill(
                    width,
                    pixels,
                    ctx.cursor_x as usize,
                    ctx.cursor_y as usize,
                    *draw_color,
                    tolerance,
                )
            };
            ctx.layer.force_update_region(bounds.flush());
        }
    }
}
