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
    panel_type: u32, // 0: Unknown, 1: Oled, 2: LcdIps, 3: LcdTn
    refresh_rate: u32, // Added refresh rate for temporal synchronization
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x_pixel = in.tex_coords.x * uniforms.width;
    var y_pixel = in.tex_coords.y * uniforms.height;

    // Panel-specific hacks (Anti-Burn-in / Response optimization)
    if (uniforms.panel_type == 1u) {
        // OLED Anti-Burn-in: Slow pixel shift
        let shift_x = sin(uniforms.time * 0.05) * 1.5;
        let shift_y = cos(uniforms.time * 0.05) * 1.5;
        x_pixel += shift_x;
        y_pixel += shift_y;
    } else if (uniforms.panel_type == 2u) {
        // LCD IPS: Gentle scroll to prevent liquid crystal "sticking"
        x_pixel += uniforms.time * 2.0;
    } else if (uniforms.panel_type == 3u) {
        // LCD TN: Vertical scroll
        y_pixel += uniforms.time * 5.0;
    }

    if (uniforms.mode == 0u) {
        // Black Layer
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (uniforms.mode == 1u) {
        // IMPROVED Louver (Vertical stripes with noise)
        // Add a micro-stagger to the louver to disrupt modern pixel arrays
        let stripe_width = 2.0; 
        let noise = sin(y_pixel * 0.5 + uniforms.time) * 0.5;
        if (u32(x_pixel + noise) % u32(stripe_width * 2.0) < u32(stripe_width)) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    } else if (uniforms.mode == 2u) {
        // IMPROVED Fast Vibration (Temporal Interference)
        // Instead of 60Hz (which causes blackout/flicker), use a slower interference frequency
        // matched to typical human flicker fusion threshold (~30-40Hz)
        let target_hz = 30.0;
        let toggle = f32(u32(uniforms.time * target_hz) % 2u);
        
        // Checkerboard pattern that flips
        let checker = u32(x_pixel + toggle) % 2u ^ u32(y_pixel) % 2u;
        if (checker == 0u) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    } else if (uniforms.mode == 3u) {
        // IMPROVED Asymmetric Curve
        let scale = 0.2;
        let val = sin(x_pixel * scale + uniforms.time * 0.5) * cos(y_pixel * scale - uniforms.time * 0.3);
        if (val > 0.0) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            discard;
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
