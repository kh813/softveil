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
    mode: u32, // 0: Black, 1: Louver, 2: HighSpeed, 3: Asymmetric, 4: AIOcr, 5: LcdContrastJammer
    alpha: f32,
    width: f32,
    height: f32,
    panel_type: u32, // 0: Unknown, 1: Oled, 2: LcdIps, 3: LcdTn
    refresh_rate: u32,
    intensity: f32,
    bidirectional: u32,
    // 物理サイズ適応パラメータ
    period_px: f32,
    scroll_speed_px: f32,
    cover_ratio: f32,
    phase_flip_hz: f32,
    // LCD コントラストジャマー用
    grid_period_px: f32,
    luminance_compress: f32,
    hatch_angle: f32,
    _pad0: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn calc_louver(scrolled: f32, stripe_width: f32, edge_px: f32, base_alpha: f32) -> f32 {
    if (scrolled < stripe_width) {
        let dist_from_edge = min(stripe_width - scrolled, scrolled);
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
        let rotated = x * cos_a + y * sin_a;
        let scrolled_d = (rotated + t * scroll_speed * 0.5) % period;
        let alpha_d = calc_louver(scrolled_d, stripe_width, edge_px, uniforms.alpha);

        // OR 合成: どの方向かの縞に当たれば遮蔽する
        let alpha_out = max(max(alpha_h, alpha_v), alpha_d);
        return vec4<f32>(0.0, 0.0, 0.0, alpha_out);

    } else if (uniforms.mode == 2u) {
        // AIOcrInterference (OLED V2)
        let p = uniforms.period_px * 0.5 * max(inten, 0.1);
        let x_p = x % p;
        let y_p = y % p;
        let is_slit = (x_p < p * 0.1) && (y_p < p * 0.8);
        let alpha_main = select(uniforms.alpha, 0.0, is_slit);
        let n = fract(sin(dot(vec2<f32>(x, y), vec2<f32>(12.9898, 78.233)) + floor(t * 30.0)) * 43758.5453);
        let alpha_noise = select(0.0, uniforms.alpha * 0.3, n > 0.8);
        let final_alpha = clamp(max(alpha_main, alpha_noise), 0.0, 1.0);
        return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
    } else if (uniforms.mode == 3u) {
        // ── HighIntensitySPD (Adaptive OLED/LCD) ─────────────────────
        if (uniforms.panel_type == 1u) {
            // OLED V4 Kinetic Void
            let p = max(uniforms.period_px * 0.25, 3.0);
            let phase = floor(t * 30.0) % 2.0 * (p * 0.5);
            let scrolled_x = (x + phase) % p;
            let slit_w = p * 0.06 * inten;
            let is_slit = scrolled_x < slit_w;
            let alpha_slit = uniforms.alpha * 0.05;
            let alpha_block = uniforms.alpha * 0.98;
            var alpha_main = select(alpha_block, alpha_slit, is_slit);
            let noise_p = floor(t * 60.0);
            let n = fract(sin(dot(floor(vec2<f32>(x, y)), vec2<f32>(12.9898, 78.233)) + noise_p) * 43758.5453);
            let alpha_noise = select(0.0, uniforms.alpha * 0.3, n > 0.8);
            let final_alpha = clamp(max(alpha_main, alpha_noise), 0.0, 1.0);
            return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
        } else {
            // LCD V6 Luminous Crystal (Comfort SPD)
            let p = max(uniforms.period_px * 0.15, 2.0);
            let slow_offset = t * 0.15;
            
            // ── シャープ・ルーバー ──────────────────
            let x_p = (x + slow_offset) % p;
            let slit_w = p * 0.15 * inten;
            let is_slit = x_p < slit_w;
            let alpha_main = select(uniforms.alpha * 0.95, 0.0, is_slit);

            // ── 規則的網点 (Bayer Ordered Dithering) ──────────────────
            let b_size = 4.0;
            let bx = i32(x % b_size);
            let by = i32(y % b_size);
            let bayer = array<f32, 16>(
                 0.0, 8.0, 2.0,10.0,
                12.0, 4.0,14.0, 6.0,
                 3.0,11.0, 1.0, 9.0,
                15.0, 7.0,13.0, 5.0
            );
            let b_val = bayer[by * 4 + bx] / 16.0;
            let mesh_alpha = select(0.0, uniforms.alpha * 0.3, b_val > 0.7);

            // ── 輝度圧縮 (Luminance Compress) ──────────────────────────
            let lum_mask = select(0.0, uniforms.luminance_compress, b_val > 0.5);

            // ── 合成 (Fix #1: 正しく final_alpha を計算して出力する) ────
            let final_alpha = clamp(alpha_main + mesh_alpha + lum_mask, 0.0, 1.0);
            let haze = select(0.0, 0.08 * uniforms.alpha, b_val > 0.9);

            return vec4<f32>(haze, haze, haze, final_alpha);
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
}
