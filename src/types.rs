//! Module: types
//! Purpose: Core enums and type definitions for Chromatic Elegy
//! Dependencies: None (foundational module)

use bevy::prelude::*;

// =============================================================================
// ACT TIMING CONSTANTS
// =============================================================================

/// Total duration of the experience in seconds (15 minutes).
pub const TOTAL_DURATION_SECONDS: f32 = 900.0;

/// Act boundary timestamps in seconds.
/// Index 0: Start of Emergence (0:00)
/// Index 1: Start of Accumulation (3:00)
/// Index 2: Start of Crescendo (7:00)
/// Index 3: Start of Release (10:00)
/// Index 4: Start of Transcendence (13:00)
/// Index 5: End of experience (15:00)
pub const ACT_BOUNDARIES_SECONDS: [f32; 6] = [0.0, 180.0, 420.0, 600.0, 780.0, 900.0];

/// Default transition duration between acts in milliseconds.
pub const TRANSITION_DURATION_MS: f32 = 2000.0;

// =============================================================================
// ACT ENUM
// =============================================================================

/// The five-act narrative structure of Chromatic Elegy.
///
/// Each act represents a distinct emotional and visual phase of the experience,
/// transitioning from emergence through transcendence over 15 minutes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum Act {
    /// Act I (0-3 min): Sparse particles drift slowly, discovering space.
    /// Viewer touch creates new particles. Dark, intimate atmosphere.
    #[default]
    Emergence,

    /// Act II (3-7 min): Particles coalesce and gather around viewer presence.
    /// Density increases, colors warm. Attraction forces dominate.
    Accumulation,

    /// Act III (7-10 min): Peak density and intensity. Dense gravitational swarms,
    /// saturated colors, maximum audio reactivity.
    Crescendo,

    /// Act IV (10-13 min): Particles begin to release and disperse upward.
    /// Touch releases rather than attracts. Bittersweet transition.
    Release,

    /// Act V (13-15 min): Weightless luminosity. Particles float gently,
    /// nearly white, peaceful dissolution into light.
    Transcendence,
}

impl Act {
    /// Returns the duration of this act in seconds.
    #[must_use]
    pub fn duration_seconds(&self) -> f32 {
        let index = self.index();
        ACT_BOUNDARIES_SECONDS[index + 1] - ACT_BOUNDARIES_SECONDS[index]
    }

    /// Returns the start time of this act in seconds.
    #[must_use]
    pub fn start_seconds(&self) -> f32 {
        ACT_BOUNDARIES_SECONDS[self.index()]
    }

    /// Returns the end time of this act in seconds.
    #[must_use]
    pub fn end_seconds(&self) -> f32 {
        ACT_BOUNDARIES_SECONDS[self.index() + 1]
    }

