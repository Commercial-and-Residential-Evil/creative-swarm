//! Module: resources
//! Purpose: Global ECS resources for Chromatic Elegy application state
//! Dependencies: types, bevy::prelude

use bevy::prelude::*;

use crate::types::{
    Act, BeatStrength, FalloffType, InteractionMode, ParticleBehaviorType, SpawnSource,
};

// =============================================================================
// APPLICATION STATE RESOURCES
// =============================================================================

/// Tracks the current act in the five-act narrative structure.
///
/// The experience unfolds over 15 minutes through five distinct acts,
/// each with unique visual characteristics and interaction modes.
#[derive(Resource, Debug, Clone)]
pub struct ActState {
    /// Current act in the narrative progression
    pub current_act: Act,
    /// Progress within current act (0.0 - 1.0)
    pub act_progress: f32,
    /// Total elapsed time since experience began
    pub total_elapsed_seconds: f32,
    /// Whether currently transitioning between acts
    pub is_transitioning: bool,
    /// Progress of act transition (0.0 - 1.0)
    pub transition_progress: f32,
}

impl Default for ActState {
    fn default() -> Self {
        Self {
            current_act: Act::Emergence,
            act_progress: 0.0,
            total_elapsed_seconds: 0.0,
            is_transitioning: false,
            transition_progress: 0.0,
        }
    }
}

/// Defines timing boundaries for each act in seconds.
///
/// Act I (Emergence): 0-3 minutes
/// Act II (Accumulation): 3-7 minutes
/// Act III (Crescendo): 7-10 minutes
/// Act IV (Release): 10-13 minutes
/// Act V (Transcendence): 13-15 minutes
#[derive(Resource, Debug, Clone)]
pub struct ActTimings {
    /// Boundary timestamps in seconds: [start, act2, act3, act4, act5, end]
    pub act_boundaries_seconds: [f32; 6],
    /// Duration of smooth transitions between acts in milliseconds
    pub transition_duration_ms: f32,
}

impl Default for ActTimings {
    fn default() -> Self {
        Self {
            // [0, 180, 420, 600, 780, 900] = [0, 3min, 7min, 10min, 13min, 15min]
            act_boundaries_seconds: [0.0, 180.0, 420.0, 600.0, 780.0, 900.0],
            transition_duration_ms: 2000.0,
        }
    }
}

/// Interpolated values derived from current act state for smooth transitions.
///
/// These values are recalculated each frame during act transitions to provide
/// smooth blending between act-specific parameters.
#[derive(Resource, Debug, Clone)]
pub struct InterpolatedActValues {
    /// Starting color of current background gradient
    pub background_color_start: Color,
    /// Ending color of current background gradient
    pub background_color_end: Color,
    /// Current particle behavior mode
    pub particle_behavior: ParticleBehaviorType,
    /// Current interaction mode
    pub interaction_mode: InteractionMode,
    /// Saturation multiplier for particle colors
    pub saturation_multiplier: f32,
    /// Target particle density for current act
    pub density_target: f32,
}

impl Default for InterpolatedActValues {
    fn default() -> Self {
        Self {
            // Deep navy blue - emergence from darkness
            background_color_start: Color::srgb(0.102, 0.102, 0.180),
            background_color_end: Color::srgb(0.051, 0.051, 0.102),
            particle_behavior: ParticleBehaviorType::Drift,
            interaction_mode: InteractionMode::Paint,
            saturation_multiplier: 1.0,
            density_target: 0.3,
        }
    }
}

// =============================================================================
// COLOR RESOURCES
// =============================================================================

