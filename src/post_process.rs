//! Module: post_process
//! Purpose: Post-processing visual effects including bloom, chromatic aberration, vignette, and film grain
//! Dependencies: resources, bevy::prelude, bevy::core_pipeline::bloom

use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;

use crate::resources::PostProcessSettings;

// =============================================================================
// POST-PROCESSING RESOURCES
// =============================================================================

/// Settings for chromatic aberration effect.
///
/// Chromatic aberration simulates lens imperfection by separating color channels,
/// creating a subtle RGB fringing effect. Intensity increases during emotional peaks.
///
/// Note: This resource stores configuration for a custom shader effect.
/// Full implementation requires a custom render pipeline or bevy_post_process crate.
#[derive(Resource, Debug, Clone)]
pub struct ChromaticAberrationSettings {
    /// Strength of the color channel separation (0.0 = none, 1.0 = maximum)
    pub strength: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Default for ChromaticAberrationSettings {
    fn default() -> Self {
        Self {
            strength: 0.0,
            enabled: true,
        }
    }
}

/// Settings for vignette effect.
///
/// Vignette darkens the edges of the screen, drawing focus to the center.
/// Intensity is stronger in early acts and dissolves by Act V for a feeling
/// of openness and transcendence.
///
/// Note: This resource stores configuration for a custom shader effect.
/// Full implementation requires a custom render pipeline or bevy_post_process crate.
#[derive(Resource, Debug, Clone)]
pub struct VignetteSettings {
    /// Darkness intensity at the edges (0.0 = none, 1.0 = fully darkened)
    pub intensity: f32,
    /// Smoothness of the vignette falloff (higher = softer edge)
    pub smoothness: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self {
            intensity: 0.3,
            smoothness: 0.5,
            enabled: true,
        }
    }
}