    /// Returns the zero-based index of this act (0-4).
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Act::Emergence => 0,
            Act::Accumulation => 1,
            Act::Crescendo => 2,
            Act::Release => 3,
            Act::Transcendence => 4,
        }
    }

    /// Returns the act for a given elapsed time in seconds.
    /// Returns `Transcendence` for times beyond the total duration.
    #[must_use]
    pub fn from_elapsed_seconds(elapsed: f32) -> Self {
        if elapsed < ACT_BOUNDARIES_SECONDS[1] {
            Act::Emergence
        } else if elapsed < ACT_BOUNDARIES_SECONDS[2] {
            Act::Accumulation
        } else if elapsed < ACT_BOUNDARIES_SECONDS[3] {
            Act::Crescendo
        } else if elapsed < ACT_BOUNDARIES_SECONDS[4] {
            Act::Release
        } else {
            Act::Transcendence
        }
    }

    /// Returns the next act in sequence, or `None` if this is the final act.
    #[must_use]
    pub fn next(&self) -> Option<Self> {
        match self {
            Act::Emergence => Some(Act::Accumulation),
            Act::Accumulation => Some(Act::Crescendo),
            Act::Crescendo => Some(Act::Release),
            Act::Release => Some(Act::Transcendence),
            Act::Transcendence => None,
        }
    }

    /// Returns the previous act in sequence, or `None` if this is the first act.
    #[must_use]
    pub fn previous(&self) -> Option<Self> {
        match self {
            Act::Emergence => None,
            Act::Accumulation => Some(Act::Emergence),
            Act::Crescendo => Some(Act::Accumulation),
            Act::Release => Some(Act::Crescendo),
            Act::Transcendence => Some(Act::Release),
        }
    }

    /// Returns the default particle behavior type for this act.
    #[must_use]
    pub fn default_behavior(&self) -> ParticleBehaviorType {
        match self {
            Act::Emergence => ParticleBehaviorType::Drift,
            Act::Accumulation => ParticleBehaviorType::Swarm,
            Act::Crescendo => ParticleBehaviorType::Orbit,
            Act::Release => ParticleBehaviorType::Disperse,
            Act::Transcendence => ParticleBehaviorType::Float,
        }
    }

    /// Returns the default interaction mode for this act.
    #[must_use]
    pub fn default_interaction_mode(&self) -> InteractionMode {
        match self {
            Act::Emergence => InteractionMode::Paint,
            Act::Accumulation => InteractionMode::Attract,
            Act::Crescendo => InteractionMode::Intensify,
            Act::Release => InteractionMode::Disperse,
            Act::Transcendence => InteractionMode::Ripple,
        }
    }

    /// Returns true if this is the final act.
    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(self, Act::Transcendence)
    }

    /// Returns all acts in order.
    #[must_use]
    pub fn all() -> [Act; 5] {
        [
            Act::Emergence,
            Act::Accumulation,
            Act::Crescendo,
            Act::Release,
            Act::Transcendence,
        ]
    }

    /// Returns the display name of the act with act number.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Act::Emergence => "Act I: Emergence",
            Act::Accumulation => "Act II: Accumulation",
            Act::Crescendo => "Act III: Crescendo",
            Act::Release => "Act IV: Release",
            Act::Transcendence => "Act V: Transcendence",
        }
    }
}

// =============================================================================
// PARTICLE BEHAVIOR TYPE ENUM
// =============================================================================

/// Defines how particles move and behave in the simulation.
///
/// Each behavior type corresponds to an act's emotional quality,
/// creating distinct visual signatures as the experience progresses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum ParticleBehaviorType {
    /// Slow, aimless movement as particles discover the space.
    /// Characteristic of Act I: Emergence.
    #[default]
    Drift,

    /// Particles coalesce and move toward common centers.
    /// Characteristic of Act II: Accumulation.
    Swarm,

    /// Dense gravitational patterns with particles circling focal points.
    /// Characteristic of Act III: Crescendo.
    Orbit,

    /// Particles break apart and rise upward, releasing tension.
    /// Characteristic of Act IV: Release.
    Disperse,

    /// Weightless, peaceful movement with minimal forces.
    /// Characteristic of Act V: Transcendence.
    Float,
}

impl ParticleBehaviorType {
    /// Returns the base speed multiplier for this behavior type.
    #[must_use]
    pub fn base_speed_multiplier(&self) -> f32 {
        match self {
            ParticleBehaviorType::Drift => 0.3,
            ParticleBehaviorType::Swarm => 0.7,
            ParticleBehaviorType::Orbit => 1.0,
            ParticleBehaviorType::Disperse => 1.2,
            ParticleBehaviorType::Float => 0.4,
        }
    }

    /// Returns the base drag coefficient for this behavior type.
    #[must_use]
    pub fn base_drag(&self) -> f32 {
        match self {
            ParticleBehaviorType::Drift => 0.98,
            ParticleBehaviorType::Swarm => 0.95,
            ParticleBehaviorType::Orbit => 0.92,
            ParticleBehaviorType::Disperse => 0.96,
            ParticleBehaviorType::Float => 0.99,
        }
    }