/// The master color palette defining all thematic colors for the experience.
///
/// Colors progress from dark, muted tones in early acts to bright, luminous
/// hues in later acts, representing an emotional journey through grief to transcendence.
#[derive(Resource, Debug, Clone)]
pub struct ColorPalette {
    /// Deep navy blue - initial emergence from void (#1a1a2e)
    pub primary_initial: Color,
    /// Rich crimson - emotional midpoint (#c41e3a)
    pub primary_midpoint: Color,
    /// Warm cream - final transcendence (#fdf6f0)
    pub primary_final: Color,
    /// Cool charcoal - secondary cool tone (#2d3436)
    pub secondary_cool: Color,
    /// Warm sand - secondary warm tone (#d4a574)
    pub secondary_warm: Color,
    /// Ethereal off-white - secondary ethereal tone (#e8d5c4)
    pub secondary_ethereal: Color,
    /// Spark coral - accent for energy (#ff6b6b)
    pub accent_spark: Color,
    /// Deep violet - accent for depth (#6c5ce7)
    pub accent_deep: Color,
    /// Hope yellow - accent for transcendence (#ffeaa7)
    pub accent_hope: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            // Primary palette - journey from dark to light
            primary_initial: Color::srgb(0.102, 0.102, 0.180),   // #1a1a2e
            primary_midpoint: Color::srgb(0.769, 0.118, 0.227),  // #c41e3a
            primary_final: Color::srgb(0.992, 0.965, 0.941),     // #fdf6f0

            // Secondary palette - supporting tones
            secondary_cool: Color::srgb(0.176, 0.204, 0.212),    // #2d3436
            secondary_warm: Color::srgb(0.831, 0.647, 0.455),    // #d4a574
            secondary_ethereal: Color::srgb(0.910, 0.835, 0.769), // #e8d5c4

            // Accent palette - emotional highlights
            accent_spark: Color::srgb(1.0, 0.420, 0.420),        // #ff6b6b
            accent_deep: Color::srgb(0.424, 0.361, 0.906),       // #6c5ce7
            accent_hope: Color::srgb(1.0, 0.918, 0.655),         // #ffeaa7
        }
    }
}

/// Pre-computed background gradient pairs for each act.
///
/// Each act has a distinct gradient representing its emotional character.
#[derive(Resource, Debug, Clone)]
pub struct BackgroundGradients {
    /// Gradient color pairs for each of the five acts: [[start, end]; 5]
    pub act_gradients: [[Color; 2]; 5],
}

impl Default for BackgroundGradients {
    fn default() -> Self {
        Self {
            act_gradients: [
                // Act I: Emergence - deep void to faint blue
                [
                    Color::srgb(0.051, 0.051, 0.090),
                    Color::srgb(0.102, 0.102, 0.180),
                ],
                // Act II: Accumulation - twilight blue to violet
                [
                    Color::srgb(0.102, 0.102, 0.180),
                    Color::srgb(0.176, 0.141, 0.251),
                ],
                // Act III: Crescendo - violet to crimson
                [
                    Color::srgb(0.176, 0.141, 0.251),
                    Color::srgb(0.384, 0.118, 0.196),
                ],
                // Act IV: Release - crimson to warm amber
                [
                    Color::srgb(0.384, 0.118, 0.196),
                    Color::srgb(0.584, 0.365, 0.259),
                ],
                // Act V: Transcendence - warm amber to luminous cream
                [
                    Color::srgb(0.584, 0.365, 0.259),
                    Color::srgb(0.910, 0.835, 0.769),
                ],
            ],
        }
    }
}

/// Current background state with animated pulse effect.
///
/// Updated each frame based on act interpolation and audio input.
#[derive(Resource, Debug, Clone)]
pub struct CurrentBackground {
    /// Current gradient start color
    pub gradient_start: Color,
    /// Current gradient end color
    pub gradient_end: Color,
    /// Intensity of audio-driven pulse effect (0.0 - 1.0)
    pub pulse_intensity: f32,
}

impl Default for CurrentBackground {
    fn default() -> Self {
        Self {
            gradient_start: Color::srgb(0.051, 0.051, 0.090),
            gradient_end: Color::srgb(0.102, 0.102, 0.180),
            pulse_intensity: 0.0,
        }
    }
}

// =============================================================================
// AUDIO RESOURCES
// =============================================================================

