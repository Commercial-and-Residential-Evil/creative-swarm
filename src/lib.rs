//! # Whirled Peas Visualiser
//!
//! A wordless poem in light and sound - an interactive visual experience
//! built with Bevy ECS.
//!
//! ## Overview
//!
//! Whirled Peas Visualiser is a 15-minute generative art experience that unfolds
//! through five acts, each with distinct visual characteristics and
//! interaction modes. Pea particles respond to audio input and user interaction,
//! creating an evolving tapestry of color and motion.
//!
//! ## Architecture
//!
//! The application is organized into modular plugins:
//!
//! - [`ResourcesPlugin`]: Global state and configuration
//! - [`ComponentsPlugin`]: ECS component registration
//! - [`VisualPlugin`]: Camera, colors, and background rendering
//! - [`ActManagementPlugin`]: Five-act narrative structure
//! - [`ParticlePlugin`]: Particle lifecycle, pooling, and motion
//! - [`TrailPlugin`]: Particle trail rendering with decay
//! - [`AudioReactivePlugin`]: Audio analysis and visual synchronization
//! - [`InteractionPlugin`]: Mouse and keyboard input handling
//! - [`PostProcessPlugin`]: Bloom, vignette, and chromatic aberration
//!
//! ## Usage
//!
//! Add the main plugin to your Bevy app:
//!
//! ```ignore
//! use bevy::prelude::*;
//! use whirled_peas::WhirledPeasPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(WhirledPeasPlugin)
//!         .run();
//! }
//! ```
//!
//! ## The Five Acts
//!
//! 1. **Emergence** (0-3 min): Sparse peas drift slowly, touch creates new peas
//! 2. **Accumulation** (3-7 min): Peas gather around viewer presence
//! 3. **Crescendo** (7-10 min): Peak density and intensity, gravitational swarms
//! 4. **Release** (10-13 min): Peas disperse upward, bittersweet transition
//! 5. **Transcendence** (13-15 min): Weightless luminosity, peaceful dissolution

use bevy::prelude::*;

// =============================================================================
// MODULE DECLARATIONS
// =============================================================================

/// Core type definitions: enums, constants, and foundational types.
pub mod types;

/// ECS component definitions for particles, trails, and interactions.
pub mod components;

/// Global ECS resources for application state and configuration.
pub mod resources;

/// Five-act narrative structure and state transitions.
pub mod act_management;

/// Particle lifecycle, object pooling, motion, and rendering.
pub mod particle;

/// Particle trail system with exponential opacity decay.
pub mod trail;

/// Audio analysis and visual synchronization systems.
pub mod audio_reactive;

/// Mouse and keyboard input handling per act.
pub mod interaction;

/// Camera setup, color management, and background rendering.
pub mod visual;

/// Post-processing effects: bloom, chromatic aberration, vignette, film grain.
pub mod post_process;

/// Intro sequence with Bauhaus-styled splash screens.
pub mod intro;

// =============================================================================
// RE-EXPORTS
// =============================================================================

/// Re-export all types for convenient access.
pub use types::{
    Act, BeatStrength, FalloffType, FrequencyBand, InteractionMode, ParticleBehaviorType,
    SpawnSource, ACT_BOUNDARIES_SECONDS, TOTAL_DURATION_SECONDS, TRANSITION_DURATION_MS,
};

/// Re-export key resources.
pub use resources::{
    ActState, ActTimings, AmbientAudioState, AudioAnalysis, AudioVisualMapping, BackgroundGradients,
    ColorPalette, CurrentBackground, CurrentInteractionMode, InteractionConfig, InterpolatedActValues,
    MotionTiming, MouseState, ParticlePool, ParticleSpawnQueue, ParticleSpawnRequest,
    PerformanceMetrics, PostProcessSettings, ResourcesPlugin,
};

/// Re-export key components.
pub use components::{
    Attractable, AudioReactive, BackgroundMarker, MouseInfluence, Particle, ParticleBehavior,
    ParticleBundle, ParticleMotion, ParticleState, ParticleVisual, PulseResponder, Spawnable,
    Trail, TrailRenderer, TrailSegment, ComponentsPlugin,
};

/// Re-export plugins for selective use.
pub use act_management::ActManagementPlugin;
pub use audio_reactive::AudioReactivePlugin;
pub use interaction::InteractionPlugin;
pub use intro::{AppState, IntroPlugin};
pub use particle::ParticlePlugin;
pub use post_process::PostProcessPlugin;
pub use trail::TrailPlugin;
pub use visual::VisualPlugin;

// =============================================================================
// MAIN PLUGIN
// =============================================================================