    /// Returns the turbulence strength for this behavior type.
    #[must_use]
    pub fn turbulence_strength(&self) -> f32 {
        match self {
            ParticleBehaviorType::Drift => 0.5,
            ParticleBehaviorType::Swarm => 0.3,
            ParticleBehaviorType::Orbit => 0.2,
            ParticleBehaviorType::Disperse => 0.6,
            ParticleBehaviorType::Float => 0.4,
        }
    }
}

// =============================================================================
// INTERACTION MODE ENUM
// =============================================================================

/// Defines how the viewer's mouse/touch input affects particles.
///
/// Each mode corresponds to an act, creating a distinct relationship
/// between viewer and particles as the experience evolves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum InteractionMode {
    /// Touch creates new particles at the cursor position.
    /// Used in Act I: Emergence.
    #[default]
    Paint,

    /// Particles are drawn toward the cursor position.
    /// Used in Act II: Accumulation.
    Attract,

    /// Proximity to cursor increases particle density and saturation.
    /// Used in Act III: Crescendo.
    Intensify,

    /// Touch releases particles, pushing them away.
    /// Used in Act IV: Release.
    Disperse,

    /// Creates gentle wave disturbances in the luminous field.
    /// Used in Act V: Transcendence.
    Ripple,
}

impl InteractionMode {
    /// Returns the force strength multiplier for this interaction mode.
    #[must_use]
    pub fn force_multiplier(&self) -> f32 {
        match self {
            InteractionMode::Paint => 0.0,
            InteractionMode::Attract => 1.0,
            InteractionMode::Intensify => 0.5,
            InteractionMode::Disperse => -0.8,
            InteractionMode::Ripple => 0.3,
        }
    }

    /// Returns true if this mode spawns particles on interaction.
    #[must_use]
    pub fn spawns_particles(&self) -> bool {
        matches!(self, InteractionMode::Paint)
    }

    /// Returns true if this mode affects particle visual properties.
    #[must_use]
    pub fn affects_visuals(&self) -> bool {
        matches!(self, InteractionMode::Intensify | InteractionMode::Ripple)
    }
}

// =============================================================================
// BEAT STRENGTH ENUM
// =============================================================================

/// Classifies the strength of detected audio beats.
///
/// Different beat strengths trigger different visual responses,
/// from subtle pulses to dramatic burst emissions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum BeatStrength {
    /// No beat detected; may trigger fade acceleration.
    #[default]
    Silence,

    /// Gentle pulse affecting nearby particles subtly.
    Soft,

    /// Medium intensity triggering concentric ripple spawns.
    Medium,

    /// Strong beat triggering radial burst emission.
    Strong,
}

impl BeatStrength {
    /// Returns the spawn count multiplier for this beat strength.
    #[must_use]
    pub fn spawn_multiplier(&self) -> f32 {
        match self {
            BeatStrength::Silence => 0.0,
            BeatStrength::Soft => 1.0,
            BeatStrength::Medium => 3.0,
            BeatStrength::Strong => 8.0,
        }
    }

    /// Returns the pulse intensity for this beat strength.
    #[must_use]
    pub fn pulse_intensity(&self) -> f32 {
        match self {
            BeatStrength::Silence => 0.0,
            BeatStrength::Soft => 0.2,
            BeatStrength::Medium => 0.5,
            BeatStrength::Strong => 1.0,
        }
    }

    /// Returns true if this beat strength should trigger particle spawning.
    #[must_use]
    pub fn should_spawn(&self) -> bool {
        !matches!(self, BeatStrength::Silence)
    }