/// Real-time audio analysis data from FFT processing.
///
/// Updated each frame with current audio levels across frequency bands.
/// Beat detection uses amplitude history to identify rhythmic events.
#[derive(Resource, Debug, Clone)]
pub struct AudioAnalysis {
    /// Low frequency amplitude (bass presence)
    pub amplitude_low: f32,
    /// Mid frequency amplitude (main musical content)
    pub amplitude_mid: f32,
    /// High frequency amplitude (brightness/shimmer)
    pub amplitude_high: f32,
    /// Peak amplitude across all bands
    pub amplitude_peak: f32,
    /// Bass frequency energy (20-150 Hz)
    pub frequency_bass: f32,
    /// Mid frequency energy (150-4000 Hz)
    pub frequency_mid: f32,
    /// High frequency energy (4000-12000 Hz)
    pub frequency_high: f32,
    /// Shimmer frequency energy (12000+ Hz)
    pub frequency_shimmer: f32,
    /// Whether a beat was detected this frame
    pub beat_detected: bool,
    /// Strength classification of detected beat
    pub beat_strength: BeatStrength,
}

impl Default for AudioAnalysis {
    fn default() -> Self {
        Self {
            amplitude_low: 0.0,
            amplitude_mid: 0.0,
            amplitude_high: 0.0,
            amplitude_peak: 0.0,
            frequency_bass: 0.0,
            frequency_mid: 0.0,
            frequency_high: 0.0,
            frequency_shimmer: 0.0,
            beat_detected: false,
            beat_strength: BeatStrength::Silence,
        }
    }
}

/// Configuration for mapping audio levels to visual parameters.
///
/// Defines how audio input modulates particle appearance and behavior.
#[derive(Resource, Debug, Clone)]
pub struct AudioVisualMapping {
    /// Amplitude maps to particle opacity: (min_opacity, max_opacity)
    pub amplitude_to_opacity_range: (f32, f32),
    /// Amplitude maps to color saturation: (min_saturation, max_saturation)
    pub amplitude_to_saturation_range: (f32, f32),
    /// Amplitude maps to particle scale: (min_scale, max_scale)
    pub amplitude_to_scale_range: (f32, f32),
    /// Amplitude maps to bloom contribution: (min_bloom, max_bloom)
    pub amplitude_to_bloom_range: (f32, f32),
    /// Frequency maps to spawn rate: (min_rate, max_rate) particles/second
    pub frequency_to_spawn_rate_range: (f32, f32),
    /// Frequency maps to hue shift: (min_shift, max_shift) degrees
    pub frequency_to_hue_shift_range: (f32, f32),
}

impl Default for AudioVisualMapping {
    fn default() -> Self {
        Self {
            amplitude_to_opacity_range: (0.3, 0.6),
            amplitude_to_saturation_range: (0.4, 1.0),
            amplitude_to_scale_range: (1.0, 2.5),
            amplitude_to_bloom_range: (0.0, 0.8),
            frequency_to_spawn_rate_range: (4.0, 40.0),
            frequency_to_hue_shift_range: (-5.0, 5.0),
        }
    }
}

/// State for the ambient audio loop that responds to particle count.
///
/// Volume is proportional to active particles - more interaction means
/// louder ambient audio, allowing the app to blend with existing device audio.
#[derive(Resource, Debug, Clone)]
pub struct AmbientAudioState {
    /// Entity holding the audio player component
    pub audio_entity: Option<Entity>,
    /// Current volume level (0.0 to 1.0)
    pub current_volume: f32,
    /// Target volume based on particle count
    pub target_volume: f32,
    /// Maximum volume when at full particle capacity
    pub max_volume: f32,
    /// Minimum particles before audio starts (threshold)
    pub particle_threshold: u32,
    /// Particles at which audio reaches max volume
    pub particle_full_volume: u32,
}

