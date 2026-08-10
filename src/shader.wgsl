struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

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

fn calc_louver(scrolled: f32, stripe_width: f32, edge_px: f32, base_alpha: f32, is_light_mode: bool) -> f32 {
    let s = scrolled;

    if (s < stripe_width) {
        let dist_from_edge = min(stripe_width - s, s);
        let edge_factor = clamp(dist_from_edge / edge_px, 0.0, 1.0);
        return base_alpha * edge_factor;
    } else {
        // 開口部 (Narrow Pixel 透過エリア): 100% 透過 (alpha = 0) で正面明るさと鮮明度を維持
        return 0.0;
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

    let is_light_mode = uniforms.v4.x > 0.5;
    let cos_a = uniforms.v4.y;
    let sin_a = uniforms.v4.z;

    let x = in.tex_coords.x * width;
    let y = in.tex_coords.y * height;

    if (mode == 0u) {
        // BlackLayer: Uniform coverage
        return vec4<f32>(0.0, 0.0, 0.0, alpha);

    } else if (mode == 1u) {
        // VerticalLouver (Samsung Flex Magic Pixel Inspired Narrow-Aperture Strategy)
        let period = max(period_px * inten, 2.0);
        
        // 正面透過性の向上: 開口部を 100% 透過に保つため、遮蔽線（Wide成分遮蔽）の割合を最適制御
        let stripe_width = period * clamp(cover_ratio * 0.60, 0.10, 0.50);
        let edge_px: f32 = max(0.5, period * 0.03);
        let scroll_speed = scroll_speed_px;
        let burn_in_offset = select(0.0, t * 0.15, panel_type == 1u);

        // Subpixel Micro Phase Stepping: R, G, B で位相を 1/3 周期ずつオフセットして斜め色収差を誘導
        let sx = in.tex_coords.x * width * 3.0;
        let sp_idx = u32(floor(sx)) % 3u;
        let subpixel_shift = f32(sp_idx) * (period * 0.33);

        let scrolled_x = ((x + subpixel_shift + t * scroll_speed * 0.7 + burn_in_offset) % period + period) % period;
        var alpha_out = calc_louver(scrolled_x, stripe_width, edge_px, alpha, is_light_mode);

        if (bidirectional == 1u) {
            let scrolled_y = ((y + subpixel_shift * 0.5 + t * scroll_speed) % period + period) % period;
            let alpha_h = calc_louver(scrolled_y, stripe_width, edge_px, alpha, is_light_mode);

            let rotated = (x * cos_a + y * sin_a);
            let scrolled_d = ((rotated + subpixel_shift * 0.7 + t * scroll_speed * 0.5) % period + period) % period;
            let alpha_d = calc_louver(scrolled_d, stripe_width, edge_px, alpha, is_light_mode);

            alpha_out = max(max(alpha_out, alpha_h), alpha_d);
        }

        if (panel_type == 1u) {
            // OLED: 完璧な真黒による鋭い光束分離 (Narrow Pixel 透過)
            return vec4<f32>(0.0, 0.0, 0.0, alpha_out);
        } else {
            // LCD: 斜め視野角における白浮き/視野角減衰を利用した微細コントラスト障害
            let offset_scale = 0.05 * alpha;
            let r_val = select(0.0, offset_scale, sp_idx == 0u);
            let b_val = select(0.0, offset_scale, sp_idx == 2u);
            return vec4<f32>(r_val, 0.0, b_val, alpha_out);
        }

    } else if (mode == 2u) {
        // ── OcrJammer (Subpixel High-Frequency Micro Jamming) ──
        let p = max(period_px * 0.4 * max(inten, 0.2), 1.0);
        let row = i32(floor(y / p));
        let x_off = select(0.0, p * 0.5, row % 2 == 0);
        let x_p = ((x + x_off + t * scroll_speed_px * 0.1) % p + p) % p;
        let y_p = (y % p + p) % p;
        
        // 正面視感度のための広いアパーチャ (100% 透過)
        let is_aperture = (x_p < p * 0.50) && (y_p < p * 0.50);
        var alpha_main = select(alpha, 0.0, is_aperture);

        let n_coord = floor(vec2<f32>(x, y) * 2.0); 
        let t_step = floor(t * 8.0);
        let n_static = stable_noise(n_coord);
        let n_anim = stable_noise(n_coord + t_step);
        let alpha_noise = select(0.0, alpha * 0.5, (n_static > 0.85) || (n_anim > 0.94));
        
        let final_alpha = clamp(max(alpha_main, alpha_noise), 0.0, 1.0);
        return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
    } else if (mode == 3u) {
        // ── HighIntensitySPD (Samsung Flex Magic Pixel Narrow-Pixel Mode) ─────────
        if (panel_type == 1u) {
            // OLED: Narrow Pixel Self-Emissive Slit (発光光束の指向性擬似再現)
            // ピクセル開口（Narrow成分）を 100% 透過に保ち、Wide成分（広角光）の周縁部のみをアトミック遮蔽
            let p = max(period_px * 0.25, 2.0);
            let fps = max(refresh_rate, 30u);
            let phase = floor(t * f32(fps)) % 4.0 * (p * 0.25);
            let x_p = ((x + phase) % p + p) % p;
            let y_p = ((y + phase * 0.7) % p + p) % p;
            
            // 狭角（Narrow）開口部: 透過 (alpha = 0.0)
            let is_narrow_aperture = (x_p < p * 0.35) && (y_p < p * 0.35);
            let alpha_main = select(alpha, 0.0, is_narrow_aperture);

            // 斜めからの覗き見に対して高周波サブピクセル位相干渉を発生させる
            let sub_n = stable_noise(vec2<f32>(x * 3.0, y) + t * 0.1);
            let final_alpha = clamp(alpha_main + select(0.0, 0.15 * alpha, sub_n > 0.92), 0.0, 1.0);
            return vec4<f32>(0.0, 0.0, 0.0, final_alpha);
            
        } else {
            // LCD: Spatial Aperture & Contrast Collapse
            // 1.5px のクリア窓を残し、遮蔽部は液晶の視野角コントラスト崩壊を利用
            let gap_px = 1.5;
            let stripe_px = max(floor(2.0 / max(1.0 - cover_ratio, 0.1)), 1.0);
            let p = gap_px + stripe_px;
            
            let scroll_x = ((x + t * scroll_speed_px) % p + p) % p;
            
            let in_gap_x = scroll_x < gap_px;
            let final_a_x = select(alpha * 0.9, 0.0, in_gap_x);
            
            if (bidirectional == 1u) {
                let scroll_y = ((y + t * scroll_speed_px * 0.7) % p + p) % p;
                let in_gap_y = scroll_y < gap_px;
                let in_any_gap = in_gap_x || in_gap_y;
                return vec4<f32>(0.0, 0.0, 0.0, select(alpha * 0.9, 0.0, in_any_gap));
            }
            
            return vec4<f32>(0.0, 0.0, 0.0, final_a_x);
        }
    } else if (mode == 4u) {
        // ── StealthDark (Narrow Pixel Emulation & Low-Luma Contrast Collapse) ──
        let dither = (floor(x) + floor(y)) % 2.0;
        let alpha_dither = alpha * 0.10 * dither;

        if (panel_type == 1u) {
            // OLED: Narrow Pixel Grid (100% 透過の狭角アパーチャ + 超静音 drift)
            let p = 2.0;
            let drift = floor(t * 0.1) % 2.0;
            let drift_x = u32(drift);
            let drift_y = u32(drift);
            let is_narrow_pixel = ((u32(x) + drift_x) % 2u == 0u) && ((u32(y) + drift_y) % 2u == 0u);
            
            // Narrow Pixel (開口部) は完全透明 0.0、背景遮蔽部は alpha
            let alpha_main = select(alpha, 0.0, is_narrow_pixel);
            return vec4<f32>(0.0, 0.0, 0.0, max(alpha_main, alpha_dither));
        } else {
            // LCD: Low-Luma Contrast Collapse (LLCC)
            let glow = 0.08 * luminance_compress; 
            
            let p_raw = max(period_px * 0.5, 3.0);
            let p = u32(max(floor(p_raw), 3.0));
            let is_aperture = (u32(floor(x)) % p >= 1u) && (u32(floor(y)) % p >= 1u);
            let alpha_main = select(alpha * 0.85, 0.0, is_aperture);
            
            let final_alpha = clamp(max(alpha_main, alpha_dither), 0.0, 1.0);
            return vec4<f32>(glow, glow, glow, final_alpha);
        }
    } else if (mode == 5u) {
        // ── StealthLight (Subpixel Narrow-Aperture UHD Jamming) ──
        let subpixel_noise = stable_noise(vec2<f32>(floor(x * 3.0), floor(y))) * 0.10 * alpha;

        if (panel_type == 1u) {
            // OLED: Narrow Slit Aperture (高輝度環境向け)
            let is_narrow_slit = (u32(x) % 3u == 0u);
            let alpha_main = select(alpha * 0.85, 0.0, is_narrow_slit);
            return vec4<f32>(0.0, 0.0, 0.0, clamp(alpha_main + subpixel_noise, 0.0, 1.0));
        } else {
            // LCD: Subpixel UHD Jamming with Spatial Contrast Collapse
            let p_raw = max(period_px * 0.4, 2.0);
            let p = u32(max(floor(p_raw), 2.0));
            
            let sx = in.tex_coords.x * width * 3.0;
            let sy = in.tex_coords.y * height;
            let sp_idx = u32(floor(sx)) % 3u;
            
            let veil = 0.15 * alpha;
            let shift = f32(sp_idx);
            let is_fine_aperture = (u32(floor(sx + shift)) % (p * 3u) < 1u);
            
            // 正面高透過 (開口部は alpha = 0.0)
            let alpha_main = select(alpha * 0.75, 0.0, is_fine_aperture);
            let noise = stable_noise(vec2<f32>(floor(sx), floor(sy))) * 0.08 * alpha;
            
            let final_a = clamp(alpha_main + noise, 0.0, 1.0);
            
            // 意図的サブピクセル色干渉
            let offset_scale = 0.05 * alpha;
            let r_offset = select(0.0, offset_scale, sp_idx == 0u);
            let b_offset = select(0.0, -offset_scale, sp_idx == 2u);
            
            return vec4<f32>(
                clamp(veil + r_offset, 0.0, 1.0),
                clamp(veil, 0.0, 1.0),
                clamp(veil + b_offset, 0.0, 1.0),
                final_a
            );
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}
