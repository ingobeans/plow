use std::ops::RangeInclusive;

use egui_macroquad::egui::{Slider, Ui};
use macroquad::prelude::*;

use crate::canvas::*;

pub fn get_tools() -> Vec<Box<dyn Tool>> {
    vec![
        // all tools
        Box::new(Brush),
        Box::new(Bucket),
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
}

impl ToolsSettings {
    pub fn new() -> Self {
        ToolsSettings { color_tolerance: 0 }
    }
}

pub struct ToolContext<'a> {
    pub layer: &'a mut Layer,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub last_cursor_x: Option<i16>,
    pub last_cursor_y: Option<i16>,
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
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
fn compare_colors(color_a: [u8; 4], color_b: [u8; 4]) -> u16 {
    let mut diffs = 0;
    for part in 0..4 {
        diffs += (color_a[part] as i16 - color_b[part] as i16).abs() as u16
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
) {
    // convert target_color to u8
    let target_color: [u8; 4] = [
        (target_color[0] * 255.).floor() as u8,
        (target_color[1] * 255.).floor() as u8,
        (target_color[2] * 255.).floor() as u8,
        (target_color[3] * 255.).floor() as u8,
    ];

    // if all pixels are same color, replace them all
    // makes function much faster when filling a whole background
    if pixels.windows(2).all(|win| win[0] == win[1]) {
        pixels.fill(target_color);
        return;
    }

    // directions to check for pixels (up, down, right, left)
    let dirs = [[0, 1], [0, -1], [1, 0], [-1, 0]];

    let row = vec![false; height];
    let mut visited: Vec<Vec<bool>> = vec![row.clone(); width];
    let mut buf: Vec<(usize, usize)> = vec![(x, y)];
    visited[x][y] = true;
    while !buf.is_empty() {
        let item = buf.pop().unwrap();
        let x = item.0;
        let y = item.1;
        let old_color = pixels[x + y * width].clone();
        pixels[x + y * width] = target_color;
        for dir in dirs {
            let x = (x as isize + dir[0]).try_into();
            let y = (y as isize + dir[1]).try_into();
            let valid = x.is_ok() && y.is_ok() && x.unwrap() < width && y.unwrap() < height;
            if valid {
                let x: usize = x.unwrap();
                let y: usize = y.unwrap();
                let has_been_visited = visited[x][y];
                if !has_been_visited {
                    if compare_colors(pixels[x + y * width], old_color) <= tolerance {
                        buf.push((x, y));
                        visited[x][y] = true;
                    }
                }
            }
        }
    }
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
        let label = ui.label("tolerance");
        let widget = Slider::new(&mut settings.color_tolerance, RangeInclusive::new(0, 100));
        ui.add(widget).labelled_by(label.id).on_hover_text(
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

            flood_fill(
                width,
                height,
                pixels,
                ctx.cursor_x as usize,
                ctx.cursor_y as usize,
                draw_color,
                tolerance,
            );
            ctx.layer.force_update_region(None);
        }
    }
}
