use line_drawing::Bresenham;
use macroquad::prelude::*;
use std::hash::Hash;

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
pub fn draw_line_image(layer: &mut Layer, color: Color, x1: i16, y1: i16, x2: i16, y2: i16) {
    for (x, y) in Bresenham::new((x1, y1), (x2, y2)) {
        if x >= 0 && x < layer.width() as i16 && y >= 0 && y < layer.height() as i16 {
            layer.set_pixel(x as u32, y as u32, color);
        }
    }
}

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub image: Image,
    pub texture: Texture2D,
    pub modified_min_x: Option<u32>,
    pub modified_min_y: Option<u32>,
    pub modified_max_x: Option<u32>,
    pub modified_max_y: Option<u32>,
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
            modified_min_x: None,
            modified_min_y: None,
            modified_max_x: None,
            modified_max_y: None,
        }
    }
    pub fn width(&self) -> usize {
        self.image.width()
    }
    pub fn height(&self) -> usize {
        self.image.height()
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if self.modified_min_x.is_none() || self.modified_min_x.unwrap() > x {
            self.modified_min_x = Some(x);
        }
        if self.modified_min_y.is_none() || self.modified_min_y.unwrap() > y {
            self.modified_min_y = Some(y);
        }
        if self.modified_max_x.is_none() || self.modified_max_x.unwrap() < x {
            self.modified_max_x = Some(x);
        }
        if self.modified_max_y.is_none() || self.modified_max_y.unwrap() < y {
            self.modified_max_y = Some(y);
        }
        self.image.set_pixel(x, y, color);
    }
    pub fn flush_texture(&mut self) {
        if self.modified_min_x.is_some() {
            let modified_min_x = self.modified_min_x.unwrap();
            let modified_min_y = self.modified_min_y.unwrap();
            let region_width = self.modified_max_x.unwrap() - modified_min_x + 1;
            let region_height = self.modified_max_y.unwrap() - modified_min_y + 1;
            self.force_update_region(Some(Rect {
                x: modified_min_x as f32,
                y: modified_min_y as f32,
                w: region_width as f32,
                h: region_height as f32,
            }));
        }

        self.modified_min_x = None;
        self.modified_min_y = None;
        self.modified_max_x = None;
        self.modified_max_y = None;
    }

    pub fn force_update_region(&mut self, region: Option<Rect>) {
        update_texture(&mut self.texture, &self.image, region);
    }
}

pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub current_layer: usize,
    pub layers: Vec<Layer>,
}

impl Canvas {
    pub fn from_image(image: Image) -> Result<Self, std::io::Error> {
        let width: u16 = image.width;
        let height: u16 = image.height;
        if !validate_canvas_size(width, height) {
            return Err(std::io::Error::other("canvas too big! no dimension may be greater than 32768, and the product of the width and height may not be greater than 1073676289"));
        }
        let layers = vec![Layer::new(image, String::from("background"))];
        Ok(Canvas {
            width,
            height,
            current_layer: 0,
            layers,
        })
    }
    pub fn new(width: u16, height: u16) -> Result<Self, std::io::Error> {
        let image = gen_empty_image(width, height);
        Canvas::from_image(image)
    }
    pub fn new_layer(&mut self) {
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

        let image = gen_empty_image(self.width, self.height);

        self.current_layer = self.current_layer.saturating_sub(1);
        self.layers
            .insert(self.current_layer, Layer::new(image, name));
    }
    pub fn delete_layer(&mut self) {
        if self.layers.len() > 1 {
            self.layers.remove(self.current_layer);
            self.current_layer = self.current_layer.saturating_sub(1);
        }
    }
}
