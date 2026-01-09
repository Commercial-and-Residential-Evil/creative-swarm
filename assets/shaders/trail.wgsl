// Shader: trail
// Purpose: Renders particle trails with exponential opacity decay, width taper, and soft edges
// Bindings:
//   - group(0) binding(0): view uniforms (camera matrices)
//   - group(1) binding(0): trail uniforms (time, fade duration, segment count)
//   - group(2) binding(0): trail segment storage buffer

// ============================================================================
// Constants
// ============================================================================

const TRAIL_SEGMENTS: u32 = 12u;
const FADE_DURATION_MS: f32 = 1500.0;
const PI: f32 = 3.14159265359;

// ============================================================================
// Structures
// ============================================================================

struct ViewUniforms {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    world_position: vec3<f32>,
    viewport: vec4<f32>,
}

struct TrailUniforms {
    time_ms: f32,
    fade_duration_ms: f32,
    base_width: f32,
    softness: f32,
}

struct TrailSegment {
    position: vec3<f32>,
    spawn_time_ms: f32,
    color: vec4<f32>,
    width: f32,
    segment_index: u32,
    _padding: vec2<f32>,
}

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) segment_opacity: f32,
    @location(3) edge_distance: f32,
}

// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0)
var<uniform> view: ViewUniforms;

@group(1) @binding(0)
var<uniform> trail: TrailUniforms;

@group(2) @binding(0)
var<storage, read> segments: array<TrailSegment>;

// ============================================================================
// Utility Functions
// ============================================================================

// Exponential decay function for opacity falloff along trail
fn exponential_decay(t: f32, decay_rate: f32) -> f32 {
    return exp(-decay_rate * t);
}

// Smooth step for soft edges
fn smooth_edge(distance: f32, softness: f32) -> f32 {
    return 1.0 - smoothstep(1.0 - softness, 1.0, abs(distance));
}

// Calculate width taper from head (index 0) to tail (index TRAIL_SEGMENTS-1)
fn calculate_width_taper(segment_index: u32, total_segments: u32) -> f32 {
    let t = f32(segment_index) / f32(total_segments - 1u);
    // Quadratic taper for smoother visual
    return 1.0 - t * t;
}

// Calculate time-based fade
fn calculate_time_fade(spawn_time_ms: f32, current_time_ms: f32, fade_duration_ms: f32) -> f32 {
    let age_ms = current_time_ms - spawn_time_ms;
    if (age_ms < 0.0) {
        return 0.0;
    }
    let t = clamp(age_ms / fade_duration_ms, 0.0, 1.0);
    return 1.0 - t;
}

// Get camera right vector for billboarding
fn get_camera_right() -> vec3<f32> {
    return normalize(vec3<f32>(view.inverse_view[0][0], view.inverse_view[1][0], view.inverse_view[2][0]));
}