    /// Creates a beat strength from an amplitude value (0.0 - 1.0).
    #[must_use]
    pub fn from_amplitude(amplitude: f32) -> Self {
        if amplitude < 0.1 {
            BeatStrength::Silence
        } else if amplitude < 0.4 {
            BeatStrength::Soft
        } else if amplitude < 0.7 {
            BeatStrength::Medium
        } else {
            BeatStrength::Strong
        }
    }
}

// =============================================================================
// FREQUENCY BAND ENUM
// =============================================================================

/// Audio frequency bands used for reactive visual mappings.
///
/// Different frequency bands drive different visual properties,
/// creating a rich audio-visual correspondence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum FrequencyBand {
    /// Low frequencies (20-250 Hz) driving background pulse.
    #[default]
    Bass,

    /// Mid frequencies (250-2000 Hz) driving particle spawn rate.
    Mid,

    /// High frequencies (2000-8000 Hz) driving accent sparkle probability.
    High,

    /// Very high frequencies (8000-20000 Hz) driving hue micro-shifts.
    Shimmer,
}

impl FrequencyBand {
    /// Returns the frequency range in Hz for this band as (min, max).
    #[must_use]
    pub fn frequency_range_hz(&self) -> (f32, f32) {
        match self {
            FrequencyBand::Bass => (20.0, 250.0),
            FrequencyBand::Mid => (250.0, 2000.0),
            FrequencyBand::High => (2000.0, 8000.0),
            FrequencyBand::Shimmer => (8000.0, 20000.0),
        }
    }

    /// Returns the visual property this band primarily affects.
    #[must_use]
    pub fn visual_target(&self) -> &'static str {
        match self {
            FrequencyBand::Bass => "background_pulse",
            FrequencyBand::Mid => "spawn_rate",
            FrequencyBand::High => "sparkle_probability",
            FrequencyBand::Shimmer => "hue_shift",
        }
    }

    /// Returns all frequency bands in order from low to high.
    #[must_use]
    pub fn all() -> [FrequencyBand; 4] {
        [
            FrequencyBand::Bass,
            FrequencyBand::Mid,
            FrequencyBand::High,
            FrequencyBand::Shimmer,
        ]
    }
}

// =============================================================================
// SPAWN SOURCE ENUM
// =============================================================================

/// Identifies the origin of a particle spawn for behavior differentiation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum SpawnSource {
    /// Particle was spawned from user mouse/touch interaction.
    Mouse,

    /// Particle was spawned in response to an audio beat.
    Beat,

    /// Particle was spawned automatically based on time/act progression.
    #[default]
    Automatic,
}

impl SpawnSource {
    /// Returns the base lifetime multiplier for particles from this source.
    #[must_use]
    pub fn lifetime_multiplier(&self) -> f32 {
        match self {
            SpawnSource::Mouse => 1.2,
            SpawnSource::Beat => 0.8,
            SpawnSource::Automatic => 1.0,
        }
    }

    /// Returns true if particles from this source should have trails.
    #[must_use]
    pub fn has_trail(&self) -> bool {
        match self {
            SpawnSource::Mouse => true,
            SpawnSource::Beat => true,
            SpawnSource::Automatic => false,
        }
    }
}

// =============================================================================
// FALLOFF TYPE ENUM
// =============================================================================

/// Defines how influence strength decreases with distance.
///
/// Used for mouse influence radius calculations and other
/// distance-based effect falloffs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub enum FalloffType {
    /// Strength decreases linearly with distance: 1 - (d / max_d).
    Linear,

    /// Strength decreases with square of distance: 1 - (d / max_d)^2.
    /// Default for mouse influence, feels natural.
    #[default]
    Quadratic,

    /// Strength decreases exponentially: e^(-k * d).
    Exponential,
}

