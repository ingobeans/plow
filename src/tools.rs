use std::ops::RangeInclusive;

use egui_macroquad::egui::{DragValue, Slider, Ui};
use macroquad::prelude::*;

use crate::{canvas::*, consts::DIRECTIONS};

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

pub enum CursorType {
    Stroke,
    Point,
}

pub struct Stroke {
    pub size: u16,
    pub pixels: Vec<Vec<bool>>,
    pub pixels_offset: i16,
    pub borders: Vec<((usize, usize), (usize, usize))>,
}
impl Stroke {
    pub fn new(size: u16) -> Self {
        let pixels = Self::generate_pixels(size);
        let borders = Self::generate_borders(&pixels);
        Stroke {
            size,
            pixels,
            pixels_offset: 0,
            borders,
        }
    }
    pub fn update(&mut self) {
        self.pixels = Self::generate_pixels(self.size);
        self.borders = Self::generate_borders(&self.pixels);
        self.pixels_offset = -(self.size as i16) / 2;
    }
    fn generate_pixels(size: u16) -> Vec<Vec<bool>> {
        if size > 1 {
            let half_brush_size = size as f32 / 2.;
            let brush_size = size as i32;
            let mut new = Vec::new();
            for x in -brush_size / 2..brush_size / 2 + 1 {
                new.push(Vec::new());
                for y in -brush_size / 2..brush_size / 2 + 1 {
                    if ((x * x + y * y) as f32) < half_brush_size * half_brush_size + 0.5 {
                        new.last_mut().unwrap().push(true);
                    } else {
                        new.last_mut().unwrap().push(false);
                    }
                }
            }
            new
        } else {
            vec![vec![true]]
        }
    }
    fn generate_borders(pixels: &[Vec<bool>]) -> Vec<((usize, usize), (usize, usize))> {
        let mut new = Vec::new();
        for (x, column) in pixels.iter().enumerate() {
            for (y, value) in column.iter().enumerate() {
                if !value {
                    continue;
                }
                let mut neighbour_above = false;
                let mut neighbour_below = false;
                let mut neighbour_right = false;
                let mut neighbour_left = false;
                let mut neighbour_bools = [
                    &mut neighbour_above,
                    &mut neighbour_below,
                    &mut neighbour_right,
                    &mut neighbour_left,
                ];
                for (dir, bool) in DIRECTIONS.iter().zip(neighbour_bools.iter_mut()) {
                    let Ok(x) = (x as isize + dir[0]).try_into() else {
                        continue;
                    };
                    let Ok(y) = (y as isize + dir[1]).try_into() else {
                        continue;
                    };
                    if let Some(column) = pixels.get::<usize>(x) {
                        if let Some(value) = column.get::<usize>(y) {
                            if *value {
                                **bool = true;
                            }
                        }
                    }
                }
                if !neighbour_above {
                    new.push(((x, y + 1), (x + 1, y + 1)));
                }
                if !neighbour_below {
                    new.push(((x, y), (x + 1, y)));
                }
                if !neighbour_left {
                    new.push(((x, y), (x, y + 1)));
                }
                if !neighbour_right {
                    new.push(((x + 1, y), (x + 1, y + 1)));
                }
            }
        }
        new
    }
}

pub struct ToolsSettings {
    pub color_tolerance: u8,
    pub flood_mode_continuous: bool,
    pub stroke: Stroke,
}

impl ToolsSettings {
    pub fn new() -> Self {
        ToolsSettings {
            color_tolerance: 0,
            flood_mode_continuous: true,
            stroke: Stroke::new(1),
        }
    }
}

pub struct ToolContext<'a> {
    pub layer: &'a mut Layer,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub cursor_in_bounds: bool,
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
    fn cursor_type(&self) -> CursorType {
        CursorType::Point
    }
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
    fn cursor_type(&self) -> CursorType {
        CursorType::Stroke
    }
    fn draw_buttons(&self, ui: &mut Ui, settings: &mut ToolsSettings) {
        let brush_size_label = ui.label("brush size");
        let drag_value = DragValue::new(&mut settings.stroke.size)
            .update_while_editing(false)
            .range(RangeInclusive::new(1, i32::MAX));
        let resp = ui.add(drag_value).labelled_by(brush_size_label.id);
        if resp.drag_stopped() || resp.lost_focus() {
            settings.stroke.update();
        }
    }
    fn update(&self, ctx: ToolContext) {
        // draw pixel if LMB is pressed

        let draw_color = if is_mouse_button_down(MouseButton::Left) {
            Some(rgb_array_to_color(ctx.primary_color))
        } else if is_mouse_button_down(MouseButton::Right) {
            Some(rgb_array_to_color(ctx.secondary_color))
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
                        &ctx.settings.stroke,
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
    fn cursor_type(&self) -> CursorType {
        CursorType::Stroke
    }
    fn update(&self, ctx: ToolContext) {
        let mut color = [0., 0., 0., 0.];
        // hmm will have to make this less tedious
        let new_ctx = ToolContext {
            layer: ctx.layer,
            cursor_x: ctx.cursor_x,
            cursor_y: ctx.cursor_y,
            cursor_in_bounds: ctx.cursor_in_bounds,
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
        // return early if not cursor in bounds
        if !ctx.cursor_in_bounds {
            return;
        }
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

    let mut bounds_tracker = BoundsTracker::new();
    let mut visited = vec![false; width * height];
    let mut buf: Vec<(usize, usize)> = vec![(x, y)];
    visited[x + y * width] = true;
    while let Some(item) = buf.pop() {
        let x = item.0;
        let y = item.1;
        let old_color = pixels[x + y * width];
        pixels[x + y * width] = target_color;
        bounds_tracker.track(x as u32, y as u32);
        for dir in DIRECTIONS {
            let x = (x as isize + dir[0]).try_into();
            let y = (y as isize + dir[1]).try_into();
            let valid = x.is_ok() && y.is_ok() && x.unwrap() < width && y.unwrap() < height;
            if valid {
                let x: usize = x.unwrap();
                let y: usize = y.unwrap();
                let has_been_visited = visited[x + y * width];
                if !has_been_visited
                    && compare_colors(pixels[x + y * width], old_color) <= tolerance
                {
                    buf.push((x, y));
                    visited[x + y * width] = true;
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
    for (index, pixel) in pixels.iter_mut().enumerate() {
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
        // return early if mouse not in canvas
        //ctx.cursor_x < 0 || ctx.cursor_y < 0 || ctx.cursor_x >= ctx.layer.width() || ctx.cursor_y >= ctx.layer.height()
        if !ctx.cursor_in_bounds {
            return;
        }
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
            let pixels: &mut [[u8; 4]] = ctx.layer.get_image_data_mut();

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