impl Default for AmbientAudioState {
    fn default() -> Self {
        Self {
            audio_entity: None,
            current_volume: 0.0,
            target_volume: 0.0,
            max_volume: 0.7, // 70% max volume to blend nicely with other audio
            particle_threshold: 5, // Audio starts fading in at 5 particles
            particle_full_volume: 50, // Full volume at 50 particles
        }
    }
}

// =============================================================================
// INTERACTION RESOURCES
// =============================================================================

/// Tracks mouse/pointer state for particle interaction.
///
/// Converts window coordinates to world space and tracks velocity
/// for momentum-based interactions.
#[derive(Resource, Debug, Clone)]
pub struct MouseState {
    /// Current mouse position in world coordinates
    pub position: Vec2,
    /// Mouse velocity (change per frame)
    pub velocity: Vec2,
    /// Whether mouse is within the window and active
    pub is_active: bool,
    /// Accumulated interaction time for radius growth
    pub accumulated_interaction: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            is_active: false,
            accumulated_interaction: 0.0,
        }
    }
}

/// Configuration for mouse interaction radius and falloff.
///
/// Radius grows from base to max based on accumulated interaction time,
/// creating a "warming up" feel as the user engages more.
#[derive(Resource, Debug, Clone)]
pub struct InteractionConfig {
    /// Starting interaction radius in pixels
    pub base_radius: f32,
    /// Maximum interaction radius after sustained engagement
    pub max_radius: f32,
    /// Current computed interaction radius
    pub current_radius: f32,
    /// Type of distance falloff for influence calculation
    pub falloff_type: FalloffType,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            base_radius: 80.0,
            max_radius: 200.0,
            current_radius: 80.0,
            falloff_type: FalloffType::Quadratic,
        }
    }
}

/// Current interaction mode based on act state.
///
/// Each act offers a different way for the user to interact with particles:
/// - Paint (Act I): Create particles
/// - Attract (Act II): Draw particles toward cursor
/// - Intensify (Act III): Increase density and saturation
/// - Disperse (Act IV): Push particles away
/// - Ripple (Act V): Create gentle wave disturbances
#[derive(Resource, Debug, Clone)]
pub struct CurrentInteractionMode {
    /// Active interaction mode
    pub mode: InteractionMode,
}

impl Default for CurrentInteractionMode {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Paint,
        }
    }
}

// =============================================================================
// PARTICLE POOL RESOURCES
// =============================================================================

/// Object pool for particle entities to avoid runtime allocations.
///
/// Pre-allocates a large pool of particle entities that are recycled
/// rather than spawned/despawned, ensuring consistent performance.
#[derive(Resource, Debug, Clone)]
pub struct ParticlePool {
    /// Entities available for activation (inactive particles)
    pub available_entities: Vec<Entity>,
    /// Count of currently active particles
    pub active_count: u32,
    /// Total pool capacity (pre-allocated entities)
    pub pool_capacity: u32,
    /// Maximum particles that can be active simultaneously
    pub max_active: u32,
}

impl Default for ParticlePool {
    fn default() -> Self {
        Self {
            available_entities: Vec::with_capacity(15000),
            active_count: 0,
            pool_capacity: 15000,
            max_active: 10000,
        }
    }
}

/// Queue for pending particle spawn requests.
///
/// Spawn requests are accumulated from various sources (mouse, beats, automatic)
/// and processed in batches by the spawn system.
#[derive(Resource, Debug, Clone)]
pub struct ParticleSpawnQueue {
    /// Pending spawn requests to process
    pub pending_spawns: Vec<ParticleSpawnRequest>,
    /// Target spawn rate in particles per second
    pub spawn_rate_per_second: f32,
    /// Accumulated time for spawn timing
    pub spawn_accumulator: f32,
}

impl Default for ParticleSpawnQueue {
    fn default() -> Self {
        Self {
            pending_spawns: Vec::with_capacity(100),
            spawn_rate_per_second: 10.0,
            spawn_accumulator: 0.0,
        }
    }
}