impl FalloffType {
    /// Calculates the falloff value for a given distance and maximum distance.
    ///
    /// Returns a value in the range [0.0, 1.0] where 1.0 is full strength
    /// (at distance 0) and 0.0 is no strength (at or beyond max_distance).
    ///
    /// # Arguments
    /// * `distance` - The current distance from the influence source.
    /// * `max_distance` - The maximum distance at which influence reaches zero.
    #[must_use]
    pub fn calculate(&self, distance: f32, max_distance: f32) -> f32 {
        if distance >= max_distance || max_distance <= 0.0 {
            return 0.0;
        }

        let normalized = distance / max_distance;

        match self {
            FalloffType::Linear => (1.0 - normalized).max(0.0),
            FalloffType::Quadratic => (1.0 - normalized * normalized).max(0.0),
            FalloffType::Exponential => (-3.0 * normalized).exp(),
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_timing() {
        assert_eq!(Act::Emergence.duration_seconds(), 180.0);
        assert_eq!(Act::Accumulation.duration_seconds(), 240.0);
        assert_eq!(Act::Crescendo.duration_seconds(), 180.0);
        assert_eq!(Act::Release.duration_seconds(), 180.0);
        assert_eq!(Act::Transcendence.duration_seconds(), 120.0);

        let total: f32 = Act::all().iter().map(|a| a.duration_seconds()).sum();
        assert_eq!(total, TOTAL_DURATION_SECONDS);
    }

    #[test]
    fn test_act_from_elapsed() {
        assert_eq!(Act::from_elapsed_seconds(0.0), Act::Emergence);
        assert_eq!(Act::from_elapsed_seconds(90.0), Act::Emergence);
        assert_eq!(Act::from_elapsed_seconds(180.0), Act::Accumulation);
        assert_eq!(Act::from_elapsed_seconds(420.0), Act::Crescendo);
        assert_eq!(Act::from_elapsed_seconds(600.0), Act::Release);
        assert_eq!(Act::from_elapsed_seconds(780.0), Act::Transcendence);
        assert_eq!(Act::from_elapsed_seconds(1000.0), Act::Transcendence);
    }

    #[test]
    fn test_act_navigation() {
        assert_eq!(Act::Emergence.next(), Some(Act::Accumulation));
        assert_eq!(Act::Transcendence.next(), None);
        assert_eq!(Act::Emergence.previous(), None);
        assert_eq!(Act::Transcendence.previous(), Some(Act::Release));
    }

    #[test]
    fn test_falloff_calculation() {
        // Linear falloff
        assert_eq!(FalloffType::Linear.calculate(0.0, 100.0), 1.0);
        assert_eq!(FalloffType::Linear.calculate(50.0, 100.0), 0.5);
        assert_eq!(FalloffType::Linear.calculate(100.0, 100.0), 0.0);

        // Quadratic falloff
        assert_eq!(FalloffType::Quadratic.calculate(0.0, 100.0), 1.0);
        assert_eq!(FalloffType::Quadratic.calculate(100.0, 100.0), 0.0);

        // Beyond max distance
        assert_eq!(FalloffType::Linear.calculate(150.0, 100.0), 0.0);
        assert_eq!(FalloffType::Quadratic.calculate(150.0, 100.0), 0.0);
    }

    #[test]
    fn test_beat_strength_from_amplitude() {
        assert_eq!(BeatStrength::from_amplitude(0.0), BeatStrength::Silence);
        assert_eq!(BeatStrength::from_amplitude(0.05), BeatStrength::Silence);
        assert_eq!(BeatStrength::from_amplitude(0.2), BeatStrength::Soft);
        assert_eq!(BeatStrength::from_amplitude(0.5), BeatStrength::Medium);
        assert_eq!(BeatStrength::from_amplitude(0.9), BeatStrength::Strong);
    }

    #[test]
    fn test_frequency_band_ranges() {
        let bands = FrequencyBand::all();
        for i in 0..bands.len() - 1 {
            let (_, max) = bands[i].frequency_range_hz();
            let (min, _) = bands[i + 1].frequency_range_hz();
            assert_eq!(max, min, "Frequency bands should be contiguous");
        }
    }
}
