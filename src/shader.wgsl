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
    mode: u32, // 0: Black, 1: Louver, 2: HighSpeed, 3: Asymmetric, 4: AIOcr
    alpha: f32,
    width: f32,
    height: f32,
    panel_type: u32, // 0: Unknown, 1: Oled, 2: LcdIps, 3: LcdTn
    refresh_rate: u32,
    intensity: f32,
    bidirectional: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.tex_coords.x * uniforms.width;
    let y = in.tex_coords.y * uniforms.height;

    // Bug Fix 1: Determine scroll speed with clear conditional logic
    var scroll_speed: f32 = 300.0; // Default for Unknown (0u)
    if (uniforms.panel_type == 1u) {
        scroll_speed = 0.2;    // OLED: Burn-in protection
    } else if (uniforms.panel_type == 2u) {
        scroll_speed = 600.0;  // LCD IPS: High speed
    } else if (uniforms.panel_type == 3u) {
        scroll_speed = 300.0;  // LCD TN: Medium speed
    }

    // Improvement E: Intensity scales coverage ratio, not physical size
    let period: f32 = 6.0; 
    let cover_ratio = clamp(0.8 / max(uniforms.intensity, 0.1), 0.3, 0.9);
    let stripe_width = period * cover_ratio;
    let edge_px: f32 = 1.5; // Improvement D: Edge blur width

    if (uniforms.mode == 0u) {
        // BlackLayer: Uniform coverage
        return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);

    } else if (uniforms.mode == 1u) {
        // VerticalLouver Improvement A, D, E
        let scrolled_y = (y + uniforms.time * scroll_speed) % period;
        let dist_y = min(scrolled_y, period - scrolled_y);
        
        var alpha_out: f32;
        if (scrolled_y < stripe_width) {
            // Blocked area with edge blur
            let dist_from_edge = min(stripe_width - scrolled_y, scrolled_y);
            alpha_out = uniforms.alpha * select(1.0, dist_from_edge / edge_px, dist_from_edge < edge_px);
        } else {
            // Transparent area with low-luminance gray (Improvement A)
            let gap_y = scrolled_y - stripe_width;
            let gap_period = period - stripe_width;
            let dist_from_gap_edge = min(gap_y, gap_period - gap_y);
            let base_gray = uniforms.alpha * 0.15;
            alpha_out = base_gray * select(1.0, 1.0 - (edge_px - dist_from_gap_edge) / edge_px, dist_from_gap_edge < edge_px);
        }

        if (uniforms.bidirectional == 1u) {
            let scrolled_x = (x + uniforms.time * scroll_speed * 0.7) % period;
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
        // FastVibration Improvement B (Double Layer Moiré)
        let phase = f32(u32(uniforms.time * 30.0) % 2u) * period * 0.5;
        let scrolled_y = (y + uniforms.time * scroll_speed + phase) % period;
        let blocked_primary = scrolled_y < stripe_width;

        // Layer 2: Different period/speed/diagonal (Improvement B)
        let period2 = period * 1.5;
        let stripe2 = stripe_width * 1.2;
        let scrolled_diag = ((x * 0.5 + y) + uniforms.time * scroll_speed * 0.6) % period2;
        let blocked_secondary = scrolled_diag < stripe2;

        var blocked = blocked_primary;
        if (uniforms.bidirectional == 1u) { // Bug Fix 2: Respect bidirectional
            let scrolled_x = (x + phase) % period;
            blocked = blocked || (scrolled_x < stripe_width);
        }

        if (blocked) {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
        } else if (blocked_secondary) {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.4);
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha * 0.05); // Subtle base for moiré
        }

    } else if (uniforms.mode == 3u) {
        // AsymmetricCurve Improvement C (Dynamic Threshold)
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
    }

    return vec4<f32>(0.0, 0.0, 0.0, uniforms.alpha);
}
