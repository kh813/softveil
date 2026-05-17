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
    mode: u32,
    alpha: f32,
    width: f32,
    height: f32,
    panel_type: u32,
    refresh_rate: u32,
    intensity: f32,
    bidirectional: u32,
    period_px: f32,
    scroll_speed_px: f32,
    cover_ratio: f32,
    phase_flip_hz: f32,
    grid_period_px: f32,
    luminance_compress: f32,
    hatch_angle: f32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32, // Ensure 16-byte alignment (80 bytes total)
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

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
    let x = in.tex_coords.x * uniforms.width;
    let y = in.tex_coords.y * uniforms.height;
    let t = uniforms.time;
    let inten = uniforms.intensity;

    if (uniforms.mode == 0u) {
        // BlackLayer: Uniform coverage
        return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);

    } else if (uniforms.mode == 1u) {
        // VerticalLouver (Cross-Louver with Diagonal Protection)
        let period = uniforms.period_px * inten;
        let stripe_width = period * clamp(uniforms.cover_ratio, 0.3, 0.9);
        let edge_px: f32 = max(1.5, period * 0.08);
        let scroll_speed = uniforms.scroll_speed_px;
        let burn_in_offset = select(0.0, t * 0.2, uniforms.panel_type == 1u);

        // 1. 水平縞 (上下方向)
        let scrolled_y = (y + t * scroll_speed) % period;
        let alpha_h = calc_louver(scrolled_y, stripe_width, edge_px, uniforms.alpha);

        // 2. 垂直縞 (左右方向)
        let scrolled_x = (x + t * scroll_speed * 0.7 + burn_in_offset) % period;
        let alpha_v = calc_louver(scrolled_x, stripe_width, edge_px, uniforms.alpha);

        // 3. 斜め成分 (hatch_angle 活用)
        let cos_a = cos(uniforms.hatch_angle);
        let sin_a = sin(uniforms.hatch_angle);
        let rotated = ((x * cos_a + y * sin_a) % period + period) % period;
        let scrolled_d = (rotated + t * scroll_speed * 0.5) % period;
        let alpha_d = calc_louver(scrolled_d, stripe_width, edge_px, uniforms.alpha);

        // OR 合成: どの方向かの縞に当たれば遮蔽する
        let alpha_out = max(max(alpha_h, alpha_v), alpha_d);
        return vec4<f32>(0.0, 0.0, 0.0, alpha_out);

    } else if (uniforms.mode == 2u) {
        // ── AIOcrInterference (Phase 5: Subpixel UHD Jamming Prototype) ──
        // ストライプ化を避けるため、千鳥格子（Staggered Grid）状のマイクロアパーチャを使用
        let p = uniforms.period_px * 0.4 * max(inten, 0.2);
        let row = floor(y / p);
        let x_off = select(0.0, p * 0.5, row % 2.0 == 0.0);
        let x_p = (x + x_off + t * uniforms.scroll_speed_px * 0.1) % p;
        let y_p = y % p;
        
        // 四角形に近いアパーチャ（開口部）
        let is_aperture = (x_p < p * 0.45) && (y_p < p * 0.45);
        var alpha_main = select(uniforms.alpha, 0.0, is_aperture);

        // サブピクセル・ノイズ (R,G,B 別々に干渉)
        let n_coord = floor(vec2<f32>(x, y) * 2.0); // 擬似サブピクセル解像度
        let n = stable_noise(n_coord + floor(t * 24.0));
        let alpha_noise = select(0.0, uniforms.alpha * 0.4, n > 0.85);
        
        let final_alpha = clamp(max(alpha_main, alpha_noise), 0.0, 1.0);
        return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
    } else if (uniforms.mode == 3u) {
        // ── HighIntensitySPD (Adaptive Narrow-Pixel Strategy) ─────────
        if (uniforms.panel_type == 1u) {
            // OLED V5 Subpixel Kinetic Void
            // 正面からは「非常に細かい砂嵐」に見えるが、斜めからは「面」として遮蔽
            let p = max(uniforms.period_px * 0.2, 2.0);
            let phase = floor(t * 60.0) % 4.0 * (p * 0.25);
            let x_p = (x + phase) % p;
            let y_p = (y + phase * 0.7) % p;
            
            // 非常に狭い開口部 (Narrow Pixel)
            let is_slit = (x_p < p * 0.1) || (y_p < p * 0.1);
            let alpha_base = uniforms.alpha * 0.95;
            let alpha_slit = uniforms.alpha * 0.1;
            var alpha_main = select(alpha_base, alpha_slit, is_slit);

            // 高周波サブピクセル干渉
            let sub_n = stable_noise(vec2<f32>(x * 3.0, y) + t);
            let final_alpha = clamp(alpha_main + select(0.0, 0.2, sub_n > 0.9), 0.0, 1.0);
            return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
        } else {
            // LCD V7 Luminous Crystal (Subpixel Selective)
            let p = max(uniforms.period_px * 0.12, 1.5);
            let slow_offset = t * 0.2;
            
            let x_p = (x + slow_offset) % p;
            let is_slit = x_p < (p * 0.15 * inten);
            let alpha_main = select(uniforms.alpha * 0.92, 0.0, is_slit);

            // Bayer Dithering (Fixed alignment switch)
            let bx = i32(x) % 4;
            let by = i32(y) % 4;
            let b_idx = by * 4 + bx;
            var b_v: f32;
            switch (b_idx) {
                case 0: { b_v = 0.0; } case 1: { b_v = 8.0; } case 2: { b_v = 2.0; } case 3: { b_v = 10.0; }
                case 4: { b_v = 12.0; } case 5: { b_v = 4.0; } case 6: { b_v = 14.0; } case 7: { b_v = 6.0; }
                case 8: { b_v = 3.0; } case 9: { b_v = 11.0; } case 10: { b_v = 1.0; } case 11: { b_v = 9.0; }
                case 12: { b_v = 15.0; } case 13: { b_v = 7.0; } case 14: { b_v = 13.0; } case 15: { b_v = 5.0; }
                default: { b_v = 0.0; }
            }
            let b_val = b_v / 16.0;
            let mesh_alpha = select(0.0, uniforms.alpha * 0.25, b_val > 0.75);

            let final_alpha = clamp(alpha_main + mesh_alpha, 0.0, 1.0);
            // 視認性向上のための微細な発光成分 (Haze)
            let haze = select(0.0, 0.05 * uniforms.alpha, b_val > 0.95);

            return vec4<f32>(haze, haze, haze, final_alpha);
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
}