/// A single request to spawn a particle with specified properties.
#[derive(Debug, Clone)]
pub struct ParticleSpawnRequest {
    /// World position to spawn particle
    pub position: Vec2,
    /// Initial velocity for the particle
    pub initial_velocity: Vec2,
    /// Base color for the particle
    pub color: Color,
    /// Lifetime in milliseconds
    pub lifetime_ms: f32,
    /// Source of the spawn request
    pub source: SpawnSource,
}

impl Default for ParticleSpawnRequest {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            initial_velocity: Vec2::ZERO,
            color: Color::WHITE,
            lifetime_ms: 5000.0,
            source: SpawnSource::Automatic,
        }
    }
}

// =============================================================================
// POST-PROCESSING RESOURCES
// =============================================================================

/// Settings for post-processing visual effects.
///
/// Bloom creates the ethereal glow, chromatic aberration adds subtle color
/// fringing, vignette focuses attention toward center, and film grain
/// adds organic texture to the visuals.
#[derive(Resource, Debug, Clone)]
pub struct PostProcessSettings {
    /// Luminance threshold for bloom effect (pixels brighter than this bloom)
    pub bloom_threshold: f32,
    /// Bloom intensity multiplier
    pub bloom_intensity: f32,
    /// Bloom blur radius in pixels
    pub bloom_radius: f32,
    /// Chromatic aberration strength (color channel separation)
    pub chromatic_aberration_strength: f32,
    /// Vignette darkness at edges
    pub vignette_intensity: f32,
    /// Film grain noise amount
    pub film_grain_amount: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            bloom_threshold: 0.7,
            bloom_intensity: 0.3,
            bloom_radius: 8.0,
            chromatic_aberration_strength: 0.0,
            vignette_intensity: 0.3,
            film_grain_amount: 0.02,
        }
    }
}

// =============================================================================
// TIMING RESOURCES
// =============================================================================

/// Timing configuration for motion and transitions.
///
/// Controls the rhythm of particle movement and how quickly
/// visual parameters interpolate during changes.
#[derive(Resource, Debug, Clone)]
pub struct MotionTiming {
    /// Base rhythm in beats per minute (for procedural animation)
    pub base_rhythm_bpm: f32,
    /// Current rhythm (may be modulated by audio)
    pub current_rhythm_bpm: f32,
    /// Duration for fast transitions in milliseconds
    pub transition_duration_ms: f32,
    /// Duration for slow transitions in milliseconds
    pub slow_transition_duration_ms: f32,
}

impl Default for MotionTiming {
    fn default() -> Self {
        Self {
            base_rhythm_bpm: 60.0,
            current_rhythm_bpm: 60.0,
            transition_duration_ms: 800.0,
            slow_transition_duration_ms: 2000.0,
        }
    }
}

/// Runtime performance metrics for monitoring and adaptive quality.
///
/// Tracks frame times and system-specific timings to enable
/// automatic quality adjustment if performance degrades.
#[derive(Resource, Debug, Clone)]
pub struct PerformanceMetrics {
    /// Current frames per second
    pub current_fps: f32,
    /// Last frame time in milliseconds
    pub frame_time_ms: f32,
    /// Time spent updating particles in milliseconds
    pub particle_update_time_ms: f32,
    /// Time spent rendering in milliseconds
    pub render_time_ms: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            current_fps: 60.0,
            frame_time_ms: 16.67,
            particle_update_time_ms: 0.0,
            render_time_ms: 0.0,
        }
    }
}

// =============================================================================
// TEXTURE RESOURCES
// =============================================================================

/// Holds the handle to the pea texture used for particle rendering.
#[derive(Resource, Debug, Clone)]
pub struct PeaTexture {
    /// Handle to the loaded pea image
    pub handle: Handle<Image>,
}

// =============================================================================
// FONT RESOURCES
// =============================================================================

/// Holds the handle to the UI font used for all text rendering.
///
/// On Android, we must explicitly load a font as the default embedded
/// font may not render correctly.
#[derive(Resource, Debug, Clone)]
pub struct UiFont {
    /// Handle to the loaded font
    pub handle: Handle<Font>,
}

