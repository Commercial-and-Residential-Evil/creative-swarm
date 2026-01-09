//! Module: components
//! Purpose: ECS components for the Chromatic Elegy particle visualization system
//! Dependencies: bevy, crate::types

use bevy::prelude::*;

use crate::types::{FrequencyBand, ParticleBehaviorType, SpawnSource};

// --- Particle Components ---

/// Marker component identifying an entity as a particle in the system.
///
/// Each particle has a unique identifier for tracking and debugging purposes.
#[derive(Component, Debug, Clone, Copy)]
pub struct Particle {
    /// Unique identifier for this particle instance
    pub id: u32,
}

/// Tracks particle lifecycle for pooling and despawn decisions.
///
/// Particles transition from active to inactive when their lifetime expires,
/// at which point they are returned to the object pool for reuse.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParticleState {
    /// Whether this particle is currently active and should be updated/rendered
    pub active: bool,
    /// Time remaining before this particle expires, in milliseconds
    pub lifetime_remaining_ms: f32,
    /// Total lifetime this particle was assigned at spawn, in milliseconds
    pub lifetime_total_ms: f32,
}

impl Default for ParticleState {
    fn default() -> Self {
        Self {
            active: false,
            lifetime_remaining_ms: 0.0,
            lifetime_total_ms: 0.0,
        }
    }
}

/// Visual properties that respond to audio and act state.
///
/// These properties are modulated by the audio-reactive systems and
/// act progression to create the dynamic visual experience.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParticleVisual {
    /// The original color assigned at spawn, used as reference for modulation
    pub base_color: Color,
    /// The currently displayed color after all modulation is applied
    pub current_color: Color,
    /// Current opacity level (0.0 to 1.0)
    pub opacity: f32,
    /// Current scale multiplier for the particle sprite
    pub scale: f32,
    /// How much this particle contributes to the bloom post-process effect
    pub bloom_contribution: f32,
}

impl Default for ParticleVisual {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            current_color: Color::WHITE,
            opacity: 1.0,
            scale: 1.0,
            bloom_contribution: 0.0,
        }
    }
}

/// Physics state for particle movement simulation.
///
/// Contains all kinematic properties needed to simulate organic particle motion
/// with drag, acceleration, and turbulence effects.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParticleMotion {
    /// Current velocity vector in world units per second
    pub velocity: Vec2,
    /// Current acceleration vector applied each frame
    pub acceleration: Vec2,
    /// Drag coefficient that reduces velocity over time (0.0 to 1.0)
    pub drag: f32,
    /// Seed value for deterministic turbulence noise calculation
    pub turbulence_seed: f32,
}

impl Default for ParticleMotion {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            drag: 0.98,
            turbulence_seed: 0.0,
        }
    }
}

/// Act-specific behavior mode that changes with narrative progression.
///
/// Each act in the five-act structure has a corresponding behavior type
/// that affects how particles move and interact with each other.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParticleBehavior {
    /// The current behavior pattern for this particle
    pub behavior_type: ParticleBehaviorType,
    /// How strongly the behavior affects particle motion (0.0 to 1.0)
    pub behavior_strength: f32,
    /// Optional target position for behaviors that require attraction points
    pub target_position: Option<Vec2>,
}

impl Default for ParticleBehavior {
    fn default() -> Self {
        Self {
            behavior_type: ParticleBehaviorType::Drift,
            behavior_strength: 1.0,
            target_position: None,
        }
    }
}

// --- Trail Components ---

/// Individual trail segment with exponential opacity decay.
///
/// Trail segments store historical position data with associated visual
/// properties that fade over time.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrailSegment {
    /// World position where this segment was recorded
    pub position: Vec2,
    /// Current opacity of this segment (decays exponentially)
    pub opacity: f32,
    /// Width of the trail at this segment
    pub width: f32,
    /// Timestamp when this segment was recorded, in milliseconds
    pub timestamp_ms: f32,
}

/// Circular buffer storing trail segment history.
///
/// Uses a fixed-size array with a head index to avoid allocations
/// during runtime updates. The trail system overwrites the oldest
/// segment when adding new positions.
#[derive(Component, Debug, Clone, Copy)]
pub struct Trail {
    /// Fixed-size circular buffer of trail segments
    pub segments: [TrailSegment; 12],
    /// Index of the most recently written segment
    pub head_index: usize,
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            segments: [TrailSegment::default(); 12],
            head_index: 0,
        }
    }
}

impl Trail {
    /// Creates a new trail with all segments initialized to default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new segment to the trail, advancing the head index.
    ///
    /// This overwrites the oldest segment in the circular buffer.
    pub fn push_segment(&mut self, segment: TrailSegment) {
        self.head_index = (self.head_index + 1) % self.segments.len();
        self.segments[self.head_index] = segment;
    }