// Get camera up vector for billboarding
fn get_camera_up() -> vec3<f32> {
    return normalize(vec3<f32>(view.inverse_view[0][1], view.inverse_view[1][1], view.inverse_view[2][1]));
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Each trail segment is rendered as a quad (2 triangles, 6 vertices)
    // Vertex layout for triangle strip emulation:
    // 0--2--4
    // |\ |\ |
    // | \| \|
    // 1--3--5

    let segment_index = in.instance_index;
    let vertex_in_quad = in.vertex_index % 6u;

    // Map 6 vertices to quad corners (0=TL, 1=BL, 2=TR, 3=BR)
    var corner: u32;
    switch (vertex_in_quad) {
        case 0u: { corner = 0u; } // Top-left
        case 1u: { corner = 1u; } // Bottom-left
        case 2u: { corner = 2u; } // Top-right
        case 3u: { corner = 2u; } // Top-right
        case 4u: { corner = 1u; } // Bottom-left
        case 5u: { corner = 3u; } // Bottom-right
        default: { corner = 0u; }
    }

    let segment = segments[segment_index];

    // Calculate segment properties
    let width_taper = calculate_width_taper(segment.segment_index, TRAIL_SEGMENTS);
    let time_fade = calculate_time_fade(segment.spawn_time_ms, trail.time_ms, trail.fade_duration_ms);

    // Exponential opacity decay along trail length (head = full, tail = faded)
    let length_t = f32(segment.segment_index) / f32(TRAIL_SEGMENTS - 1u);
    let length_opacity = exponential_decay(length_t, 2.5);

    // Combined opacity
    let combined_opacity = length_opacity * time_fade;

    // Calculate final width
    let final_width = trail.base_width * width_taper * segment.width;

    // Billboard the quad to face camera
    let camera_right = get_camera_right();
    let camera_up = get_camera_up();

    // Offset based on corner
    let horizontal_offset = select(-1.0, 1.0, (corner & 2u) != 0u);
    let vertical_offset = select(-1.0, 1.0, (corner & 1u) == 0u);

    let world_position = segment.position
        + camera_right * horizontal_offset * final_width * 0.5
        + camera_up * vertical_offset * final_width * 0.5;

    // Transform to clip space
    out.clip_position = view.view_proj * vec4<f32>(world_position, 1.0);

    // Pass through color with inherited particle color
    out.color = segment.color;

    // UV coordinates for potential texture sampling
    out.uv = vec2<f32>(
        select(0.0, 1.0, (corner & 2u) != 0u),
        select(0.0, 1.0, (corner & 1u) != 0u)
    );

    // Store opacity for fragment shader
    out.segment_opacity = combined_opacity;

    // Edge distance for soft edge calculation (-1 to 1, 0 at center)
    out.edge_distance = horizontal_offset;

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center for soft edges
    let center_uv = in.uv * 2.0 - 1.0;
    let distance_from_center = length(center_uv);

    // Soft edge falloff
    let edge_softness = smooth_edge(distance_from_center, trail.softness);

    // Additional radial softness for rounded trail segments
    let radial_falloff = 1.0 - smoothstep(0.7, 1.0, distance_from_center);

    // Combine all opacity factors
    let final_opacity = in.segment_opacity * edge_softness * radial_falloff;

    // Early discard for fully transparent pixels
    if (final_opacity < 0.001) {
        discard;
    }

    // Output color with computed opacity
    // Using premultiplied alpha for proper additive blending
    let premultiplied_color = in.color.rgb * final_opacity;

    return vec4<f32>(premultiplied_color, final_opacity);
}

// ============================================================================
// Alternative: Instanced Line Strip Vertex Shader
// For connecting trail segments as a continuous line
// ============================================================================

struct LineVertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) line_coord: f32,
    @location(2) opacity: f32,
}

@vertex
fn vertex_line(in: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;

    // Two vertices per line segment (start and end)
    let is_end = in.vertex_index % 2u == 1u;
    let segment_a_index = in.instance_index;
    let segment_b_index = min(in.instance_index + 1u, TRAIL_SEGMENTS - 1u);

    let segment_a = segments[segment_a_index];
    let segment_b = segments[segment_b_index];

    // Interpolate position
    let position = select(segment_a.position, segment_b.position, is_end);
    let segment = select(segment_a, segment_b, is_end);

    // Calculate opacity
    let time_fade = calculate_time_fade(segment.spawn_time_ms, trail.time_ms, trail.fade_duration_ms);
    let length_t = f32(segment.segment_index) / f32(TRAIL_SEGMENTS - 1u);
    let length_opacity = exponential_decay(length_t, 2.5);

    out.clip_position = view.view_proj * vec4<f32>(position, 1.0);
    out.color = segment.color;
    out.line_coord = select(0.0, 1.0, is_end);
    out.opacity = length_opacity * time_fade;

    return out;
}

@fragment
fn fragment_line(in: LineVertexOutput) -> @location(0) vec4<f32> {
    if (in.opacity < 0.001) {
        discard;
    }

    let premultiplied_color = in.color.rgb * in.opacity;
    return vec4<f32>(premultiplied_color, in.opacity);
}
