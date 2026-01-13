//! Module: visual
//! Purpose: Color management, camera setup, and background rendering for Chromatic Elegy
//! Dependencies: types, resources, components, bevy::prelude

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::components::{BackgroundMarker, Particle, ParticleVisual};
use crate::intro::AppState;
use crate::resources::{
    ActState, BackgroundGradients, CurrentBackground, InterpolatedActValues,
};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Target viewport width for the experience.
pub const VIEWPORT_WIDTH: f32 = 1920.0;

/// Target viewport height for the experience.
pub const VIEWPORT_HEIGHT: f32 = 1080.0;

/// Initial clear color matching Act I background (deep navy void).
pub const INITIAL_CLEAR_COLOR: Color = Color::srgb(0.051, 0.051, 0.090);

/// Background entity z-depth (far behind particles).
const BACKGROUND_Z_DEPTH: f32 = -100.0;

/// Marker for the intro-phase background (despawned when entering Fidget).
#[derive(Component)]
struct IntroBackground;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Linearly interpolates between two colors in sRGB space.
///
/// This function provides smooth color transitions for act-based
/// visual changes and audio-reactive effects.
///
/// # Arguments
/// * `a` - Start color
/// * `b` - End color
/// * `t` - Interpolation factor, clamped to [0.0, 1.0]
///
/// # Returns
/// Interpolated color between `a` and `b`
///
/// # Example
/// ```ignore
/// let start = Color::srgb(0.0, 0.0, 0.0);
/// let end = Color::srgb(1.0, 1.0, 1.0);
/// let mid = color_lerp(start, end, 0.5);
/// // mid is approximately Color::srgb(0.5, 0.5, 0.5)
/// ```
#[inline]
#[must_use]
pub fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let a_srgba = a.to_srgba();
    let b_srgba = b.to_srgba();

    Color::srgba(
        a_srgba.red + (b_srgba.red - a_srgba.red) * t,
        a_srgba.green + (b_srgba.green - a_srgba.green) * t,
        a_srgba.blue + (b_srgba.blue - a_srgba.blue) * t,
        a_srgba.alpha + (b_srgba.alpha - a_srgba.alpha) * t,
    )
}

/// Adjusts the saturation of a color by a multiplier.
///
/// Uses the luminance-preserving saturation adjustment method.
/// A multiplier of 1.0 leaves the color unchanged, less than 1.0
/// desaturates toward grayscale, and greater than 1.0 increases saturation.
///
/// # Arguments
/// * `color` - The color to adjust
/// * `multiplier` - Saturation multiplier (0.0 = grayscale, 1.0 = unchanged)
///
/// # Returns
/// The color with adjusted saturation
///
/// # Example
/// ```ignore
/// let saturated = Color::srgb(1.0, 0.0, 0.0); // Pure red
/// let desaturated = adjust_saturation(saturated, 0.5);
/// // desaturated is now a muted red closer to gray
/// ```
#[inline]
#[must_use]
pub fn adjust_saturation(color: Color, multiplier: f32) -> Color {
    let srgba = color.to_srgba();

    // Calculate perceived luminance using standard coefficients
    let luminance = 0.2126 * srgba.red + 0.7152 * srgba.green + 0.0722 * srgba.blue;

    // Interpolate each channel toward the luminance (grayscale) value
    let multiplier = multiplier.max(0.0);
    let r = luminance + (srgba.red - luminance) * multiplier;
    let g = luminance + (srgba.green - luminance) * multiplier;
    let b = luminance + (srgba.blue - luminance) * multiplier;

    // Clamp to valid range
    Color::srgba(
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        srgba.alpha,
    )
}

/// Converts a hexadecimal color string to a Bevy Color.
///
/// Supports both 6-character (RGB) and 8-character (RGBA) hex strings,
/// with or without a leading '#'.
///
/// # Arguments
/// * `hex` - Hexadecimal color string (e.g., "#ff6b6b", "ff6b6b", "#ff6b6bff")
///
/// # Returns
/// The parsed color, or white if parsing fails
///
/// # Example
/// ```ignore
/// let coral = hex_to_color("#ff6b6b");
/// let coral_with_alpha = hex_to_color("#ff6b6b80");
/// ```
#[must_use]
pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    let parse_component = |s: &str| -> f32 {
        u8::from_str_radix(s, 16)
            .map(|v| v as f32 / 255.0)
            .unwrap_or(1.0)
    };

    match hex.len() {
        6 => {
            let r = parse_component(&hex[0..2]);
            let g = parse_component(&hex[2..4]);
            let b = parse_component(&hex[4..6]);
            Color::srgb(r, g, b)
        }
        8 => {
            let r = parse_component(&hex[0..2]);
            let g = parse_component(&hex[2..4]);
            let b = parse_component(&hex[4..6]);
            let a = parse_component(&hex[6..8]);
            Color::srgba(r, g, b, a)
        }
        _ => {
            warn!("Invalid hex color format: {}. Expected 6 or 8 characters.", hex);
            Color::WHITE
        }
    }
}

