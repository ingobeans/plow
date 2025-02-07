use macroquad::prelude::*;
pub const BG_COLOR: Color = color_u8!(14, 14, 14, 255);
pub const BORDER_WIDTH: f32 = 2.;
pub const SCROLL_AMT: f32 = 1.1;
pub const MIN_ZOOM: f32 = 0.001;

const DEFAULT_VERTEX: &str = r#"#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const GRID_FRAGMENT: &str = r#"#version 100
precision mediump float;

void main() {
    vec2 pos = gl_FragCoord.xy;
    float grid_size = 10.0;
    vec3 color = vec3(0.);
    color = vec3(0.13,0.13,0.13);
    float offset = 0.0;

    // offset every other line (so we get grid and not stripes)
    if (mod(pos.y/grid_size,2.0) < 1.0) {
        offset = 1.0;
    }
    // change color of every other pixel
    if (mod(pos.x/grid_size+offset,2.0) < 1.0) {
        color = vec3(0.25,0.25,0.25);
    }
    gl_FragColor = vec4(color,1.0);
}"#;

pub fn get_grid_material() -> Material {
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX,
            fragment: GRID_FRAGMENT,
        },
        MaterialParams::default(),
    )
    .unwrap()
}
