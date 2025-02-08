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

pub struct ToolContext<'a> {
    pub layer: &'a mut Layer,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub last_cursor_x: Option<i16>,
    pub last_cursor_y: Option<i16>,
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
}

#[allow(unused)]
pub trait Tool {
    fn name(&self) -> String;
    fn draw(&self, ctx: ToolContext) {}
}
pub struct Brush;
impl Tool for Brush {
    fn name(&self) -> String {
        String::from("brush")
    }
    fn draw(&self, ctx: ToolContext) {
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
                    let (region_x, region_y, region_max_x, region_max_y) = draw_line_image(
                        &mut ctx.layer.image,
                        draw_color,
                        last_cursor_x,
                        last_cursor_y,
                        ctx.cursor_x,
                        ctx.cursor_y,
                    );
                    if region_x.is_some() {
                        let region_x = region_x.unwrap();
                        let region_y = region_y.unwrap();
                        let region_width = region_max_x.unwrap() - region_x + 1;
                        let region_height = region_max_y.unwrap() - region_y + 1;
                        ctx.layer.update_texture(Some(Rect {
                            x: region_x as f32,
                            y: region_y as f32,
                            w: region_width as f32,
                            h: region_height as f32,
                        }));
                    }
                }
            } else {
                ctx.layer
                    .image
                    .set_pixel(ctx.cursor_x as u32, ctx.cursor_y as u32, draw_color);

                ctx.layer.update_texture(Some(Rect {
                    x: ctx.cursor_x as f32,
                    y: ctx.cursor_y as f32,
                    w: 1.,
                    h: 1.,
                }));
            }
        }
    }
}
pub struct Bucket;
impl Tool for Bucket {
    fn name(&self) -> String {
        String::from("bucket")
    }
}