/// Converts a color to its hexadecimal string representation.
///
/// # Arguments
/// * `color` - The color to convert
///
/// # Returns
/// A hex string in the format "#rrggbb" (or "#rrggbbaa" if alpha < 1.0)
#[must_use]
pub fn color_to_hex(color: Color) -> String {
    let srgba = color.to_srgba();
    let r = (srgba.red * 255.0).round() as u8;
    let g = (srgba.green * 255.0).round() as u8;
    let b = (srgba.blue * 255.0).round() as u8;

    if srgba.alpha >= 0.999 {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    } else {
        let a = (srgba.alpha * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }
}

// =============================================================================
// STARTUP SYSTEMS
// =============================================================================

/// Spawns the 2D camera with post-processing pipeline configured.
///
/// Configures the camera for:
/// - 1920x1080 viewport with fixed vertical scaling
/// - Initial clear color matching Act I background
/// Sets up the main 2D camera with orthographic projection.
///
/// - Bloom settings for ethereal glow effects
///
/// # Stage
/// Startup
///
/// # Ordering
/// Must run before `setup_background` and other visual setup systems.
pub fn setup_camera(mut commands: Commands) {
    info!("Setting up camera for {}x{} viewport", VIEWPORT_WIDTH, VIEWPORT_HEIGHT);

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(INITIAL_CLEAR_COLOR),
            ..default()
        },
        OrthographicProjection {
            // Use fixed vertical scaling to maintain consistent particle sizes
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        },
        Name::new("MainCamera"),
    ));

    info!("Camera setup complete");
}

/// Spawns an immediate solid-color background for the intro phase.
/// This prevents any flash of content before the UI renders.
/// Despawned when transitioning to Fidget state.
fn setup_intro_background(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: INITIAL_CLEAR_COLOR,
            custom_size: Some(Vec2::new(VIEWPORT_WIDTH * 2.0, VIEWPORT_HEIGHT * 2.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BACKGROUND_Z_DEPTH),
        IntroBackground,
        Name::new("IntroBackground"),
    ));
}