/// Settings for film grain effect.
///
/// Film grain adds organic noise texture to the visuals, creating a
/// cinematic quality and preventing banding in gradients.
///
/// Note: This resource stores configuration for a custom shader effect.
/// Full implementation requires a custom render pipeline or bevy_post_process crate.
#[derive(Resource, Debug, Clone)]
pub struct FilmGrainSettings {
    /// Amount of grain noise (0.0 = none, 1.0 = heavy grain)
    pub amount: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Default for FilmGrainSettings {
    fn default() -> Self {
        Self {
            amount: 0.02,
            enabled: true,
        }
    }
}

// =============================================================================
// CONSTANTS
// =============================================================================

/// Default bloom intensity for the ethereal glow effect.
const _DEFAULT_BLOOM_INTENSITY: f32 = 0.3;

/// Maximum bloom intensity during peak moments.
const MAX_BLOOM_INTENSITY: f32 = 1.0;

/// Default bloom low frequency boost (softness).
const DEFAULT_BLOOM_LF_BOOST: f32 = 0.7;

/// Default bloom low frequency boost curvature.
const DEFAULT_BLOOM_LF_BOOST_CURVATURE: f32 = 0.95;

/// Default bloom high pass frequency (controls brightness threshold behavior).
const DEFAULT_BLOOM_HIGH_PASS_FREQUENCY: f32 = 1.0;

/// Maximum chromatic aberration strength.
const MAX_CHROMATIC_ABERRATION: f32 = 0.015;

/// Maximum vignette intensity.
const MAX_VIGNETTE_INTENSITY: f32 = 0.6;

// =============================================================================
// SYSTEMS
// =============================================================================

/// Updates Bevy's BloomSettings component on the camera based on PostProcessSettings.
///
/// This system:
/// - Reads `PostProcessSettings.bloom_threshold`, `bloom_intensity`, and `bloom_radius`
/// - Updates the `Bloom` component on the camera entity
/// - Configures bloom for the ethereal glow characteristic of Chromatic Elegy
///
/// # Stage
/// PostUpdate
///
/// # Ordering
/// Runs before `apply_post_process_chain`.
///
/// # Note
/// Bevy 0.13+ has built-in bloom support via the `Bloom` component.
pub fn update_bloom(
    post_process_settings: Res<PostProcessSettings>,
    mut camera_query: Query<&mut Bloom, With<Camera>>,
) {
    // Only update if settings changed
    if !post_process_settings.is_changed() {
        return;
    }

    let Ok(mut bloom) = camera_query.get_single_mut() else {
        // Camera may not have Bloom component yet; this is handled in setup
        return;
    };

    // Map PostProcessSettings to Bevy's Bloom parameters
    // Clamp intensity to reasonable range
    let intensity = post_process_settings
        .bloom_intensity
        .clamp(0.0, MAX_BLOOM_INTENSITY);

    // Update bloom settings
    bloom.intensity = intensity;

    // Map bloom_radius to Bevy's low_frequency_boost for soft, ethereal glow
    // Higher radius = more diffuse bloom
    let radius_factor = (post_process_settings.bloom_radius / 16.0).clamp(0.0, 1.0);
    bloom.low_frequency_boost = DEFAULT_BLOOM_LF_BOOST * (1.0 + radius_factor * 0.5);
    bloom.low_frequency_boost_curvature = DEFAULT_BLOOM_LF_BOOST_CURVATURE;

    // High pass frequency affects bloom threshold behavior
    // Lower threshold = more pixels bloom
    let threshold_factor = 1.0 - post_process_settings.bloom_threshold.clamp(0.0, 1.0);
    bloom.high_pass_frequency = DEFAULT_BLOOM_HIGH_PASS_FREQUENCY * (1.0 + threshold_factor);

    debug!(
        "Bloom updated: intensity={:.3}, low_freq_boost={:.3}, high_pass={:.3}",
        bloom.intensity, bloom.low_frequency_boost, bloom.high_pass_frequency
    );
}

/// Updates chromatic aberration settings based on PostProcessSettings.
///
/// This system:
/// - Reads `PostProcessSettings.chromatic_aberration_strength`
/// - Updates `ChromaticAberrationSettings` resource
/// - Prepares configuration for custom post-process shader
///
/// # Stage
/// PostUpdate
///
/// # Ordering
/// Runs before `apply_post_process_chain`.
///
/// # Note
/// This is a placeholder for custom shader integration. The actual chromatic
/// aberration effect requires a custom render pipeline or third-party crate.
pub fn update_chromatic_aberration(
    post_process_settings: Res<PostProcessSettings>,
    mut chromatic_settings: ResMut<ChromaticAberrationSettings>,
) {
    // Only update if settings changed
    if !post_process_settings.is_changed() {
        return;
    }

    // Clamp strength to safe range
    let strength = post_process_settings
        .chromatic_aberration_strength
        .clamp(0.0, MAX_CHROMATIC_ABERRATION);

    chromatic_settings.strength = strength;
    chromatic_settings.enabled = strength > 0.0001;

    debug!(
        "Chromatic aberration updated: strength={:.4}, enabled={}",
        chromatic_settings.strength, chromatic_settings.enabled
    );
}

/// Updates vignette settings based on PostProcessSettings.
///
/// This system:
/// - Reads `PostProcessSettings.vignette_intensity`
/// - Updates `VignetteSettings` resource
/// - Prepares configuration for custom post-process shader
///
/// # Stage
/// PostUpdate
///
/// # Ordering
/// Runs before `apply_post_process_chain`.
///
/// # Note
/// Vignette intensity progression:
/// - Acts I-III: Stronger vignette (0.3-0.4) for intimate focus
/// - Act IV: Gradually reducing (0.2-0.1)
/// - Act V: Minimal or none for expansive feeling
///
/// This is a placeholder for custom shader integration.
pub fn update_vignette(
    post_process_settings: Res<PostProcessSettings>,
    mut vignette_settings: ResMut<VignetteSettings>,
) {
    // Only update if settings changed
    if !post_process_settings.is_changed() {
        return;
    }

    // Clamp intensity to safe range
    let intensity = post_process_settings
        .vignette_intensity
        .clamp(0.0, MAX_VIGNETTE_INTENSITY);

    vignette_settings.intensity = intensity;
    vignette_settings.enabled = intensity > 0.01;

    // Smoothness inversely related to intensity for natural feel
    // Higher intensity = sharper edge, lower intensity = softer fade
    vignette_settings.smoothness = 0.3 + (1.0 - intensity / MAX_VIGNETTE_INTENSITY) * 0.5;

    debug!(
        "Vignette updated: intensity={:.3}, smoothness={:.3}, enabled={}",
        vignette_settings.intensity, vignette_settings.smoothness, vignette_settings.enabled
    );
}

/// Updates film grain settings based on PostProcessSettings.
///
/// This system:
/// - Reads `PostProcessSettings.film_grain_amount`
/// - Updates `FilmGrainSettings` resource
/// - Prepares configuration for custom post-process shader
///
/// # Stage
/// PostUpdate
///
/// # Ordering
/// Runs before `apply_post_process_chain`.
///
/// # Note
/// Film grain is kept subtle (default 0.02) to add organic texture
/// without being distracting. Amount may increase slightly during
/// darker scenes for cinematic effect.
pub fn update_film_grain(
    post_process_settings: Res<PostProcessSettings>,
    mut film_grain_settings: ResMut<FilmGrainSettings>,
) {
    // Only update if settings changed
    if !post_process_settings.is_changed() {
        return;
    }

    // Clamp amount to reasonable range
    let amount = post_process_settings.film_grain_amount.clamp(0.0, 0.1);

    film_grain_settings.amount = amount;
    film_grain_settings.enabled = amount > 0.001;

    debug!(
        "Film grain updated: amount={:.3}, enabled={}",
        film_grain_settings.amount, film_grain_settings.enabled
    );
}

/// Orchestrates the post-processing chain and logs current settings.
///
/// This system:
/// - Runs after all individual post-process update systems
/// - Ensures all settings are synchronized
/// - Logs current post-processing state for debugging
///
/// # Stage
/// PostUpdate
///
/// # Ordering
/// Runs after `update_bloom`, `update_chromatic_aberration`, `update_vignette`.
///
/// # Note
/// This is a coordination point for the post-processing pipeline. In a full
/// implementation, this would orchestrate render passes for custom effects.
/// Currently serves as a sync point and debug logging utility.
pub fn apply_post_process_chain(
    post_process_settings: Res<PostProcessSettings>,
    chromatic_settings: Res<ChromaticAberrationSettings>,
    vignette_settings: Res<VignetteSettings>,
    film_grain_settings: Res<FilmGrainSettings>,
) {
    // Only log when any settings changed
    let any_changed = post_process_settings.is_changed()
        || chromatic_settings.is_changed()
        || vignette_settings.is_changed()
        || film_grain_settings.is_changed();

    if !any_changed {
        return;
    }

    // Log comprehensive post-processing state for debugging
    debug!(
        "Post-processing chain synced:\n\
         - Bloom: threshold={:.2}, intensity={:.2}, radius={:.1}\n\
         - Chromatic Aberration: strength={:.4}, enabled={}\n\
         - Vignette: intensity={:.2}, smoothness={:.2}, enabled={}\n\
         - Film Grain: amount={:.3}, enabled={}",
        post_process_settings.bloom_threshold,
        post_process_settings.bloom_intensity,
        post_process_settings.bloom_radius,
        chromatic_settings.strength,
        chromatic_settings.enabled,
        vignette_settings.intensity,
        vignette_settings.smoothness,
        vignette_settings.enabled,
        film_grain_settings.amount,
        film_grain_settings.enabled,
    );
}

// =============================================================================
// STARTUP SYSTEMS
// =============================================================================

/// Sets up the Bloom component on the camera for post-processing.
///
/// This system adds the `Bloom` component to the main camera if it doesn't
/// already have one. Called during startup to ensure bloom is available.
///
/// # Stage
/// Startup (runs after camera setup)
pub fn setup_bloom(
    mut commands: Commands,
    post_process_settings: Res<PostProcessSettings>,
    camera_query: Query<Entity, (With<Camera2d>, Without<Bloom>)>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        // Camera may already have bloom or doesn't exist yet
        return;
    };