    /// Returns an iterator over segments from newest to oldest.
    pub fn iter_segments(&self) -> impl Iterator<Item = &TrailSegment> {
        let len = self.segments.len();
        (0..len).map(move |i| {
            let index = (self.head_index + len - i) % len;
            &self.segments[index]
        })
    }
}

/// Rendering configuration for trail visualization.
///
/// Controls how trails are rendered, including width tapering
/// from head to tail.
#[derive(Component, Debug, Clone, Copy)]
pub struct TrailRenderer {
    /// Whether trail rendering is enabled for this particle
    pub enabled: bool,
    /// Width of the trail at the head (newest segment)
    pub base_width: f32,
    /// How quickly the trail tapers from head to tail (0.0 to 1.0)
    pub taper_factor: f32,
}

impl Default for TrailRenderer {
    fn default() -> Self {
        Self {
            enabled: true,
            base_width: 2.0,
            taper_factor: 0.8,
        }
    }
}

// --- Interaction Components ---

/// Per-particle tracking of mouse/pointer influence.
///
/// Updated each frame by the interaction system to track how
/// the cursor affects this specific particle.
#[derive(Component, Debug, Clone, Copy)]
pub struct MouseInfluence {
    /// Whether this particle is currently within the influence radius
    pub affected: bool,
    /// Distance from this particle to the cursor, in world units
    pub distance_to_cursor: f32,
    /// Calculated influence strength based on distance and falloff (0.0 to 1.0)
    pub influence_strength: f32,
}

impl Default for MouseInfluence {
    fn default() -> Self {
        Self {
            affected: false,
            distance_to_cursor: f32::MAX,
            influence_strength: 0.0,
        }
    }
}

/// Marker for particles that respond to attraction forces.
///
/// Particles with this component will be affected by cursor attraction
/// and other gravitational effects based on the current interaction mode.
#[derive(Component, Debug, Clone, Copy)]
pub struct Attractable {
    /// Weight multiplier for attraction forces (higher = more attracted)
    pub attraction_weight: f32,
}

impl Default for Attractable {
    fn default() -> Self {
        Self {
            attraction_weight: 1.0,
        }
    }
}

/// Tracks origin of particle for behavior differentiation.
///
/// Different spawn sources may result in different visual or behavioral
/// characteristics for the particle throughout its lifetime.
#[derive(Component, Debug, Clone, Copy)]
pub struct Spawnable {
    /// The source that created this particle
    pub spawn_source: SpawnSource,
}

impl Default for Spawnable {
    fn default() -> Self {
        Self {
            spawn_source: SpawnSource::Automatic,
        }
    }
}

// --- Audio-Reactive Components ---

/// Configuration for audio-driven visual modulation.
///
/// Particles with this component will have their visual properties
/// modulated based on the specified frequency band and sensitivity.
#[derive(Component, Debug, Clone, Copy)]
pub struct AudioReactive {
    /// How strongly amplitude changes affect this particle (0.0 to 1.0)
    pub amplitude_sensitivity: f32,
    /// Which frequency band this particle responds to
    pub frequency_band: FrequencyBand,
    /// Smoothing factor for response interpolation (higher = smoother, slower)
    pub response_smoothing: f32,
}

impl Default for AudioReactive {
    fn default() -> Self {
        Self {
            amplitude_sensitivity: 0.5,
            frequency_band: FrequencyBand::Mid,
            response_smoothing: 0.1,
        }
    }
}

/// Responds to beat triggers with expansion/contraction.
///
/// When a beat is detected, particles with this component will pulse
/// outward and then decay back to their original state.
#[derive(Component, Debug, Clone, Copy)]
pub struct PulseResponder {
    /// Current phase of the pulse animation (0.0 to 2*PI)
    pub pulse_phase: f32,
    /// Current amplitude of the pulse effect
    pub pulse_amplitude: f32,
    /// How quickly the pulse effect decays (higher = faster decay)
    pub decay_rate: f32,
    /// Current scale modifier from pulse effect (1.0 = no change)
    pub current_scale_modifier: f32,
}

impl Default for PulseResponder {
    fn default() -> Self {
        Self {
            pulse_phase: 0.0,
            pulse_amplitude: 0.0,
            decay_rate: 3.0,
            current_scale_modifier: 1.0,
        }
    }
}

impl PulseResponder {
    /// Triggers a new pulse with the specified amplitude.
    pub fn trigger(&mut self, amplitude: f32) {
        self.pulse_phase = 0.0;
        self.pulse_amplitude = amplitude;
    }