/// Removes the intro background when entering Fidget state.
fn cleanup_intro_background(
    mut commands: Commands,
    query: Query<Entity, With<IntroBackground>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Spawns the background entity with gradient visualization.
///
/// Creates a full-screen sprite that displays the background gradient.
/// The gradient transitions between acts following the emotional arc
/// from deep void (Act I) to luminous cream (Act V).
///
/// # Stage
/// Startup
///
/// # Ordering
/// Must run after `setup_camera`.
pub fn setup_background(
    mut commands: Commands,
    background_gradients: Res<BackgroundGradients>,
    mut current_background: ResMut<CurrentBackground>,
) {
    info!("Setting up background gradient");

    // Initialize current background with Act I gradient
    let act1_gradient = &background_gradients.act_gradients[0];
    current_background.gradient_start = act1_gradient[0];
    current_background.gradient_end = act1_gradient[1];

    // Create the background sprite
    // The sprite is sized to cover the viewport plus margin
    let background_size = Vec2::new(VIEWPORT_WIDTH * 1.2, VIEWPORT_HEIGHT * 1.2);

    commands.spawn((
        Sprite {
            color: current_background.gradient_start,
            custom_size: Some(background_size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BACKGROUND_Z_DEPTH),
        BackgroundMarker,
        Name::new("Background"),
    ));

    info!(
        "Background created with initial gradient: {} -> {}",
        color_to_hex(current_background.gradient_start),
        color_to_hex(current_background.gradient_end)
    );
}

// =============================================================================
// UPDATE SYSTEMS
// =============================================================================

/// Applies act-based color modulation to all particles.
///
/// This system:
/// - Queries all particles with `ParticleVisual`
/// - Shifts `current_color` based on act progression
/// - Uses `InterpolatedActValues` for target colors and saturation
/// - Applies `saturation_multiplier` for act-specific color intensity
///
/// # Stage
/// Update
///
/// # Ordering
/// Runs after `apply_audio_to_visuals`, before `apply_pulse_effect`.
pub fn apply_act_colors(
    mut particles: Query<&mut ParticleVisual, With<Particle>>,
    interpolated_values: Res<InterpolatedActValues>,
    act_state: Res<ActState>,
) {
    // Skip processing if resources haven't changed to save performance
    if !interpolated_values.is_changed() && !act_state.is_changed() {
        return;
    }

    let saturation = interpolated_values.saturation_multiplier;
    let act_progress = act_state.act_progress;

    // Calculate target color shift based on act
    // Early acts favor cooler tones, later acts warm toward cream
    let warmth_factor = match act_state.current_act {
        crate::types::Act::Emergence => 0.0,
        crate::types::Act::Accumulation => 0.2 + act_progress * 0.1,
        crate::types::Act::Crescendo => 0.3 + act_progress * 0.2,
        crate::types::Act::Release => 0.5 + act_progress * 0.2,
        crate::types::Act::Transcendence => 0.7 + act_progress * 0.3,
    };

    // Define warmth colors for blending
    let warm_tint = Color::srgba(1.0, 0.95, 0.9, 0.0); // Subtle cream warmth

    for mut visual in particles.iter_mut() {
        // Start with the base color
        let base = visual.base_color;

        // Apply warmth shift based on act progression
        let with_warmth = if warmth_factor > 0.0 {
            color_lerp(base, warm_tint, warmth_factor * 0.3)
        } else {
            base
        };

        // Apply saturation adjustment
        let with_saturation = adjust_saturation(with_warmth, saturation);

        // Preserve the original alpha from base color
        let base_srgba = base.to_srgba();
        let result_srgba = with_saturation.to_srgba();

        visual.current_color = Color::srgba(
            result_srgba.red,
            result_srgba.green,
            result_srgba.blue,
            base_srgba.alpha * visual.opacity,
        );
    }
}

/// Updates the background gradient based on current act interpolation.
///
/// This system:
/// - Queries the `BackgroundMarker` entity
/// - Updates the sprite color with `CurrentBackground` values
/// - Applies gradient from `gradient_start` to `gradient_end`
/// - Smoothly blends colors during act transitions
///
/// # Stage
/// Update
///
/// # Ordering
/// Runs after `interpolate_act_values`.
pub fn update_background_gradient(
    mut background_query: Query<&mut Sprite, With<BackgroundMarker>>,
    current_background: Res<CurrentBackground>,
) {
    // Only update if background changed
    if !current_background.is_changed() {
        return;
    }

    let Ok(mut sprite) = background_query.get_single_mut() else {
        return;
    };

    // For a simple sprite, we use the average of the gradient colors
    // A more sophisticated approach would use a shader for true gradient rendering
    let gradient_color = color_lerp(
        current_background.gradient_start,
        current_background.gradient_end,
        0.5,
    );

    // Apply pulse intensity as a subtle brightness modulation
    let pulse = current_background.pulse_intensity;
    if pulse > 0.001 {
        let brightened = color_lerp(
            gradient_color,
            Color::WHITE,
            pulse * 0.1, // Subtle 10% max brightness boost
        );
        sprite.color = brightened;
    } else {
        sprite.color = gradient_color;
    }
}

/// Updates the camera clear color to match the current background.
///
/// This ensures the camera clear color stays synchronized with the
/// background gradient, preventing visual artifacts at the edges.
///
/// # Stage
/// Update
///
/// # Ordering
/// Runs after `update_background_gradient`.
pub fn sync_camera_clear_color(
    mut camera_query: Query<&mut Camera>,
    current_background: Res<CurrentBackground>,
) {
    if !current_background.is_changed() {
        return;
    }

    let Ok(mut camera) = camera_query.get_single_mut() else {
        return;
    };

    // Use the start of the gradient as the clear color
    camera.clear_color = ClearColorConfig::Custom(current_background.gradient_start);
}

// =============================================================================
// SYSTEM SETS
// =============================================================================

/// System set for visual startup initialization.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualStartupSet;

/// System set for visual update processing.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualUpdateSet;

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that manages visual rendering, colors, and the background.
///
/// This plugin handles:
/// - Camera setup with proper viewport configuration
/// - Background gradient entity creation
/// - Act-based color modulation for particles
/// - Background gradient updates synchronized with act state
///
/// # Systems
/// - `setup_camera` (Startup): Spawns 2D camera
/// - `setup_background` (Startup, after camera): Spawns background entity
/// - `apply_act_colors` (Update): Modulates particle colors per act
/// - `update_background_gradient` (Update): Updates background gradient
/// - `sync_camera_clear_color` (Update): Syncs camera clear color
pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        // Note: UiFont is loaded by ResourcesPlugin's load_ui_font system
        app
            // Configure startup systems with ordering - intro background prevents flash
            .add_systems(Startup, (setup_camera, setup_intro_background).chain())
            // Setup real background and cleanup intro background when entering Fidget
            .add_systems(
                OnEnter(AppState::Fidget),
                (cleanup_intro_background, setup_background).chain(),
            )
            // Configure update systems - only run during Fidget state
            .add_systems(
                Update,
                (
                    apply_act_colors,
                    update_background_gradient,
                    sync_camera_clear_color.after(update_background_gradient),
                )
                    .run_if(in_state(AppState::Fidget)),
            );

        info!("VisualPlugin initialized");
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_lerp_endpoints() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        // t=0 should return first color
        let result_0 = color_lerp(black, white, 0.0);
        let srgba = result_0.to_srgba();
        assert!((srgba.red - 0.0).abs() < 0.001);
        assert!((srgba.green - 0.0).abs() < 0.001);
        assert!((srgba.blue - 0.0).abs() < 0.001);

        // t=1 should return second color
        let result_1 = color_lerp(black, white, 1.0);
        let srgba = result_1.to_srgba();
        assert!((srgba.red - 1.0).abs() < 0.001);
        assert!((srgba.green - 1.0).abs() < 0.001);
        assert!((srgba.blue - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp_midpoint() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        let result = color_lerp(black, white, 0.5);
        let srgba = result.to_srgba();

        assert!((srgba.red - 0.5).abs() < 0.001);
        assert!((srgba.green - 0.5).abs() < 0.001);
        assert!((srgba.blue - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp_clamping() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        // t < 0 should clamp to 0
        let result_neg = color_lerp(black, white, -0.5);
        let srgba = result_neg.to_srgba();
        assert!((srgba.red - 0.0).abs() < 0.001);

        // t > 1 should clamp to 1
        let result_over = color_lerp(black, white, 1.5);
        let srgba = result_over.to_srgba();
        assert!((srgba.red - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_adjust_saturation_unchanged() {
        let red = Color::srgb(1.0, 0.0, 0.0);
        let result = adjust_saturation(red, 1.0);
        let srgba = result.to_srgba();

        assert!((srgba.red - 1.0).abs() < 0.001);
        assert!((srgba.green - 0.0).abs() < 0.001);
        assert!((srgba.blue - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_adjust_saturation_grayscale() {
        let red = Color::srgb(1.0, 0.0, 0.0);
        let result = adjust_saturation(red, 0.0);
        let srgba = result.to_srgba();

        // All channels should be equal (grayscale)
        assert!((srgba.red - srgba.green).abs() < 0.001);
        assert!((srgba.green - srgba.blue).abs() < 0.001);
    }

    #[test]
    fn test_adjust_saturation_desaturate() {
        let red = Color::srgb(1.0, 0.0, 0.0);
        let result = adjust_saturation(red, 0.5);
        let srgba = result.to_srgba();

        // Red channel should be reduced, others increased
        assert!(srgba.red < 1.0);
        assert!(srgba.green > 0.0);
        assert!(srgba.blue > 0.0);
    }

    #[test]
    fn test_hex_to_color_rgb() {
        let color = hex_to_color("#ff6b6b");
        let srgba = color.to_srgba();

        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.green - 0.42).abs() < 0.01);
        assert!((srgba.blue - 0.42).abs() < 0.01);
    }

    #[test]
    fn test_hex_to_color_rgba() {
        let color = hex_to_color("#ff6b6b80");
        let srgba = color.to_srgba();

        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.alpha - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_hex_to_color_no_hash() {
        let color = hex_to_color("ff6b6b");
        let srgba = color.to_srgba();

        assert!((srgba.red - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hex_to_color_invalid() {
        let color = hex_to_color("invalid");
        let srgba = color.to_srgba();

        // Should return white for invalid input
        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.green - 1.0).abs() < 0.01);
        assert!((srgba.blue - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hex_rgb() {
        let color = Color::srgb(1.0, 0.0, 0.0);
        let hex = color_to_hex(color);

        assert_eq!(hex, "#ff0000");
    }

    #[test]
    fn test_color_to_hex_rgba() {
        let color = Color::srgba(1.0, 0.0, 0.0, 0.5);
        let hex = color_to_hex(color);

        assert_eq!(hex, "#ff000080");
    }

    #[test]
    fn test_viewport_constants() {
        assert_eq!(VIEWPORT_WIDTH, 1920.0);
        assert_eq!(VIEWPORT_HEIGHT, 1080.0);
    }

    #[test]
    fn test_initial_clear_color() {
        let srgba = INITIAL_CLEAR_COLOR.to_srgba();

        // Should be very dark (Act I background)
        assert!(srgba.red < 0.1);
        assert!(srgba.green < 0.1);
        assert!(srgba.blue < 0.15);
    }
}