/// Main plugin for Whirled Peas Visualiser.
///
/// Registers all sub-plugins in the correct order to ensure proper
/// initialization and system scheduling. This is the primary entry
/// point for integrating the experience into a Bevy application.
///
/// # Plugin Registration Order
///
/// Plugins are registered in dependency order:
/// 1. Resources - Global state initialization
/// 2. Components - Component type registration
/// 3. Visual - Camera and rendering setup
/// 4. Act Management - Narrative structure
/// 5. Particle - Core particle systems
/// 6. Trail - Particle trail rendering
/// 7. Audio Reactive - Audio-visual synchronization
/// 8. Interaction - User input handling
/// 9. Post Process - Visual effects
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use whirled_peas::WhirledPeasPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(WhirledPeasPlugin)
///     .run();
/// ```
pub struct WhirledPeasPlugin;

impl Plugin for WhirledPeasPlugin {
    fn build(&self, app: &mut App) {
        // Register all sub-plugins in correct dependency order
        app.add_plugins((
            ResourcesPlugin,
            ComponentsPlugin,
            IntroPlugin,      // Intro first - manages AppState
            VisualPlugin,
            ActManagementPlugin,
            ParticlePlugin,
            TrailPlugin,
            AudioReactivePlugin,
            InteractionPlugin,
            PostProcessPlugin,
        ));

        info!("Whirled Peas Visualiser initialized - a wordless poem in light and sound");
    }
}

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/// Returns the current act for a given elapsed time in seconds.
///
/// This is a convenience function that wraps [`Act::from_elapsed_seconds`].
///
/// # Arguments
///
/// * `elapsed_seconds` - Total elapsed time since the experience began
///
/// # Returns
///
/// The [`Act`] corresponding to the given time
#[inline]
#[must_use]
pub fn act_at_time(elapsed_seconds: f32) -> Act {
    Act::from_elapsed_seconds(elapsed_seconds)
}

/// Returns the progress (0.0 to 1.0) within a specific act for a given time.
///
/// # Arguments
///
/// * `elapsed_seconds` - Total elapsed time since the experience began
///
/// # Returns
///
/// Progress within the current act as a value from 0.0 to 1.0
#[must_use]
pub fn act_progress_at_time(elapsed_seconds: f32) -> f32 {
    let act = Act::from_elapsed_seconds(elapsed_seconds);
    let act_start = act.start_seconds();
    let act_duration = act.duration_seconds();

    if act_duration <= 0.0 {
        return 1.0;
    }

    ((elapsed_seconds - act_start) / act_duration).clamp(0.0, 1.0)
}

/// Returns the overall progress (0.0 to 1.0) through the entire experience.
///
/// # Arguments
///
/// * `elapsed_seconds` - Total elapsed time since the experience began
///
/// # Returns
///
/// Progress through the entire 15-minute experience
#[inline]
#[must_use]
pub fn total_progress(elapsed_seconds: f32) -> f32 {
    (elapsed_seconds / TOTAL_DURATION_SECONDS).clamp(0.0, 1.0)
}

// =============================================================================
// ANDROID ENTRY POINT
// =============================================================================

/// Android entry point.
///
/// This function is called by the Android GameActivity when the app starts.
/// The `#[bevy_main]` attribute generates the `android_main` symbol that
/// the native activity loader expects.
///
/// On non-Android platforms, this function is not compiled.
#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "Whirled Peas".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(WhirledPeasPlugin)
        .run();
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_at_time() {
        assert_eq!(act_at_time(0.0), Act::Emergence);
        assert_eq!(act_at_time(90.0), Act::Emergence);
        assert_eq!(act_at_time(180.0), Act::Accumulation);
        assert_eq!(act_at_time(300.0), Act::Accumulation);
        assert_eq!(act_at_time(420.0), Act::Crescendo);
        assert_eq!(act_at_time(600.0), Act::Release);
        assert_eq!(act_at_time(780.0), Act::Transcendence);
        assert_eq!(act_at_time(900.0), Act::Transcendence);
    }

    #[test]
    fn test_act_progress_at_time() {
        // Start of emergence
        let progress = act_progress_at_time(0.0);
        assert!((progress - 0.0).abs() < 0.001);

        // Middle of emergence (90 seconds into 180-second act)
        let progress = act_progress_at_time(90.0);
        assert!((progress - 0.5).abs() < 0.001);

        // End of emergence
        let progress = act_progress_at_time(179.9);
        assert!(progress > 0.99);
    }

    #[test]
    fn test_total_progress() {
        assert!((total_progress(0.0) - 0.0).abs() < 0.001);
        assert!((total_progress(450.0) - 0.5).abs() < 0.001);
        assert!((total_progress(900.0) - 1.0).abs() < 0.001);
        assert!((total_progress(1000.0) - 1.0).abs() < 0.001); // Clamped
    }

    #[test]
    fn test_plugin_builds_without_panic() {
        // Verify the plugin struct exists and can be instantiated
        let _plugin = WhirledPeasPlugin;
    }
}
