// Shader: post_process
// Purpose: Full-screen post-processing with bloom, chromatic aberration, vignette, and film grain
// Bindings: uniforms (group 0, binding 0), scene_texture (group 0, binding 1), scene_sampler (group 0, binding 2)
// Compatible with: Bevy 0.13 rendering pipeline

// ============================================================================
// UNIFORM STRUCTURES
// ============================================================================

struct PostProcessUniforms {
    // Time for animated effects
    time: f32,
    // Screen resolution (width, height)
    resolution: vec2<f32>,

    // Bloom parameters
    bloom_threshold: f32,      // Default: 0.7
    bloom_intensity: f32,      // Range: 0.1-0.6
    bloom_radius: f32,         // Default: 8.0 pixels

    // Chromatic aberration
    chromatic_intensity: f32,  // Range: 0.001-0.003, increases Act III

    // Vignette parameters
    vignette_intensity: f32,   // Stronger early acts, dissolves by Act V
    vignette_smoothness: f32,  // Edge falloff (default: 0.5)

    // Film grain
    grain_intensity: f32,      // Default: 0.02 (minimal organic texture)

    // Padding for alignment (WebGPU requires 16-byte alignment)
    _padding: f32,
}

// ============================================================================
// BINDINGS
// ============================================================================

@group(0) @binding(0)
var<uniform> uniforms: PostProcessUniforms;

@group(0) @binding(1)
var scene_texture: texture_2d<f32>;

@group(0) @binding(2)
var scene_sampler: sampler;

// ============================================================================
// VERTEX SHADER - Full-screen quad
// ============================================================================

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate full-screen triangle (more efficient than quad)
    // Vertices: (-1,-1), (3,-1), (-1,3) covers entire screen
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Fast pseudo-random hash function
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Animated noise for film grain
fn film_grain_noise(uv: vec2<f32>, time: f32) -> f32 {
    let seed = uv * uniforms.resolution + vec2<f32>(time * 1000.0, time * 1337.0);
    return hash(seed) * 2.0 - 1.0;
}

// Luminance calculation (Rec. 709)
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Soft threshold for bloom extraction
fn soft_threshold(color: vec3<f32>, threshold: f32) -> vec3<f32> {
    let brightness = luminance(color);
    let soft = brightness - threshold + 0.5;
    let contribution = clamp(soft * soft * 0.25, 0.0, 1.0);
    return color * contribution;
}

// ============================================================================
// BLOOM FUNCTIONS
// ============================================================================

// 9-tap Gaussian weights (sigma ~= 2.0, normalized)
const GAUSSIAN_WEIGHTS: array<f32, 5> = array<f32, 5>(
    0.227027,  // center
    0.1945946, // offset 1
    0.1216216, // offset 2
    0.054054,  // offset 3
    0.016216   // offset 4
);

// Separable Gaussian blur - horizontal pass
fn blur_horizontal(uv: vec2<f32>, radius: f32) -> vec3<f32> {
    let texel_size = 1.0 / uniforms.resolution;
    var result = textureSample(scene_texture, scene_sampler, uv).rgb * GAUSSIAN_WEIGHTS[0];

    for (var i = 1; i < 5; i++) {
        let offset = texel_size.x * f32(i) * radius / 8.0;
        let uv_left = vec2<f32>(uv.x - offset, uv.y);
        let uv_right = vec2<f32>(uv.x + offset, uv.y);
        result += textureSample(scene_texture, scene_sampler, uv_left).rgb * GAUSSIAN_WEIGHTS[i];
        result += textureSample(scene_texture, scene_sampler, uv_right).rgb * GAUSSIAN_WEIGHTS[i];
    }

    return result;
}

// Separable Gaussian blur - vertical pass
fn blur_vertical(uv: vec2<f32>, radius: f32) -> vec3<f32> {
    let texel_size = 1.0 / uniforms.resolution;
    var result = textureSample(scene_texture, scene_sampler, uv).rgb * GAUSSIAN_WEIGHTS[0];

    for (var i = 1; i < 5; i++) {
        let offset = texel_size.y * f32(i) * radius / 8.0;
        let uv_up = vec2<f32>(uv.x, uv.y - offset);
        let uv_down = vec2<f32>(uv.x, uv.y + offset);
        result += textureSample(scene_texture, scene_sampler, uv_up).rgb * GAUSSIAN_WEIGHTS[i];
        result += textureSample(scene_texture, scene_sampler, uv_down).rgb * GAUSSIAN_WEIGHTS[i];
    }

    return result;
}