    // Create initial bloom settings
    let bloom = Bloom {
        intensity: post_process_settings.bloom_intensity.clamp(0.0, MAX_BLOOM_INTENSITY),
        low_frequency_boost: DEFAULT_BLOOM_LF_BOOST,
        low_frequency_boost_curvature: DEFAULT_BLOOM_LF_BOOST_CURVATURE,
        high_pass_frequency: DEFAULT_BLOOM_HIGH_PASS_FREQUENCY,
        // Use default compositing mode for natural-looking bloom
        composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::EnergyConserving,
        ..default()
    };

    commands.entity(camera_entity).insert(bloom);

    info!(
        "Bloom component added to camera: intensity={:.2}, low_freq_boost={:.2}",
        post_process_settings.bloom_intensity, DEFAULT_BLOOM_LF_BOOST
    );
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Calculates recommended post-process settings for a given act progression.
///
/// This helper function provides act-specific post-processing values:
/// - Acts I-II: Subtle bloom, stronger vignette
/// - Act III: Peak bloom intensity, chromatic aberration appears
/// - Act IV: High bloom, chromatic aberration peaks, vignette reduces
/// - Act V: Soft ethereal bloom, minimal vignette, subtle aberration
///
/// # Arguments
/// * `act_index` - Zero-based act index (0 = Emergence, 4 = Transcendence)
/// * `act_progress` - Progress within the act (0.0 to 1.0)
///
/// # Returns
/// Tuple of (bloom_intensity, chromatic_aberration_strength, vignette_intensity)
#[must_use]
pub fn calculate_act_post_process_values(act_index: usize, act_progress: f32) -> (f32, f32, f32) {
    let progress = act_progress.clamp(0.0, 1.0);

    match act_index {
        // Act I: Emergence - subtle, intimate
        0 => {
            let bloom = 0.15 + progress * 0.1;
            let chromatic = 0.0;
            let vignette = 0.35 - progress * 0.05;
            (bloom, chromatic, vignette)
        }
        // Act II: Accumulation - building intensity
        1 => {
            let bloom = 0.25 + progress * 0.15;
            let chromatic = progress * 0.002;
            let vignette = 0.30 - progress * 0.05;
            (bloom, chromatic, vignette)
        }
        // Act III: Crescendo - peak intensity
        2 => {
            let bloom = 0.4 + progress * 0.3;
            let chromatic = 0.002 + progress * 0.008;
            let vignette = 0.25 - progress * 0.05;
            (bloom, chromatic, vignette)
        }
        // Act IV: Release - emotional outpouring
        3 => {
            let bloom = 0.6 + progress * 0.2;
            let chromatic = 0.01 - progress * 0.005;
            let vignette = 0.20 - progress * 0.15;
            (bloom, chromatic, vignette)
        }
        // Act V: Transcendence - ethereal dissolution
        4 | _ => {
            let bloom = 0.5 - progress * 0.2;
            let chromatic = 0.005 - progress * 0.005;
            let vignette = 0.05 - progress * 0.05;
            (bloom.max(0.3), chromatic.max(0.0), vignette.max(0.0))
        }
    }
}

/// Smoothly interpolates post-process values during act transitions.
///
/// # Arguments
/// * `from` - Starting values (bloom, chromatic, vignette)
/// * `to` - Target values (bloom, chromatic, vignette)
/// * `t` - Interpolation factor (0.0 to 1.0)
///
/// # Returns
/// Interpolated values tuple
#[inline]
#[must_use]
pub fn lerp_post_process_values(
    from: (f32, f32, f32),
    to: (f32, f32, f32),
    t: f32,
) -> (f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        from.0 + (to.0 - from.0) * t,
        from.1 + (to.1 - from.1) * t,
        from.2 + (to.2 - from.2) * t,
    )
}

