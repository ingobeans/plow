use line_drawing::Bresenham;
use macroquad::prelude::*;
use std::{hash::Hash, io::Cursor, path::PathBuf};
use strum::IntoStaticStr;

use crate::{consts::MIN_ZOOM, tools::Stroke};

pub fn gen_empty_image(width: u16, height: u16) -> Image {
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
    pub modified: bool,
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
            modified: false,
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
        self.modified = true;
    }
    pub fn flush_texture(&mut self) {
        let bounds = self.bounds_tracker.flush();
        self.force_update_region(bounds);
    }
    pub fn get_image_data_mut(&mut self) -> &mut [[u8; 4]] {
        self.modified = true;
        self.image.get_image_data_mut()
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

/// Action to undo a specific type of change
#[derive(IntoStaticStr)]
pub enum UndoAction {
    /// For changes of entire layer, track entire layer image data (index, image)
    LayerFull(usize, Image),
    /// For changes in a region of layer, track the data of that region (index, region, subimage)
    LayerRegion(usize, Rect, Image),
    /// When a layer is created, track its index to know what to remove to undo it
    CreateLayer(usize),
    /// When a layer is deleted, track where to insert it, its name and its data to undo it (index, name)
    DeleteLayer(usize, String, Image),
    /// When layers are merged down, track index of the destination layer, its name, and both layer's old data (index, source image, source layer name, dest image)
    MergeLayersDown(usize, Image, String, Image),
    /// When layer is renamed, track its index and old name
    RenameLayer(usize, String),
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
    pub preffered_file_format: ImageFormat,
    pub save_path: Option<PathBuf>,
    pub undo_history: Vec<UndoAction>,
    modified: bool,
}

impl Canvas {
    pub fn from_image(
        image: Image,
        name: String,
        preffered_file_format: ImageFormat,
    ) -> Result<Self, std::io::Error> {
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
            preffered_file_format,
            save_path: None,
            undo_history: Vec::new(),
            modified: false,
        })
    }
    pub fn is_modified(&self) -> bool {
        let mut modified = self.modified;
        for layer in &self.layers {
            modified |= layer.modified;
        }
        modified
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
        Canvas::from_image(image, name, ImageFormat::Png)
    }
    pub fn to_image(&self) -> Image {
        let mut image = self.layers.last().unwrap().image.clone();
        for layer in self.layers.iter().rev().skip(1) {
            image.overlay(&layer.image);
        }
        image
    }
    pub fn export(&mut self, overwrite_old_if_possible: bool) {
        // mark all layers as unmodified
        for layer in self.layers.iter_mut() {
            layer.modified = false;
        }
        self.modified = false;

        let image = self.to_image();

        // buffer to store png image data in
        let mut buffered_writer = Cursor::new(Vec::new());

        // convert image to png and write to the buffer
        image::write_buffer_with_format(
            &mut buffered_writer,
            &image.bytes,
            image.width as u32,
            image.height as u32,
            image::ColorType::Rgba8,
            self.preffered_file_format,
        )
        .expect("Couldn't convert canvas to bytes buffer.");

        let file_ext = self.preffered_file_format.extensions_str()[0];

        // if on standalone, and file has already been saved before to a known path, and `overwrite_old_if_possible` is true, then directly overwrite old path
        #[cfg(not(target_arch = "wasm32"))]
        {
            if overwrite_old_if_possible {
                if let Some(path) = &self.save_path {
                    let _ = std::fs::write(path, buffered_writer.into_inner());
                    return;
                }
            }
        }

        // download the buffer data with quad-file-download
        let result = quad_files::download(
            &(self.name.clone() + "." + file_ext),
            &buffered_writer.into_inner(),
            Some(""),
        );

        // keep track where file was saved (only for standalone)
        if let Ok(Some(location)) = result {
            if let Some(file_name) = location.file_stem() {
                self.name = file_name.to_string_lossy().to_string();
            }
            self.save_path = Some(location);
        }
    }
    pub fn rename_layer(&mut self, new_name: String) {
        let layer = &mut self.layers[self.current_layer];

        // replace layer.name with new_name, and assign the old value of layer.name to old_name
        let old_name = std::mem::replace(&mut layer.name, new_name);

        self.undo_history
            .push(UndoAction::RenameLayer(self.current_layer, old_name));
    }
    pub fn undo(&mut self) {
        let action = self.undo_history.pop();
        if let Some(action) = action {
            match action {
                UndoAction::CreateLayer(index) => {
                    self.layers.remove(index);
                    self.current_layer = index;
                }
                UndoAction::MergeLayersDown(index, source, source_name, dest) => {
                    let new = Layer::new(source, source_name);
                    self.layers[index].image = dest;
                    self.layers[index].force_update_region(None);
                    self.layers.insert(index, new);
                }
                UndoAction::DeleteLayer(index, name, image) => {
                    let new = Layer::new(image, name);
                    self.layers.insert(index, new);
                }
                UndoAction::RenameLayer(index, name) => {
                    self.layers[index].name = name;
                }
                UndoAction::LayerFull(index, data) => {
                    self.layers[index].image = data;
                    self.layers[index].force_update_region(None);
                }
                _ => {
                    unimplemented!();
                }
            }
        }
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
        self.modified = true;

        self.undo_history
            .push(UndoAction::CreateLayer(self.current_layer));

        let name = self.get_new_layer_name();
        let image = gen_empty_image(self.width, self.height);

        self.layers
            .insert(self.current_layer, Layer::new(image, name));
    }
    pub fn merge_layers_down(&mut self) {
        self.modified = true;

        if self.current_layer != self.layers.len() - 1 {
            // add to history
            self.undo_history.push(UndoAction::MergeLayersDown(
                self.current_layer,
                self.layers[self.current_layer].image.clone(),
                self.layers[self.current_layer].name.clone(),
                self.layers[self.current_layer + 1].image.clone(),
            ));
            // merge down
            let old_layer = self.layers.remove(self.current_layer);
            self.layers[self.current_layer]
                .image
                .overlay(&old_layer.image);
            self.layers[self.current_layer].force_update_region(None);
            self.layers[self.current_layer].modified =
                self.layers[self.current_layer].modified || old_layer.modified;
        }
    }
    pub fn duplicate_layer(&mut self) {
        self.modified = true;

        self.undo_history
            .push(UndoAction::CreateLayer(self.current_layer));

        let name = self.get_new_layer_name();
        let source = &self.layers[self.current_layer];
        let image = source.image.clone();
        let mut new = Layer::new(image, name);
        new.modified = source.modified;
        self.layers.insert(self.current_layer, new);
    }
    pub fn delete_layer(&mut self) {
        self.modified = true;

        if self.layers.len() > 1 {
            // add to history
            self.undo_history.push(UndoAction::DeleteLayer(
                self.current_layer,
                self.layers[self.current_layer].name.clone(),
                self.layers[self.current_layer].image.clone(),
            ));
            // remove layer
            self.layers.remove(self.current_layer);
            if self.current_layer == self.layers.len() {
                self.current_layer = self.current_layer.saturating_sub(1);
            }
        }
    }
}
