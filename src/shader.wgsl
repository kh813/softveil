struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Full-screen triangle trick
    let x = f32(i32(in_vertex_index) << 1 & 2) - 1.0;
    let y = f32(i32(in_vertex_index) & 2) - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

struct Uniforms {
    time: f32,
    mode: u32, // 0: Black, 1: Louver, 2: HighSpeed, 3: Asymmetric
    alpha: f32,
    width: f32,
    height: f32,
    is_oled: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x_pixel = in.tex_coords.x * uniforms.width;
    var y_pixel = in.tex_coords.y * uniforms.height;

    // OLED Anti-Burn-in: Shift the pattern slightly over time
    if (uniforms.is_oled == 1u) {
        let shift_x = sin(uniforms.time * 0.1) * 2.0;
        let shift_y = cos(uniforms.time * 0.1) * 2.0;
        x_pixel += shift_x;
        y_pixel += shift_y;
    }

    if (uniforms.mode == 0u) {
        // Black Layer
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (uniforms.mode == 1u) {
        // Louver (Vertical stripes)
        if (u32(x_pixel) % 2u == 0u) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    } else if (uniforms.mode == 2u) {
        // High Speed Motion Masking (Vibrating pattern)
        // Shift by 1px every other frame (or based on time)
        let offset = f32(u32(uniforms.time * 60.0) % 2u);
        if (u32(x_pixel + offset) % 2u == 0u) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    } else if (uniforms.mode == 3u) {
        // Asymmetric Curve (Conceptual implementation)
        // A complex pattern that makes it hard to read from angles
        let scale = 0.1;
        let val = sin(x_pixel * scale) + cos(y_pixel * scale + uniforms.time);
        if (val > 0.0) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
