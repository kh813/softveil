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
    v0: vec4<f32>, // time, mode, alpha, width
    v1: vec4<f32>, // height, panel_type, refresh_rate, intensity
    v2: vec4<f32>, // bidirectional, period_px, scroll_speed_px, cover_ratio
    v3: vec4<f32>, // phase_flip_hz, grid_period_px, luminance_compress, hatch_angle
    v4: vec4<f32>, // padding
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Full-screen triangle trick (covers [-1, 1] range)
    // Idx 0: (-1, -1), Idx 1: (3, -1), Idx 2: (-1, 3)
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    let pos = uv * 2.0 - 1.0;
    
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

fn stable_noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn calc_louver(scrolled: f32, stripe_width: f32, edge_px: f32, base_alpha: f32) -> f32 {
    let s = scrolled;
    if (s < stripe_width) {
        let dist_from_edge = min(stripe_width - s, s);
        return base_alpha * select(1.0, dist_from_edge / edge_px, dist_from_edge < edge_px);
    } else {
        return base_alpha * 0.15;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Unpack uniforms
    let t = uniforms.v0.x;
    let mode = u32(uniforms.v0.y);
    let alpha = uniforms.v0.z;
    let width = uniforms.v0.w;
    
    let height = uniforms.v1.x;
    let panel_type = u32(uniforms.v1.y);
    let refresh_rate = u32(uniforms.v1.z);
    let inten = uniforms.v1.w;
    
    let bidirectional = u32(uniforms.v2.x);
    let period_px = uniforms.v2.y;
    let scroll_speed_px = uniforms.v2.z;
    let cover_ratio = uniforms.v2.w;
    
    let phase_flip_hz = uniforms.v3.x;
    let grid_period_px = uniforms.v3.y;
    let luminance_compress = uniforms.v3.z;
    let hatch_angle = uniforms.v3.w;

    let x = in.tex_coords.x * width;
    let y = in.tex_coords.y * height;

    if (mode == 0u) {
        // BlackLayer: Uniform coverage
        return vec4<f32>(0.0, 0.0, 0.0, alpha);

    } else if (mode == 1u) {
        // VerticalLouver (Cross-Louver with Diagonal Protection)
        let period = max(period_px * inten, 1.0);
        let stripe_width = period * clamp(cover_ratio, 0.3, 0.9);
        let edge_px: f32 = max(1.5, period * 0.08);
        let scroll_speed = scroll_speed_px;
        let burn_in_offset = select(0.0, t * 0.2, panel_type == 1u);

        // 1. 水平縞 (上下方向)
        let scrolled_y = ((y + t * scroll_speed) % period + period) % period;
        let alpha_h = calc_louver(scrolled_y, stripe_width, edge_px, alpha);

        // 2. 垂直縞 (左右方向)
        let scrolled_x = ((x + t * scroll_speed * 0.7 + burn_in_offset) % period + period) % period;
        let alpha_v = calc_louver(scrolled_x, stripe_width, edge_px, alpha);

        // 3. 斜め成分
        let cos_a = cos(hatch_angle);
        let sin_a = sin(hatch_angle);
        let rotated = (x * cos_a + y * sin_a);
        let scrolled_d = ((rotated + t * scroll_speed * 0.5) % period + period) % period;
        let alpha_d = calc_louver(scrolled_d, stripe_width, edge_px, alpha);

        let alpha_out = max(max(alpha_h, alpha_v), alpha_d);
        return vec4<f32>(0.0, 0.0, 0.0, alpha_out);

    } else if (mode == 2u) {
        // ── AIOcrInterference (Phase 5: Subpixel UHD Jamming Prototype) ──
        let p = max(period_px * 0.4 * max(inten, 0.2), 1.0);
        let row = floor(y / p);
        let x_off = select(0.0, p * 0.5, row % 2.0 == 0.0);
        let x_p = ((x + x_off + t * scroll_speed_px * 0.1) % p + p) % p;
        let y_p = (y % p + p) % p;
        
        let is_aperture = (x_p < p * 0.45) && (y_p < p * 0.45);
        var alpha_main = select(alpha, 0.0, is_aperture);

        let n_coord = floor(vec2<f32>(x, y) * 2.0); 
        let n = stable_noise(n_coord + floor(t * 24.0));
        let alpha_noise = select(0.0, alpha * 0.4, n > 0.85);
        
        let final_alpha = clamp(max(alpha_main, alpha_noise), 0.0, 1.0);
        return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
    } else if (mode == 3u) {
        // ── HighIntensitySPD (Adaptive Narrow-Pixel Strategy) ─────────
        if (panel_type == 1u) {
            // OLED V5 Subpixel Kinetic Void
            let p = max(period_px * 0.2, 2.0);
            let phase = floor(t * 60.0) % 4.0 * (p * 0.25);
            let x_p = ((x + phase) % p + p) % p;
            let y_p = ((y + phase * 0.7) % p + p) % p;
            
            let is_slit = (x_p < p * 0.1) || (y_p < p * 0.1);
            let alpha_base = alpha * 0.95;
            let alpha_slit = alpha * 0.1;
            var alpha_main = select(alpha_base, alpha_slit, is_slit);

            let sub_n = stable_noise(vec2<f32>(x * 3.0, y) + t);
            let final_alpha = clamp(alpha_main + select(0.0, 0.2, sub_n > 0.9), 0.0, 1.0);
            return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
        } else {
            // LCD V7 Luminous Crystal (Subpixel Selective)
            let p = max(period_px * 0.12, 1.5);
            let slow_offset = t * 0.2;
            
            let x_p = ((x + slow_offset) % p + p) % p;
            let is_slit = x_p < (p * 0.15 * inten);
            let alpha_main = select(alpha * 0.92, 0.0, is_slit);

            let bx = i32(x) % 4;
            let by = i32(y) % 4;
            let b_idx = ((by * 4 + bx) % 16 + 16) % 16;
            var b_v: f32;
            switch (b_idx) {
                case 0: { b_v = 0.0; } case 1: { b_v = 8.0; } case 2: { b_v = 2.0; } case 3: { b_v = 10.0; }
                case 4: { b_v = 12.0; } case 5: { b_v = 4.0; } case 6: { b_v = 14.0; } case 7: { b_v = 6.0; }
                case 8: { b_v = 3.0; } case 9: { b_v = 11.0; } case 10: { b_v = 1.0; } case 11: { b_v = 9.0; }
                case 12: { b_v = 15.0; } case 13: { b_v = 7.0; } case 14: { b_v = 13.0; } case 15: { b_v = 5.0; }
                default: { b_v = 0.0; }
            }
            let b_val = b_v / 16.0;
            let mesh_alpha = select(0.0, alpha * 0.25, b_val > 0.75);

            let final_alpha = clamp(alpha_main + mesh_alpha, 0.0, 1.0);
            let haze = select(0.0, 0.05 * alpha, b_val > 0.95);

            return vec4<f32>(haze, haze, haze, final_alpha);
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}