    /// Updates the pulse state, advancing phase and applying decay.
    ///
    /// Returns the current pulse value for visual modulation.
    pub fn update(&mut self, delta_seconds: f32) -> f32 {
        if self.pulse_amplitude > 0.001 {
            self.pulse_phase += delta_seconds * std::f32::consts::TAU;
            self.pulse_amplitude *= (-self.decay_rate * delta_seconds).exp();
            self.pulse_amplitude * self.pulse_phase.sin().abs()
        } else {
            self.pulse_amplitude = 0.0;
            0.0
        }
    }
}

// --- Marker Components ---

/// Marker component for the background entity.
///
/// Used to identify the background sprite/mesh for gradient updates
/// and audio-reactive pulse effects.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BackgroundMarker;

// --- Component Bundles ---

/// Bundle containing all components needed for a complete particle entity.
///
/// Used during pool initialization to create fully-formed particle entities
/// that can be activated/deactivated without modification.
#[derive(Bundle, Default)]
pub struct ParticleBundle {
    /// Core particle marker with ID
    pub particle: Particle,
    /// Lifecycle state
    pub state: ParticleState,
    /// Visual properties
    pub visual: ParticleVisual,
    /// Physics properties
    pub motion: ParticleMotion,
    /// Behavior configuration
    pub behavior: ParticleBehavior,
    /// Trail history
    pub trail: Trail,
    /// Trail rendering config
    pub trail_renderer: TrailRenderer,
    /// Mouse interaction state
    pub mouse_influence: MouseInfluence,
    /// Attraction configuration
    pub attractable: Attractable,
    /// Spawn source tracking
    pub spawnable: Spawnable,
    /// Audio reactivity config
    pub audio_reactive: AudioReactive,
    /// Beat pulse response
    pub pulse_responder: PulseResponder,
    /// Sprite for rendering
    pub sprite: Sprite,
    /// Transform for position/scale/rotation
    pub transform: Transform,
    /// Global transform (computed)
    pub global_transform: GlobalTransform,
    /// Visibility control
    pub visibility: Visibility,
    /// Inherited visibility (computed)
    pub inherited_visibility: InheritedVisibility,
    /// View visibility (computed)
    pub view_visibility: ViewVisibility,
}

impl ParticleBundle {
    /// Creates a new particle bundle with the specified ID.
    ///
    /// The particle starts in an inactive state with default values.
    pub fn new(id: u32) -> Self {
        Self {
            particle: Particle { id },
            state: ParticleState::default(),
            visual: ParticleVisual::default(),
            motion: ParticleMotion::default(),
            behavior: ParticleBehavior::default(),
            trail: Trail::default(),
            trail_renderer: TrailRenderer::default(),
            mouse_influence: MouseInfluence::default(),
            attractable: Attractable::default(),
            spawnable: Spawnable::default(),
            audio_reactive: AudioReactive::default(),
            pulse_responder: PulseResponder::default(),
            sprite: Sprite::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Hidden,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self { id: 0 }
    }
}

// --- Plugin ---

/// Plugin that registers all particle and related components.
///
/// Note: Components don't need explicit registration in Bevy 0.13+,
/// but this plugin is provided for consistency and potential future use.
pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, _app: &mut App) {
        // Components are automatically registered when used.
        // This plugin exists for organizational purposes and
        // potential future initialization logic.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_state_default() {
        let state = ParticleState::default();
        assert!(!state.active);
        assert_eq!(state.lifetime_remaining_ms, 0.0);
        assert_eq!(state.lifetime_total_ms, 0.0);
    }

    #[test]
    fn test_trail_circular_buffer() {
        let mut trail = Trail::new();

        // Push 15 segments (more than capacity of 12)
        for i in 0..15 {
            trail.push_segment(TrailSegment {
                position: Vec2::new(i as f32, 0.0),
                opacity: 1.0,
                width: 1.0,
                timestamp_ms: i as f32 * 100.0,
            });
        }

        // Head index should wrap around
        assert!(trail.head_index < 12);

        // Most recent segment should have position (14, 0)
        let segments: Vec<_> = trail.iter_segments().collect();
        assert_eq!(segments[0].position.x, 14.0);
    }

    #[test]
    fn test_pulse_responder_trigger() {
        let mut responder = PulseResponder::default();
        responder.trigger(1.0);

        assert_eq!(responder.pulse_phase, 0.0);
        assert_eq!(responder.pulse_amplitude, 1.0);
    }

    #[test]
    fn test_pulse_responder_decay() {
        let mut responder = PulseResponder::default();
        responder.trigger(1.0);

        // Update for one second
        let _ = responder.update(1.0);

        // Amplitude should have decayed
        assert!(responder.pulse_amplitude < 1.0);
    }

    #[test]
    fn test_particle_bundle_creation() {
        let bundle = ParticleBundle::new(42);
        assert_eq!(bundle.particle.id, 42);
        assert!(!bundle.state.active);
        assert_eq!(bundle.visibility, Visibility::Hidden);
    }
}