// =============================================================================
// SYSTEM SETS
// =============================================================================

/// System set for post-processing update systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostProcessUpdateSet;

/// System set for post-processing startup systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostProcessStartupSet;

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that manages post-processing visual effects.
///
/// This plugin handles:
/// - Bloom effect via Bevy's built-in `Bloom` component
/// - Chromatic aberration configuration (placeholder for custom shader)
/// - Vignette effect configuration (placeholder for custom shader)
/// - Film grain configuration (placeholder for custom shader)
///
/// # Systems
///
/// ## Startup
/// - `setup_bloom`: Adds Bloom component to camera
///
/// ## PostUpdate (ordered)
/// - `update_bloom`: Updates Bevy's BloomSettings
/// - `update_chromatic_aberration`: Updates ChromaticAberrationSettings
/// - `update_vignette`: Updates VignetteSettings
/// - `update_film_grain`: Updates FilmGrainSettings
/// - `apply_post_process_chain`: Orchestrates and logs post-processing state
///
/// # Note
/// Full post-processing (chromatic aberration, vignette, film grain) requires
/// custom render pipelines. This module sets up the configuration resources;
/// actual shader integration should be done via Bevy's render graph or a
/// third-party crate like `bevy_post_process`.
///
/// # Example
/// ```ignore
/// use bevy::prelude::*;
/// use chromatic_elegy::post_process::PostProcessPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(PostProcessPlugin)
///         .run();
/// }
/// ```
pub struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        // Register post-processing resources
        app.init_resource::<ChromaticAberrationSettings>()
            .init_resource::<VignetteSettings>()
            .init_resource::<FilmGrainSettings>();

        // Add startup systems
        // setup_bloom runs after the camera is created (in PostStartup to ensure camera exists)
        app.add_systems(PostStartup, setup_bloom);

        // Add PostUpdate systems with proper ordering
        // update_bloom, update_chromatic_aberration, update_vignette run in parallel
        // apply_post_process_chain runs after all of them
        app.add_systems(
            PostUpdate,
            (
                update_bloom,
                update_chromatic_aberration,
                update_vignette,
                update_film_grain,
            )
                .in_set(PostProcessUpdateSet),
        )
        .add_systems(
            PostUpdate,
            apply_post_process_chain.after(PostProcessUpdateSet),
        );

        info!("PostProcessPlugin initialized");
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromatic_aberration_settings_default() {
        let settings = ChromaticAberrationSettings::default();
        assert_eq!(settings.strength, 0.0);
        assert!(settings.enabled);
    }

    #[test]
    fn test_vignette_settings_default() {
        let settings = VignetteSettings::default();
        assert_eq!(settings.intensity, 0.3);
        assert_eq!(settings.smoothness, 0.5);
        assert!(settings.enabled);
    }

    #[test]
    fn test_film_grain_settings_default() {
        let settings = FilmGrainSettings::default();
        assert_eq!(settings.amount, 0.02);
        assert!(settings.enabled);
    }

    #[test]
    fn test_act_post_process_values_emergence() {
        let (bloom, chromatic, vignette) = calculate_act_post_process_values(0, 0.0);
        assert!((bloom - 0.15).abs() < 0.001);
        assert_eq!(chromatic, 0.0);
        assert!((vignette - 0.35).abs() < 0.001);
    }

    #[test]
    fn test_act_post_process_values_crescendo_peak() {
        let (bloom, chromatic, vignette) = calculate_act_post_process_values(2, 1.0);
        assert!(bloom > 0.6); // High bloom at crescendo peak
        assert!(chromatic > 0.005); // Chromatic aberration present
        assert!(vignette < 0.25); // Vignette reduced
    }

    #[test]
    fn test_act_post_process_values_transcendence() {
        let (bloom, chromatic, vignette) = calculate_act_post_process_values(4, 1.0);
        assert!(bloom >= 0.3); // Minimum bloom maintained
        assert_eq!(chromatic, 0.0); // No chromatic aberration at end
        assert_eq!(vignette, 0.0); // No vignette at transcendence end
    }

    #[test]
    fn test_act_post_process_values_progression() {
        // Bloom should generally increase through Acts I-IV
        let (bloom_1, _, _) = calculate_act_post_process_values(0, 0.5);
        let (bloom_2, _, _) = calculate_act_post_process_values(1, 0.5);
        let (bloom_3, _, _) = calculate_act_post_process_values(2, 0.5);

        assert!(bloom_2 > bloom_1);
        assert!(bloom_3 > bloom_2);
    }

    #[test]
    fn test_lerp_post_process_values_endpoints() {
        let from = (0.0, 0.0, 0.0);
        let to = (1.0, 0.5, 0.25);

        let result_0 = lerp_post_process_values(from, to, 0.0);
        assert_eq!(result_0, from);

        let result_1 = lerp_post_process_values(from, to, 1.0);
        assert_eq!(result_1, to);
    }

    #[test]
    fn test_lerp_post_process_values_midpoint() {
        let from = (0.0, 0.0, 0.0);
        let to = (1.0, 0.5, 0.25);

        let result = lerp_post_process_values(from, to, 0.5);
        assert!((result.0 - 0.5).abs() < 0.001);
        assert!((result.1 - 0.25).abs() < 0.001);
        assert!((result.2 - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_lerp_post_process_values_clamping() {
        let from = (0.0, 0.0, 0.0);
        let to = (1.0, 0.5, 0.25);

        // t < 0 should clamp to 0
        let result_neg = lerp_post_process_values(from, to, -0.5);
        assert_eq!(result_neg, from);

        // t > 1 should clamp to 1
        let result_over = lerp_post_process_values(from, to, 1.5);
        assert_eq!(result_over, to);
    }

    #[test]
    fn test_bloom_constants() {
        assert_eq!(DEFAULT_BLOOM_INTENSITY, 0.3);
        assert_eq!(MAX_BLOOM_INTENSITY, 1.0);
        assert!(DEFAULT_BLOOM_LF_BOOST > 0.0);
    }

    #[test]
    fn test_max_constants_are_reasonable() {
        assert!(MAX_CHROMATIC_ABERRATION > 0.0);
        assert!(MAX_CHROMATIC_ABERRATION < 0.1); // Should be subtle
        assert!(MAX_VIGNETTE_INTENSITY > 0.0);
        assert!(MAX_VIGNETTE_INTENSITY <= 1.0);
    }

    #[test]
    fn test_vignette_progression_decreases() {
        // Vignette should decrease from Act I to Act V
        let (_, _, vignette_1) = calculate_act_post_process_values(0, 0.5);
        let (_, _, vignette_5) = calculate_act_post_process_values(4, 0.5);

        assert!(vignette_1 > vignette_5);
    }
}