// Combined bloom with threshold extraction
// Note: For production, bloom should use multiple render passes
// This is a single-pass approximation for demonstration
fn apply_bloom(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let texel_size = 1.0 / uniforms.resolution;
    let radius = uniforms.bloom_radius;

    // Extract bright pixels and blur in a simplified single pass
    var bloom = vec3<f32>(0.0);
    var total_weight = 0.0;

    // Sample in a cross pattern for approximate bloom
    for (var x = -4; x <= 4; x++) {
        for (var y = -4; y <= 4; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size * radius / 4.0;
            let sample_uv = uv + offset;
            let sample_color = textureSample(scene_texture, scene_sampler, sample_uv).rgb;
            let bright = soft_threshold(sample_color, uniforms.bloom_threshold);

            // Gaussian-like weight based on distance
            let dist = length(vec2<f32>(f32(x), f32(y)));
            let weight = exp(-dist * dist / 8.0);

            bloom += bright * weight;
            total_weight += weight;
        }
    }

    bloom /= total_weight;

    // Additive blend
    return color + bloom * uniforms.bloom_intensity;
}

// ============================================================================
// CHROMATIC ABERRATION
// ============================================================================

fn apply_chromatic_aberration(uv: vec2<f32>) -> vec3<f32> {
    // Calculate direction from center for radial aberration
    let center = vec2<f32>(0.5, 0.5);
    let direction = uv - center;
    let dist_from_center = length(direction);

    // Aberration increases toward edges (radial)
    let aberration = uniforms.chromatic_intensity * dist_from_center;

    // Offset RGB channels
    let r_offset = direction * aberration;
    let b_offset = -direction * aberration;

    // Sample each channel with offset
    let r = textureSample(scene_texture, scene_sampler, uv + r_offset).r;
    let g = textureSample(scene_texture, scene_sampler, uv).g;
    let b = textureSample(scene_texture, scene_sampler, uv + b_offset).b;

    return vec3<f32>(r, g, b);
}

// ============================================================================
// VIGNETTE
// ============================================================================

fn apply_vignette(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    // Distance from center (0 at center, ~0.707 at corners)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);

    // Smooth vignette falloff
    // Starts fading at distance 0.4, fully dark at ~0.9 (adjusted by smoothness)
    let vignette_start = 0.4;
    let vignette_end = 0.9 - uniforms.vignette_smoothness * 0.3;

    let vignette = 1.0 - smoothstep(vignette_start, vignette_end, dist) * uniforms.vignette_intensity;

    return color * vignette;
}

// ============================================================================
// FILM GRAIN
// ============================================================================

fn apply_film_grain(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    // Generate animated noise
    let noise = film_grain_noise(uv, uniforms.time);

    // Apply grain based on luminance (more visible in midtones)
    let lum = luminance(color);
    let grain_mask = 1.0 - abs(lum - 0.5) * 2.0; // Peak at 0.5 luminance

    // Add grain with intensity control
    let grain = noise * uniforms.grain_intensity * grain_mask;

    return color + vec3<f32>(grain);
}

// ============================================================================
// MAIN FRAGMENT SHADER
// ============================================================================

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Step 1: Sample with chromatic aberration
    var color = apply_chromatic_aberration(uv);

    // Step 2: Apply bloom
    color = apply_bloom(color, uv);

    // Step 3: Apply vignette
    color = apply_vignette(color, uv);

    // Step 4: Apply film grain (last to avoid grain in bloom)
    color = apply_film_grain(color, uv);

    // Clamp final output
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(color, 1.0);
}

// ============================================================================
// BLOOM EXTRACTION PASS (for multi-pass pipeline)
// ============================================================================

@fragment
fn bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(scene_texture, scene_sampler, in.uv).rgb;
    let bright = soft_threshold(color, uniforms.bloom_threshold);
    return vec4<f32>(bright, 1.0);
}

// ============================================================================
// BLOOM BLUR HORIZONTAL PASS
// ============================================================================

@fragment
fn bloom_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    let blurred = blur_horizontal(in.uv, uniforms.bloom_radius);
    return vec4<f32>(blurred, 1.0);
}

// ============================================================================
// BLOOM BLUR VERTICAL PASS
// ============================================================================

@fragment
fn bloom_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    let blurred = blur_vertical(in.uv, uniforms.bloom_radius);
    return vec4<f32>(blurred, 1.0);
}

// ============================================================================
// BLOOM COMPOSITE PASS
// ============================================================================
// Note: Requires bloom_texture binding for production use
// This entry point demonstrates the composite step

@fragment
fn bloom_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(scene_texture, scene_sampler, in.uv).rgb;
    // In production: let bloom = textureSample(bloom_texture, bloom_sampler, in.uv).rgb;
    // return vec4<f32>(scene + bloom * uniforms.bloom_intensity, 1.0);
    return vec4<f32>(scene, 1.0);
}
