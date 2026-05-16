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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.tex_coords.x * uniforms.width;
    let y = in.tex_coords.y * uniforms.height;

    // intensity はユーザー設定の倍率として乗算（1.0=標準、0.5=高密度、2.0=低密度）
    let period = uniforms.period_px * uniforms.intensity;
    let cover_ratio = clamp(uniforms.cover_ratio, 0.3, 0.9);
    let stripe_width = period * cover_ratio;
    let scroll_speed = uniforms.scroll_speed_px;

    // OLED のバーンイン防止オフセット
    let burn_in_offset = select(0.0, uniforms.time * 0.2, uniforms.panel_type == 1u);

    let edge_px: f32 = max(1.5, period * 0.08); // エッジブラーも周期に比例させる

    if (uniforms.mode == 0u) {
        // BlackLayer: Uniform coverage
        return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);

    } else if (uniforms.mode == 1u) {
        // VerticalLouver
        let scrolled_y = (y + uniforms.time * scroll_speed) % period;
        let dist_y = min(scrolled_y, period - scrolled_y);
        
        var alpha_out: f32;
        if (scrolled_y < stripe_width) {
            // Blocked area with edge blur
            let dist_from_edge = min(stripe_width - scrolled_y, scrolled_y);
            alpha_out = uniforms.alpha * select(1.0, dist_from_edge / edge_px, dist_from_edge < edge_px);
        } else {
            // Transparent area with low-luminance gray
            let gap_y = scrolled_y - stripe_width;
            let gap_period = period - stripe_width;
            let dist_from_gap_edge = min(gap_y, gap_period - gap_y);
            let base_gray = uniforms.alpha * 0.15;
            alpha_out = base_gray * select(1.0, 1.0 - (edge_px - dist_from_gap_edge) / edge_px, dist_from_gap_edge < edge_px);
        }

        if (uniforms.bidirectional == 1u) {
            let scrolled_x = (x + uniforms.time * scroll_speed * 0.7 + burn_in_offset) % period;
            let dist_x = min(scrolled_x, period - scrolled_x);
            var alpha_x: f32;
            if (scrolled_x < stripe_width) {
                let dist_from_edge_x = min(stripe_width - scrolled_x, scrolled_x);
                alpha_x = uniforms.alpha * select(1.0, dist_from_edge_x / edge_px, dist_from_edge_x < edge_px);
            } else {
                alpha_x = uniforms.alpha * 0.15;
            }
            alpha_out = max(alpha_out, alpha_x);
        }

        return vec4<f32>(0.0, 0.0, 0.0, alpha_out);

    } else if (uniforms.mode == 2u) {
        // FastVibration
        let phase = f32(u32(uniforms.time * uniforms.phase_flip_hz) % 2u) * period * 0.5;
        let scrolled_y = (y + uniforms.time * scroll_speed + phase) % period;
        let blocked_primary = scrolled_y < stripe_width;

        let period2 = period * 1.5;
        let stripe2 = stripe_width * 1.2;
        let scrolled_diag = ((x * 0.5 + y) + uniforms.time * scroll_speed * 0.6) % period2;
        let blocked_secondary = scrolled_diag < stripe2;

        var blocked = blocked_primary;
        if (uniforms.bidirectional == 1u) {
            let scrolled_x = (x + phase + burn_in_offset) % period;
            blocked = blocked || (scrolled_x < stripe_width);
        }

        if (blocked) {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
        } else if (blocked_secondary) {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.4);
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.05);
        }

    } else if (uniforms.mode == 3u) {
        // AsymmetricCurve
        let scale = 0.15 / max(uniforms.intensity, 0.1);
        let threshold = -0.3 + uniforms.intensity * 0.4;
        let val = sin(x * scale + uniforms.time * 5.0)
                * cos(y * scale - uniforms.time * 3.3);
        let blocked = val > threshold;

        return select(
            vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.1),
            vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha),
            blocked
        );

    } else if (uniforms.mode == 4u) {
        // AIOcrInterference
        let p = vec2<f32>(x / (2.0 * max(uniforms.intensity, 0.1)), y / (2.0 * max(uniforms.intensity, 0.1)));
        let seed = floor(uniforms.time * 30.0);
        let n = fract(sin(dot(floor(p), vec2<f32>(127.1, 311.7)) + seed) * 43758.5453);
        let blocked = n > 0.5;

        return select(
            vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.1),
            vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha),
            blocked
        );
    } else if (uniforms.mode == 5u) {
        // ── LcdContrastJammer ──────────────────────────────────────────
        let grid_p  = max(uniforms.grid_period_px, 2.0);
        let lc      = clamp(uniforms.luminance_compress, 0.0, 0.5);
        let t       = uniforms.time;
        let inten   = uniforms.intensity;

        // ── Layer A: 輝度圧縮マイクログリッド ────────────────────────
        let gx = floor(x / grid_p);
        let gy = floor(y / grid_p);
        let checker = (i32(gx) + i32(gy)) % 2;
        let lum_mod = lc * select(-1.0, 1.0, checker == 0) / max(inten, 0.5);
        let alpha_a = uniforms.alpha * (0.12 + abs(lum_mod));

        // ── Layer B: 逆コントラスト斜線ハッチング ────────────────────
        let hatch_angle = uniforms.hatch_angle;
        let hatch_p  = grid_p * 3.0;
        let rotated  = x * cos(hatch_angle) + y * sin(hatch_angle);
        let hatch_phase = (rotated % hatch_p) / hatch_p;
        let hatch_blocked = hatch_phase < (0.15 / max(inten, 0.5));
        let alpha_b = select(0.0, uniforms.alpha * 0.25, hatch_blocked);

        // ── Layer C: 動的位相ノイズ ─────────────────────
        let noise_p  = grid_p * 2.0;
        let noise_phase = fract(
            sin(dot(
                floor(vec2<f32>(x, y) / noise_p),
                vec2<f32>(127.1, 311.7)
            ) + floor(t * 8.0)) * 43758.5453
        );
        let alpha_c = uniforms.alpha * noise_phase * 0.08;

        let alpha_combined = clamp(alpha_a + alpha_b + alpha_c, 0.0, uniforms.alpha * 0.50);
        return vec4<f32>(0.0, 0.0, 0.0, alpha_combined);
    }

    return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
}
