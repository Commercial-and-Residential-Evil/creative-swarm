// Shader: background_gradient
// Purpose: Dynamic full-screen gradient with breathing effect and vignette preparation
// Bindings: uniforms (gradient colors, pulse intensity), time
//
// Act Gradients:
//   Act I:   #0a0a0f -> #1a1a2e (darkness to deep blue)
//   Act II:  #1a1a2e -> #2d1f3d (deep blue to purple-touched)
//   Act III: #3d1f2d -> #4a1a2a (wine-touched darkness)
//   Act IV:  #4a2a3a -> #8a6a7a (warming, lightening)
//   Act V:   #d4c4b4 -> #fdf6f0 (blush to luminous white)

// ============================================================================
// Vertex Input/Output Structures
// ============================================================================

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// ============================================================================
// Uniform Structures
// ============================================================================

struct BackgroundUniforms {
    // Gradient colors (linear RGB, pre-converted from sRGB)
    gradient_start: vec4<f32>,    // Top color (w = unused, for alignment)
    gradient_end: vec4<f32>,      // Bottom color (w = unused, for alignment)

    // Animation parameters
    pulse_intensity: f32,         // Bass frequency influence [0.0 - 1.0]
    time: f32,                    // Elapsed time in seconds

    // Vignette parameters
    vignette_strength: f32,       // Edge darkening strength [0.0 - 1.0]
    vignette_radius: f32,         // Vignette falloff radius [0.5 - 1.5]
}

@group(0) @binding(0)
var<uniform> uniforms: BackgroundUniforms;

// ============================================================================
// Utility Functions
// ============================================================================

// Attempt at smoother interpolation
fn smooth_step_custom(t: f32) -> f32 {
    let clamped = clamp(t, 0.0, 1.0);
    return clamped * clamped * (3.0 - 2.0 * clamped);
}

// Even smoother quintic interpolation (smoother at boundaries)
fn quintic_smooth(t: f32) -> f32 {
    let clamped = clamp(t, 0.0, 1.0);
    return clamped * clamped * clamped * (clamped * (clamped * 6.0 - 15.0) + 10.0);
}

// Compute radial distance from center for vignette
fn compute_vignette(uv: vec2<f32>, strength: f32, radius: f32) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);

    // Smooth falloff from center
    let normalized_dist = dist / radius;
    let vignette = 1.0 - smooth_step_custom(normalized_dist) * strength;

    return clamp(vignette, 0.0, 1.0);
}

// Subtle breathing oscillation based on time and pulse intensity
fn breathing_offset(time: f32, intensity: f32) -> f32 {
    // Primary slow breathing wave
    let slow_wave = sin(time * 0.5) * 0.5 + 0.5;

    // Secondary faster wave for bass pulse response
    let fast_wave = sin(time * 2.0) * 0.5 + 0.5;

    // Blend based on pulse intensity (bass drives the faster wave)
    let blended = mix(slow_wave, fast_wave, intensity * 0.3);

    // Scale to subtle range
    return blended * 0.02 * intensity;
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Full-screen triangle technique (more efficient than quad)
    // Generates a triangle that covers the entire screen
    // vertex 0: (-1, -1), vertex 1: (3, -1), vertex 2: (-1, 3)
    let x = f32(i32(in.vertex_index & 1u) * 4 - 1);
    let y = f32(i32(in.vertex_index >> 1u) * 4 - 1);

    out.position = vec4<f32>(x, y, 0.0, 1.0);

    // UV coordinates: map clip space to [0, 1] range
    // Y is flipped so UV (0,0) is top-left
    out.uv = vec2<f32>(
        (x + 1.0) * 0.5,
        1.0 - (y + 1.0) * 0.5
    );

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate breathing offset for subtle gradient animation
    let breath = breathing_offset(uniforms.time, uniforms.pulse_intensity);

    // Vertical gradient interpolation factor
    // UV.y = 0 at top, UV.y = 1 at bottom
    // Add breathing offset for subtle vertical movement
    let gradient_t = clamp(in.uv.y + breath, 0.0, 1.0);

    // Apply quintic smoothing for pleasing gradient falloff
    let smooth_t = quintic_smooth(gradient_t);

    // Interpolate between gradient colors
    let base_color = mix(
        uniforms.gradient_start.rgb,
        uniforms.gradient_end.rgb,
        smooth_t
    );

    // Apply subtle brightness pulse from bass
    // Pulse adds slight luminance variation
    let pulse_factor = 1.0 + uniforms.pulse_intensity * 0.05 * sin(uniforms.time * 3.0);
    let pulsed_color = base_color * pulse_factor;

    // Calculate vignette darkening
    let vignette = compute_vignette(
        in.uv,
        uniforms.vignette_strength,
        uniforms.vignette_radius
    );

    // Apply vignette to final color
    let final_color = pulsed_color * vignette;

    return vec4<f32>(final_color, 1.0);
}
