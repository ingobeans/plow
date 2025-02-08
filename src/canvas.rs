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
pub fn draw_line_image(
    image: &mut Image,
    color: Color,
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
) -> (Option<i16>, Option<i16>, Option<i16>, Option<i16>) {
    let mut min_x = None;
    let mut min_y = None;
    let mut max_x = None;
    let mut max_y = None;
    for (x, y) in Bresenham::new((x1, y1), (x2, y2)) {
        if x >= 0 && x < image.width() as i16 && y >= 0 && y < image.height() as i16 {
            if min_x.is_none() || min_x.unwrap() > x {
                min_x = Some(x);
            }
            if min_y.is_none() || min_y.unwrap() > y {
                min_y = Some(y);
            }
            if max_x.is_none() || max_x.unwrap() < x {
                max_x = Some(x);
            }
            if max_y.is_none() || max_y.unwrap() < y {
                max_y = Some(y);
            }
            image.set_pixel(x as u32, y as u32, color);
        }
    }
    (min_x, min_y, max_x, max_y)
}

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub image: Image,
    pub texture: Texture2D,
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
        }
    }
    pub fn update_texture(&mut self, region: Option<Rect>) {
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
