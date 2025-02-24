use image::{write_buffer_with_format, ExtendedColorType};
use line_drawing::Bresenham;
use macroquad::prelude::*;
use std::{hash::Hash, io::Cursor};

use crate::{consts::MIN_ZOOM, tools::Stroke};

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

fn texture_from(image: &Image) -> Texture2D {
    let texture = Texture2D::from_image(image);
    texture.set_filter(FilterMode::Nearest);
    texture
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
        *texture = texture_from(image);
    }
}

pub fn draw_line_image(
    layer: &mut Layer,
    color: Color,
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    stroke: &Stroke,
) {
    for (x, y) in Bresenham::new((x1, y1), (x2, y2)) {
        for (stroke_x, row) in stroke.pixels.iter().enumerate() {
            for (stroke_y, value) in row.iter().enumerate() {
                if !value {
                    continue;
                }
                let x = (x + stroke_x as i16 + stroke.pixels_offset).try_into();
                let y = (y + stroke_y as i16 + stroke.pixels_offset).try_into();
                if x.is_ok() && y.is_ok() {
                    let x = x.unwrap();
                    let y = y.unwrap();
                    if x < layer.width() as u32 && y < layer.height() as u32 {
                        layer.set_pixel(x, y, color);
                    }
                }
            }
        }
    }
}

/// Keeps track of the largest and smallest coordinates given to it by track(). Can be flushed to generate a Rect for its area, and wipe coordinate data.
pub struct BoundsTracker {
    empty: bool,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}
impl BoundsTracker {
    pub fn new() -> Self {
        BoundsTracker {
            empty: true,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        }
    }
    pub fn track(&mut self, x: u32, y: u32) {
        if self.min_x > x || self.empty {
            self.min_x = x;
        }
        if self.min_y > y || self.empty {
            self.min_y = y;
        }
        if self.max_x < x || self.empty {
            self.max_x = x;
        }
        if self.max_y < y || self.empty {
            self.max_y = y;
        }
        self.empty = false;
    }
    pub fn flush(&mut self) -> Option<Rect> {
        if self.empty {
            return None;
        }
        self.empty = true;
        Some(Rect {
            x: self.min_x as f32,
            y: self.min_y as f32,
            w: (self.max_x - self.min_x + 1) as f32,
            h: (self.max_y - self.min_y + 1) as f32,
        })
    }
}

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub image: Image,
    pub texture: Texture2D,
    pub bounds_tracker: BoundsTracker,
}

impl Hash for Layer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Layer {
    pub fn new(image: Image, name: String) -> Self {
        let texture = texture_from(&image);
        Layer {
            image,
            name,
            visible: true,
            texture,
            bounds_tracker: BoundsTracker::new(),
        }
    }
    pub fn width(&self) -> usize {
        self.image.width()
    }
    pub fn height(&self) -> usize {
        self.image.height()
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.bounds_tracker.track(x, y);
        self.image.set_pixel(x, y, color);
    }
    pub fn flush_texture(&mut self) {
        let bounds = self.bounds_tracker.flush();
        self.force_update_region(bounds);
    }

    pub fn force_update_region(&mut self, region: Option<Rect>) {
        //if let Some(region) = region {
        //    for x in region.x as u32..(region.x + region.w) as u32 {
        //        for y in region.y as u32..(region.y + region.h) as u32 {
        //            self.image.set_pixel(x, y, RED);
        //        }
        //    }
        //}
        update_texture(&mut self.texture, &self.image, region);
    }
}

pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub current_layer: usize,
    pub layers: Vec<Layer>,
    pub name: String,
    pub camera_grid_size: f32,
    pub camera_x: f32,
    pub camera_y: f32,
}

fn export_png(image: Image, name: String) {
    // buffer to store png image data in
    let mut buffered_writer = Cursor::new(Vec::new());

    // convert image to png and write to the buffer
    write_buffer_with_format(
        &mut buffered_writer,
        &image.bytes,
        image.width as u32,
        image.height as u32,
        ExtendedColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("Couldn't convert canvas to PNG buffer.");

    // download the buffer data with quad-file-download
    let _ = quad_files::download(&(name + ".png"), &buffered_writer.into_inner(), Some("PNG"));
}

impl Canvas {
    pub fn from_image(image: Image, name: String) -> Result<Self, std::io::Error> {
        let width: u16 = image.width;
        let height: u16 = image.height;
        if !validate_canvas_size(width, height) {
            return Err(std::io::Error::other("canvas too big! no dimension may be greater than 32768, and the product of the width and height may not be greater than 1073676289"));
        }
        let layers = vec![Layer::new(image, String::from("background"))];
        let (camera_grid_size, camera_x, camera_y) =
            Self::generate_camera_bounds_to_fit(width, height);

        Ok(Canvas {
            width,
            height,
            current_layer: 0,
            layers,
            name,
            camera_grid_size,
            camera_x,
            camera_y,
        })
    }
    fn generate_camera_bounds_to_fit(canvas_width: u16, canvas_height: u16) -> (f32, f32, f32) {
        // make zoom to show entire canvas height
        let camera_grid_size: f32 = (screen_width() / canvas_height as f32 / 2.0).max(MIN_ZOOM);
        // make camera position default be at center of canvas
        let camera_x = canvas_width as f32 / 2. * camera_grid_size - screen_width() / 2.;
        let camera_y = canvas_height as f32 / 2. * camera_grid_size - screen_height() / 2.;
        (camera_grid_size, camera_x, camera_y)
    }
    pub fn new(width: u16, height: u16, name: String) -> Result<Self, std::io::Error> {
        let image = gen_empty_image(width, height);
        Canvas::from_image(image, name)
    }
    pub fn to_image(&self) -> Image {
        let mut image = self.layers.last().unwrap().image.clone();
        for layer in self.layers.iter().rev().skip(1) {
            image.overlay(&layer.image);
        }
        image
    }
    pub fn export(&self) {
        let image = self.to_image();
        export_png(image, self.name.clone());
    }
    fn get_new_layer_name(&self) -> String {
        // get a name for the new layer (that isnt already used!!!!!)
        let mut layer_name_index = self.layers.len() + 1;
        let mut name = format!("layer {}", layer_name_index);
        let names = self
            .layers
            .iter()
            .map(|f| &f.name)
            .collect::<Vec<&String>>();
        while names.contains(&&name) {
            layer_name_index += 1;
            name = format!("layer {}", layer_name_index);
        }
        name
    }
    pub fn new_layer(&mut self) {
        let name = self.get_new_layer_name();

        let image = gen_empty_image(self.width, self.height);

        self.layers
            .insert(self.current_layer, Layer::new(image, name));
    }
    pub fn merge_layers_down(&mut self) {
        if self.current_layer != self.layers.len() - 1 {
            let old_layer = self.layers.remove(self.current_layer);
            self.layers[self.current_layer]
                .image
                .overlay(&old_layer.image);
            self.layers[self.current_layer].force_update_region(None);
        }
    }
    pub fn duplicate_layer(&mut self) {
        let name = self.get_new_layer_name();
        let image = self.layers[self.current_layer].image.clone();
        let new = Layer::new(image, name);
        self.layers.insert(self.current_layer, new);
    }
    pub fn delete_layer(&mut self) {
        if self.layers.len() > 1 {
            self.layers.remove(self.current_layer);
            if self.current_layer == self.layers.len() {
                self.current_layer = self.current_layer.saturating_sub(1);
            }
        }
    }
}