/// Plugin that registers all resources for Chromatic Elegy.
///
/// This plugin initializes all global state resources with their default values.
/// Resources can be customized after initialization by accessing them as mutable.
pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        info!(">>> ResourcesPlugin::build() STARTING <<<");

        // Load font directly using world access
        {
            let world = app.world_mut();
            info!(">>> Got world_mut <<<");
            let asset_server = world.resource::<AssetServer>();
            info!(">>> Got AssetServer <<<");
            let handle = asset_server.load("fonts/FiraSans-Bold.ttf");
            info!(">>> Loaded font handle <<<");
            world.insert_resource(UiFont { handle });
            info!(">>> UiFont resource inserted <<<");
        }

        app
            // Application state
            .init_resource::<ActState>()
            .init_resource::<ActTimings>()
            .init_resource::<InterpolatedActValues>()
            // Colors
            .init_resource::<ColorPalette>()
            .init_resource::<BackgroundGradients>()
            .init_resource::<CurrentBackground>()
            // Audio
            .init_resource::<AudioAnalysis>()
            .init_resource::<AudioVisualMapping>()
            .init_resource::<AmbientAudioState>()
            // Interaction
            .init_resource::<MouseState>()
            .init_resource::<InteractionConfig>()
            .init_resource::<CurrentInteractionMode>()
            // Particle pool
            .init_resource::<ParticlePool>()
            .init_resource::<ParticleSpawnQueue>()
            // Post-processing
            .init_resource::<PostProcessSettings>()
            // Timing
            .init_resource::<MotionTiming>()
            .init_resource::<PerformanceMetrics>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_state_default() {
        let state = ActState::default();
        assert!(matches!(state.current_act, Act::Emergence));
        assert_eq!(state.act_progress, 0.0);
        assert_eq!(state.total_elapsed_seconds, 0.0);
        assert!(!state.is_transitioning);
    }

    #[test]
    fn test_act_timings_boundaries() {
        let timings = ActTimings::default();
        // Verify act boundaries are in ascending order
        for i in 0..5 {
            assert!(timings.act_boundaries_seconds[i] < timings.act_boundaries_seconds[i + 1]);
        }
        // Verify total duration is 15 minutes (900 seconds)
        assert_eq!(timings.act_boundaries_seconds[5], 900.0);
    }

    #[test]
    fn test_color_palette_valid_colors() {
        let palette = ColorPalette::default();
        // All colors should be valid sRGB values (components between 0 and 1)
        let check_color = |c: Color| {
            let rgba = c.to_srgba();
            assert!(rgba.red >= 0.0 && rgba.red <= 1.0);
            assert!(rgba.green >= 0.0 && rgba.green <= 1.0);
            assert!(rgba.blue >= 0.0 && rgba.blue <= 1.0);
        };
        check_color(palette.primary_initial);
        check_color(palette.primary_midpoint);
        check_color(palette.primary_final);
    }

    #[test]
    fn test_particle_pool_capacity() {
        let pool = ParticlePool::default();
        assert_eq!(pool.pool_capacity, 15000);
        assert_eq!(pool.max_active, 10000);
        assert!(pool.max_active <= pool.pool_capacity);
    }

    #[test]
    fn test_audio_visual_mapping_ranges() {
        let mapping = AudioVisualMapping::default();
        // Verify all ranges have min < max
        assert!(mapping.amplitude_to_opacity_range.0 < mapping.amplitude_to_opacity_range.1);
        assert!(mapping.amplitude_to_saturation_range.0 < mapping.amplitude_to_saturation_range.1);
        assert!(mapping.amplitude_to_scale_range.0 < mapping.amplitude_to_scale_range.1);
    }

    #[test]
    fn test_interaction_config_radius() {
        let config = InteractionConfig::default();
        assert!(config.base_radius <= config.max_radius);
        assert!(config.current_radius >= config.base_radius);
    }
}
