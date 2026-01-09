// Shader: particle.wgsl
// Purpose: Soft circle particle rendering with instanced geometry and HDR bloom output
// Bindings:
//   @group(0) @binding(0) - Camera uniforms (view_proj matrix, viewport)
//   @group(1) @binding(0) - Global uniforms (time, audio_level)
//
// Features:
//   - Instanced quad rendering for particles
//   - Soft circle falloff using smoothstep
//   - Per-instance: position, color, opacity, scale, bloom_contribution
//   - Audio-reactive scaling support
//   - HDR output for bloom threshold extraction
//   - Additive blend mode compatible (light accumulation)

// ============================================================================
// Uniform Structures
// ============================================================================

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    viewport: vec2<f32>,
    _padding: vec2<f32>,
}

struct GlobalUniforms {
    time: f32,
    delta_time: f32,
    audio_level: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_high: f32,
    _padding: vec2<f32>,
}

// ============================================================================
// Vertex Structures
// ============================================================================

struct VertexInput {
    // Quad vertex (4 vertices per particle: -1,-1 to 1,1)
    @location(0) quad_pos: vec2<f32>,
    @location(1) quad_uv: vec2<f32>,

    // Per-instance data
    @location(2) instance_position: vec3<f32>,
    @location(3) instance_color: vec4<f32>,      // RGB + opacity in alpha
    @location(4) instance_scale: f32,            // Base scale (2px to 24px range)
    @location(5) instance_bloom: f32,            // Bloom contribution (0.0 to 1.0)
    @location(6) instance_audio_react: f32,      // Audio reactivity multiplier
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,                  // UV for circle computation
    @location(1) color: vec3<f32>,               // Particle color (RGB)
    @location(2) opacity: f32,                   // Particle opacity (0.2 to 0.9)
    @location(3) bloom_contribution: f32,        // How much this particle contributes to bloom
}

// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var<uniform> globals: GlobalUniforms;

// ============================================================================
// Utility Functions
// ============================================================================

// Soft circle falloff using smoothstep for anti-aliased edges
fn soft_circle(uv: vec2<f32>, softness: f32) -> f32 {
    let dist = length(uv);
    // Inner radius where full opacity begins
    let inner = 1.0 - softness;
    // Smooth falloff from inner to outer edge
    return 1.0 - smoothstep(inner, 1.0, dist);
}

// Gaussian-like falloff for even softer particles
fn gaussian_circle(uv: vec2<f32>) -> f32 {
    let dist_sq = dot(uv, uv);
    // Gaussian falloff: e^(-dist^2 * factor)
    // Factor of 2.5 gives nice soft edge while keeping center bright
    return exp(-dist_sq * 2.5);
}

// Combined soft circle with gaussian center for glow effect
fn glow_circle(uv: vec2<f32>, softness: f32) -> f32 {
    let soft = soft_circle(uv, softness);
    let gaussian = gaussian_circle(uv);
    // Blend: gaussian for bright center, soft circle for defined edge
    return mix(soft, gaussian, 0.3);
}

// Audio-reactive scale modulation
fn audio_modulated_scale(base_scale: f32, audio_react: f32, audio_level: f32) -> f32 {
    // Scale multiplier based on audio level
    // audio_react controls sensitivity (0 = no reaction, 1 = full reaction)
    let audio_mult = 1.0 + (audio_level * audio_react * 0.5);
    return base_scale * audio_mult;
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate audio-modulated scale
    let final_scale = audio_modulated_scale(
        in.instance_scale,
        in.instance_audio_react,
        globals.audio_level
    );

    // Billboard the quad: expand quad vertices by scale
    // Quad is centered at instance position, facing camera
    let scaled_offset = in.quad_pos * final_scale;

    // Transform instance position to view space
    let world_pos = vec4<f32>(
        in.instance_position.x + scaled_offset.x,
        in.instance_position.y + scaled_offset.y,
        in.instance_position.z,
        1.0
    );

    // Project to clip space
    out.clip_position = camera.view_proj * world_pos;

    // Pass UV for circle computation (centered at 0,0)
    out.uv = in.quad_pos;

    // Pass instance properties to fragment shader
    out.color = in.instance_color.rgb;
    out.opacity = in.instance_color.a;
    out.bloom_contribution = in.instance_bloom;

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Output structure for HDR rendering with bloom
struct FragmentOutput {
    @location(0) color: vec4<f32>,       // Main HDR color output
    // Optional: separate bloom target
    // @location(1) bloom: vec4<f32>,
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate soft circle alpha
    // Using glow_circle for combination of defined edge and soft center
    let softness = 0.4; // Adjust for softer/harder edges
    let circle_alpha = glow_circle(in.uv, softness);

    // Discard pixels outside the circle (optimization)
    if (circle_alpha < 0.001) {
        discard;
    }

    // Combine circle falloff with instance opacity
    // Clamp opacity to specified range (0.2 to 0.9)
    let clamped_opacity = clamp(in.opacity, 0.2, 0.9);
    let final_alpha = circle_alpha * clamped_opacity;

    // Calculate HDR color for bloom
    // Bloom contribution amplifies color beyond 1.0 for bloom threshold
    // Base color + bloom boost creates values > 1.0 that bloom extraction catches
    let bloom_boost = 1.0 + (in.bloom_contribution * 2.0);
    let hdr_color = in.color * bloom_boost;

    // Output HDR color with premultiplied alpha for additive blending
    // Additive blend: src_color * src_alpha + dst_color
    // By premultiplying, we get proper light accumulation
    return vec4<f32>(hdr_color * final_alpha, final_alpha);
}

// ============================================================================
// Alternative Fragment: Separate Bloom Output
// ============================================================================
// Use this variant if your render pipeline has a separate bloom render target

// @fragment
// fn fragment_main_with_bloom(in: VertexOutput) -> FragmentOutput {
//     var out: FragmentOutput;
//
//     let softness = 0.4;
//     let circle_alpha = glow_circle(in.uv, softness);
//
//     if (circle_alpha < 0.001) {
//         discard;
//     }
//
//     let clamped_opacity = clamp(in.opacity, 0.2, 0.9);
//     let final_alpha = circle_alpha * clamped_opacity;
//
//     // Main color output (standard range)
//     out.color = vec4<f32>(in.color * final_alpha, final_alpha);
//
//     // Bloom output (only bright parts)
//     let bloom_intensity = in.bloom_contribution * circle_alpha * clamped_opacity;
//     out.bloom = vec4<f32>(in.color * bloom_intensity, bloom_intensity);
//
//     return out;
// }
