//! Module: audio_reactive
//! Purpose: Audio analysis and visual synchronization systems for Chromatic Elegy
//! Dependencies: types, components, resources, bevy::prelude

use bevy::prelude::*;
use bevy::color::Hsla;

use crate::components::{
    AudioReactive, Particle, ParticleState, ParticleVisual, PulseResponder,
};
use crate::resources::{
    ActState, AmbientAudioState, AudioAnalysis, AudioVisualMapping, CurrentBackground,
    ParticlePool, ParticleSpawnQueue,
};
use crate::types::{Act, BeatStrength, FrequencyBand};

// =============================================================================
// EVENTS
// =============================================================================

/// Event fired when a beat is detected in the audio stream.
///
/// The strength classification determines the visual response magnitude,
/// from subtle pulses to dramatic burst emissions.
#[derive(Event, Debug, Clone, Copy)]
pub struct BeatDetected {
    /// Classification of the detected beat's intensity
    pub strength: BeatStrength,
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Smoothly interpolates between current and target values with configurable smoothing.
///
/// Uses exponential smoothing for natural, organic transitions.
///
/// # Arguments
/// * `current` - The current value to interpolate from
/// * `target` - The target value to interpolate toward
/// * `smoothing` - Smoothing factor (0.0 = instant, 1.0 = no change). Typical range: 0.05-0.3
/// * `dt` - Delta time in seconds
///
/// # Returns
/// The interpolated value moving from current toward target
#[inline]
pub fn lerp_smooth(current: f32, target: f32, smoothing: f32, dt: f32) -> f32 {
    // Calculate the interpolation factor based on smoothing and delta time
    // Higher smoothing means slower approach to target
    let factor = 1.0 - (smoothing.clamp(0.0, 0.99)).powf(dt * 60.0);
    current + (target - current) * factor
}

/// Maps a value from one range to another.
///
/// # Arguments
/// * `value` - The input value to map
/// * `in_min` - Minimum of the input range
/// * `in_max` - Maximum of the input range
/// * `out_min` - Minimum of the output range
/// * `out_max` - Maximum of the output range
///
/// # Returns
/// The mapped value, clamped to the output range
#[inline]
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    if (in_max - in_min).abs() < f32::EPSILON {
        return out_min;
    }
    let normalized = ((value - in_min) / (in_max - in_min)).clamp(0.0, 1.0);
    out_min + normalized * (out_max - out_min)
}

// =============================================================================
// AUDIO PROCESSING SYSTEMS
// =============================================================================

/// Processes audio input and updates frequency band analysis.
///
/// This is a placeholder implementation that generates procedural audio-like data
/// based on elapsed time and current act. Actual FFT audio integration can be added
/// later by replacing the procedural generation with real audio analysis.
///
/// # System Ordering
/// - Priority: HIGH
/// - Runs before: `detect_beats`
pub fn process_audio_input(
    time: Res<Time>,
    act_state: Res<ActState>,
    mut audio_analysis: ResMut<AudioAnalysis>,
) {
    let elapsed = time.elapsed_secs();
    let act_factor = get_act_intensity_factor(&act_state.current_act);

    // Generate procedural audio-like data based on time and act
    // These simulate realistic amplitude variations for testing without real audio

    // Base frequencies for oscillation (simulating different frequency responses)
    let bass_freq = 0.5; // Slow bass pulse
    let mid_freq = 1.2; // Medium frequency content
    let high_freq = 2.5; // Faster high frequency shimmer
    let shimmer_freq = 4.0; // Very fast shimmer

    // Generate amplitude values with act-based intensity
    let bass_base = (elapsed * bass_freq * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let mid_base = (elapsed * mid_freq * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let high_base = (elapsed * high_freq * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let shimmer_base = (elapsed * shimmer_freq * std::f32::consts::TAU).sin() * 0.5 + 0.5;

    // Add some noise/variation using multiple frequencies
    let noise1 = (elapsed * 3.7).sin() * 0.1;
    let noise2 = (elapsed * 7.3).sin() * 0.05;

    // Apply act-based intensity scaling
    audio_analysis.amplitude_low = (bass_base * act_factor * 0.7 + noise1).clamp(0.0, 1.0);
    audio_analysis.amplitude_mid = (mid_base * act_factor + noise2).clamp(0.0, 1.0);
    audio_analysis.amplitude_high = (high_base * act_factor * 0.8).clamp(0.0, 1.0);
    audio_analysis.amplitude_peak = audio_analysis
        .amplitude_mid
        .max(audio_analysis.amplitude_high)
        .max(audio_analysis.amplitude_low);

    // Frequency band energies (similar to amplitudes but with different characteristics)
    audio_analysis.frequency_bass = (bass_base * act_factor * 0.8 + noise1 * 0.5).clamp(0.0, 1.0);
    audio_analysis.frequency_mid = (mid_base * act_factor * 0.9 + noise2).clamp(0.0, 1.0);
    audio_analysis.frequency_high = (high_base * act_factor * 0.7).clamp(0.0, 1.0);
    audio_analysis.frequency_shimmer = (shimmer_base * act_factor * 0.5).clamp(0.0, 1.0);
}

/// Returns an intensity factor based on the current act.
///
/// Acts have different baseline intensities that affect audio-visual mappings:
/// - Emergence: Very low (0.3) - sparse, quiet
/// - Accumulation: Medium (0.6) - building
/// - Crescendo: Maximum (1.0) - peak intensity
/// - Release: Medium-high (0.7) - diminishing
/// - Transcendence: Low (0.4) - peaceful dissolution
fn get_act_intensity_factor(act: &Act) -> f32 {
    match act {
        Act::Emergence => 0.3,
        Act::Accumulation => 0.6,
        Act::Crescendo => 1.0,
        Act::Release => 0.7,
        Act::Transcendence => 0.4,
    }
}

/// Analyzes amplitude history to detect beat events and classify their strength.
///
/// Uses energy flux analysis to detect sudden amplitude increases that
/// correspond to rhythmic beats. Beats are classified by their intensity.
///
/// # System Ordering
/// - Runs after: `process_audio_input`
/// - Runs before: `apply_audio_to_spawn_rate`
pub fn detect_beats(
    time: Res<Time>,
    mut audio_analysis: ResMut<AudioAnalysis>,
    mut beat_events: EventWriter<BeatDetected>,
) {
    let elapsed = time.elapsed_secs();

    // Simulate beat detection using a rhythmic pattern
    // In a real implementation, this would analyze amplitude history
    // and detect sudden energy increases

    // Create beats at approximately 60-72 BPM (every 0.83-1.0 seconds)
    // with some variation
    let beat_period = 0.9 + (elapsed * 0.1).sin() * 0.15;
    let beat_phase = (elapsed % beat_period) / beat_period;

    // Detect beat on the "attack" phase (first 10% of the period)
    let is_beat = beat_phase < 0.1;

    // Classify beat strength based on amplitude
    let amplitude = audio_analysis.amplitude_peak;
    let strength = classify_beat_strength(amplitude);

    audio_analysis.beat_detected = is_beat && strength != BeatStrength::Silence;
    audio_analysis.beat_strength = strength;

    // Send beat event if detected
    if audio_analysis.beat_detected {
        beat_events.send(BeatDetected { strength });
    }
}

/// Classifies beat strength based on amplitude threshold.
///
/// # Thresholds
/// - Silence: amplitude < 0.1
/// - Soft: 0.1 <= amplitude < 0.3
/// - Medium: 0.3 <= amplitude < 0.6
/// - Strong: amplitude >= 0.6
fn classify_beat_strength(amplitude: f32) -> BeatStrength {
    if amplitude < 0.1 {
        BeatStrength::Silence
    } else if amplitude < 0.3 {
        BeatStrength::Soft
    } else if amplitude < 0.6 {
        BeatStrength::Medium
    } else {
        BeatStrength::Strong
    }
}

// =============================================================================
// AUDIO-TO-VISUAL MAPPING SYSTEMS
// =============================================================================

/// Maps mid-frequency audio levels to particle spawn rate.
///
/// Higher mid-frequency energy results in more particles being spawned,
/// creating a direct visual representation of the audio content.
///
/// # Mapping Range
/// - Input: frequency_mid (0.0 - 1.0)
/// - Output: spawn_rate_per_second (4.0 - 40.0)
///
/// # System Ordering
/// - Runs after: `detect_beats`
/// - Runs before: `spawn_particles_from_queue`
pub fn apply_audio_to_spawn_rate(
    audio_analysis: Res<AudioAnalysis>,
    mapping: Res<AudioVisualMapping>,
    mut spawn_queue: ResMut<ParticleSpawnQueue>,
) {
    let (min_rate, max_rate) = mapping.frequency_to_spawn_rate_range;

    // Map mid-frequency energy to spawn rate
    let target_rate = map_range(audio_analysis.frequency_mid, 0.0, 1.0, min_rate, max_rate);

    // Apply smoothing to prevent jarring rate changes
    spawn_queue.spawn_rate_per_second = lerp_smooth(
        spawn_queue.spawn_rate_per_second,
        target_rate,
        0.2,
        1.0 / 60.0, // Assume 60 FPS for smoothing
    );
}

/// Applies audio-driven modulation to particle visual properties.
///
/// Particles with the `AudioReactive` component have their opacity, saturation,
/// scale, and bloom contribution modulated based on their assigned frequency band.
///
/// # Visual Mappings
/// - Amplitude -> Opacity: 0.3 - 0.6
/// - Amplitude -> Saturation: 0.4 - 1.0
/// - Amplitude -> Scale: 1.0 - 2.5
/// - Amplitude -> Bloom: 0.0 - 0.8
pub fn apply_audio_to_visuals(
    time: Res<Time>,
    audio_analysis: Res<AudioAnalysis>,
    mapping: Res<AudioVisualMapping>,
    mut query: Query<(&mut ParticleVisual, &AudioReactive), With<Particle>>,
) {
    let dt = time.delta_secs();

    for (mut visual, audio_reactive) in query.iter_mut() {
        // Get amplitude for this particle's frequency band
        let amplitude = get_amplitude_for_band(&audio_analysis, audio_reactive.frequency_band);

        // Apply sensitivity scaling
        let scaled_amplitude = amplitude * audio_reactive.amplitude_sensitivity;

        // Map amplitude to visual properties
        let target_opacity = map_range(
            scaled_amplitude,
            0.0,
            1.0,
            mapping.amplitude_to_opacity_range.0,
            mapping.amplitude_to_opacity_range.1,
        );

        let target_scale = map_range(
            scaled_amplitude,
            0.0,
            1.0,
            mapping.amplitude_to_scale_range.0,
            mapping.amplitude_to_scale_range.1,
        );

        let target_bloom = map_range(
            scaled_amplitude,
            0.0,
            1.0,
            mapping.amplitude_to_bloom_range.0,
            mapping.amplitude_to_bloom_range.1,
        );

        // Apply smoothing for gradual changes
        let smoothing = audio_reactive.response_smoothing;
        visual.opacity = lerp_smooth(visual.opacity, target_opacity, smoothing, dt);
        visual.scale = lerp_smooth(visual.scale, target_scale, smoothing, dt);
        visual.bloom_contribution =
            lerp_smooth(visual.bloom_contribution, target_bloom, smoothing, dt);

        // Apply saturation modulation to current color
        let target_saturation = map_range(
            scaled_amplitude,
            0.0,
            1.0,
            mapping.amplitude_to_saturation_range.0,
            mapping.amplitude_to_saturation_range.1,
        );

        // Modulate current color saturation using Bevy 0.15 color API
        let base_hsla = Hsla::from(visual.base_color);
        let current_hsla = Hsla::from(visual.current_color);
        let new_saturation = lerp_smooth(
            current_hsla.saturation,
            base_hsla.saturation * target_saturation,
            smoothing,
            dt,
        );

        visual.current_color = Color::from(Hsla::new(
            current_hsla.hue,
            new_saturation,
            current_hsla.lightness,
            visual.opacity,
        ));
    }
}

/// Returns the amplitude value for a specific frequency band.
fn get_amplitude_for_band(analysis: &AudioAnalysis, band: FrequencyBand) -> f32 {
    match band {
        FrequencyBand::Bass => analysis.amplitude_low,
        FrequencyBand::Mid => analysis.amplitude_mid,
        FrequencyBand::High => analysis.amplitude_high,
        FrequencyBand::Shimmer => analysis.frequency_shimmer,
    }
}

/// Applies pulse effect to particles with PulseResponder component.
///
/// When a beat is detected, particles expand briefly then contract back,
/// creating a "breathing" visual effect based on each particle's age.
///
/// # Behavior
/// - Each particle pulses independently based on its lifetime
/// - Younger particles pulse more intensely, fading as they age
/// - Creates organic, varied visual where particles are at different phases
pub fn apply_pulse_effect(
    mut query: Query<(&ParticleState, &mut ParticleVisual, &mut PulseResponder), With<Particle>>,
) {
    // Pulse frequency - how many cycles per second
    const PULSE_FREQUENCY: f32 = 1.5;
    // Maximum scale variation (0.15 = up to 15% size change)
    const PULSE_AMPLITUDE: f32 = 0.15;

    for (state, mut visual, mut pulse_responder) in query.iter_mut() {
        if !state.active || state.lifetime_total_ms <= 0.0 {
            pulse_responder.current_scale_modifier = 1.0;
            continue;
        }

        // Calculate particle age in seconds
        let age_seconds = (state.lifetime_total_ms - state.lifetime_remaining_ms) / 1000.0;

        // Lifetime progress (0.0 = just spawned, 1.0 = about to expire)
        let lifetime_progress = 1.0 - (state.lifetime_remaining_ms / state.lifetime_total_ms);

        // Pulse intensity fades as particle ages (stronger when young)
        let intensity = (1.0 - lifetime_progress * 0.7).max(0.0);

        // Continuous sine wave based on particle's age
        // Each particle is at a different phase based on when it was spawned
        let phase = age_seconds * PULSE_FREQUENCY * std::f32::consts::TAU;
        let pulse_value = (phase.sin() * 0.5 + 0.5) * intensity; // 0.0 to intensity

        // Apply pulse as scale modifier
        pulse_responder.current_scale_modifier = 1.0 + pulse_value * PULSE_AMPLITUDE;

        // Subtle bloom contribution when pulsing outward
        if pulse_value > 0.3 {
            visual.bloom_contribution = (visual.bloom_contribution + pulse_value * 0.1).min(1.0);
        }
    }
}

/// Applies subtle breathing effect to the background based on bass frequencies.
///
/// Creates a gentle pulse in the background that responds to low-frequency
/// audio content, adding depth and atmosphere without being distracting.
///
/// # Mapping
/// - Input: frequency_bass (0.0 - 1.0)
/// - Output: pulse_intensity (0.0 - 0.15)
pub fn apply_background_pulse(
    time: Res<Time>,
    audio_analysis: Res<AudioAnalysis>,
    mut background: ResMut<CurrentBackground>,
) {
    let dt = time.delta_secs();

    // Map bass frequency to subtle pulse intensity
    // Range is deliberately small (0.0 - 0.15) for subtle breathing effect
    let target_intensity = map_range(audio_analysis.frequency_bass, 0.0, 1.0, 0.0, 0.15);

    // Apply smoothing for gradual, organic transitions
    // Using higher smoothing (0.3) for slower, more meditative response
    background.pulse_intensity = lerp_smooth(background.pulse_intensity, target_intensity, 0.3, dt);
}

// =============================================================================
// AMBIENT AUDIO SYSTEMS
// =============================================================================

/// Resource to hold the pre-loaded audio handle (doesn't start playback).
#[derive(Resource)]
pub struct AmbientAudioHandle(pub Handle<AudioSource>);

/// Pre-loads the ambient audio asset without starting playback.
///
/// Audio playback is deferred until particles exceed the threshold,
/// preventing the app from stealing audio focus on launch.
pub fn preload_ambient_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Pre-loading ambient audio loop");
    let audio_handle: Handle<AudioSource> = asset_server.load("audio/loop.wav");
    commands.insert_resource(AmbientAudioHandle(audio_handle));
}

/// Manages ambient audio playback based on particle count.
///
/// - Starts playback only when particles exceed threshold (avoids stealing audio focus)
/// - Stops playback when particles drop to zero (releases audio focus)
/// - Volume scales with particle count for smooth blending with device audio
pub fn update_ambient_audio(
    mut commands: Commands,
    time: Res<Time>,
    particle_pool: Res<ParticlePool>,
    mut ambient_state: ResMut<AmbientAudioState>,
    audio_handle: Option<Res<AmbientAudioHandle>>,
    audio_sinks: Query<&AudioSink>,
) {
    let dt = time.delta_secs();
    let active = particle_pool.active_count;

    // Calculate target volume based on particle count
    let target = if active < ambient_state.particle_threshold {
        0.0
    } else {
        let particles_above_threshold = active - ambient_state.particle_threshold;
        let range = ambient_state.particle_full_volume - ambient_state.particle_threshold;
        let progress = (particles_above_threshold as f32 / range as f32).clamp(0.0, 1.0);
        progress * ambient_state.max_volume
    };

    ambient_state.target_volume = target;

    // Handle audio entity lifecycle based on whether we need audio
    let needs_audio = target > 0.001;
    let has_audio = ambient_state.audio_entity.is_some();

    if needs_audio && !has_audio {
        // Start audio playback - particles have crossed threshold
        if let Some(handle) = &audio_handle {
            info!("Starting ambient audio (particles: {})", active);
            let entity = commands
                .spawn((
                    AudioPlayer::<AudioSource>(handle.0.clone()),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: bevy::audio::Volume::new(0.0), // Start silent, will fade in
                        ..default()
                    },
                    Name::new("AmbientAudioLoop"),
                ))
                .id();
            ambient_state.audio_entity = Some(entity);
        }
    } else if !needs_audio && has_audio && ambient_state.current_volume < 0.001 {
        // Stop audio playback - volume has faded to zero, release audio focus
        if let Some(entity) = ambient_state.audio_entity.take() {
            info!("Stopping ambient audio (releasing audio focus)");
            commands.entity(entity).despawn();
        }
    }

    // Smooth volume transitions
    let smoothing = if target > ambient_state.current_volume {
        0.3 // Slower fade in
    } else {
        0.15 // Faster fade out
    };

    ambient_state.current_volume = lerp_smooth(
        ambient_state.current_volume,
        target,
        smoothing,
        dt,
    );

    // Apply volume to audio sink if it exists
    if let Some(entity) = ambient_state.audio_entity {
        if let Ok(sink) = audio_sinks.get(entity) {
            sink.set_volume(ambient_state.current_volume);
        }
    }
}

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that registers all audio-reactive systems and events.
///
/// This plugin handles:
/// - Audio input processing (placeholder for FFT integration)
/// - Beat detection and event emission
/// - Audio-to-spawn-rate mapping
/// - Particle visual modulation based on audio
/// - Pulse effects synchronized with beats
/// - Background breathing effects
/// - Ambient audio loop with volume based on particle count
pub struct AudioReactivePlugin;

impl Plugin for AudioReactivePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<BeatDetected>()
            // Startup: pre-load ambient audio (doesn't start playback)
            .add_systems(Startup, preload_ambient_audio)
            // Add systems with proper ordering
            .add_systems(
                Update,
                (
                    // Audio processing chain (high priority, runs first)
                    process_audio_input,
                    detect_beats.after(process_audio_input),
                    // Spawn rate mapping (after beat detection)
                    apply_audio_to_spawn_rate.after(detect_beats),
                    // Visual systems (can run in parallel after audio processing)
                    apply_audio_to_visuals.after(detect_beats),
                    apply_pulse_effect.after(apply_audio_to_visuals),
                    apply_background_pulse.after(detect_beats),
                    // Ambient audio management (starts/stops based on particles)
                    update_ambient_audio,
                ),
            );
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_smooth_instant() {
        // With smoothing = 0, should approach target quickly
        let result = lerp_smooth(0.0, 1.0, 0.0, 0.016);
        assert!(
            (result - 1.0).abs() < 0.01,
            "Expected near 1.0, got {}",
            result
        );
    }

    #[test]
    fn test_lerp_smooth_gradual() {
        // With smoothing = 0.5, should move partway toward target
        let result = lerp_smooth(0.0, 1.0, 0.5, 0.016);
        assert!(
            result > 0.0 && result < 1.0,
            "Expected between 0 and 1, got {}",
            result
        );
    }

    #[test]
    fn test_lerp_smooth_no_change_when_equal() {
        let result = lerp_smooth(0.5, 0.5, 0.5, 0.016);
        assert!(
            (result - 0.5).abs() < f32::EPSILON,
            "Expected 0.5, got {}",
            result
        );
    }

    #[test]
    fn test_map_range_basic() {
        // Map 0.5 from [0, 1] to [0, 100]
        let result = map_range(0.5, 0.0, 1.0, 0.0, 100.0);
        assert!(
            (result - 50.0).abs() < f32::EPSILON,
            "Expected 50.0, got {}",
            result
        );
    }

    #[test]
    fn test_map_range_clamping() {
        // Values outside input range should be clamped
        let result_low = map_range(-1.0, 0.0, 1.0, 0.0, 100.0);
        let result_high = map_range(2.0, 0.0, 1.0, 0.0, 100.0);

        assert!(
            (result_low - 0.0).abs() < f32::EPSILON,
            "Expected 0.0, got {}",
            result_low
        );
        assert!(
            (result_high - 100.0).abs() < f32::EPSILON,
            "Expected 100.0, got {}",
            result_high
        );
    }

    #[test]
    fn test_map_range_inverted() {
        // Inverted output range should work
        let result = map_range(0.5, 0.0, 1.0, 100.0, 0.0);
        assert!(
            (result - 50.0).abs() < f32::EPSILON,
            "Expected 50.0, got {}",
            result
        );
    }

    #[test]
    fn test_map_range_zero_input_range() {
        // Zero-width input range should return out_min
        let result = map_range(5.0, 5.0, 5.0, 0.0, 100.0);
        assert!(
            (result - 0.0).abs() < f32::EPSILON,
            "Expected 0.0, got {}",
            result
        );
    }

    #[test]
    fn test_classify_beat_strength_silence() {
        assert_eq!(classify_beat_strength(0.0), BeatStrength::Silence);
        assert_eq!(classify_beat_strength(0.05), BeatStrength::Silence);
        assert_eq!(classify_beat_strength(0.099), BeatStrength::Silence);
    }

    #[test]
    fn test_classify_beat_strength_soft() {
        assert_eq!(classify_beat_strength(0.1), BeatStrength::Soft);
        assert_eq!(classify_beat_strength(0.2), BeatStrength::Soft);
        assert_eq!(classify_beat_strength(0.299), BeatStrength::Soft);
    }

    #[test]
    fn test_classify_beat_strength_medium() {
        assert_eq!(classify_beat_strength(0.3), BeatStrength::Medium);
        assert_eq!(classify_beat_strength(0.45), BeatStrength::Medium);
        assert_eq!(classify_beat_strength(0.599), BeatStrength::Medium);
    }

    #[test]
    fn test_classify_beat_strength_strong() {
        assert_eq!(classify_beat_strength(0.6), BeatStrength::Strong);
        assert_eq!(classify_beat_strength(0.8), BeatStrength::Strong);
        assert_eq!(classify_beat_strength(1.0), BeatStrength::Strong);
    }

    #[test]
    fn test_get_act_intensity_factor() {
        assert_eq!(get_act_intensity_factor(&Act::Emergence), 0.3);
        assert_eq!(get_act_intensity_factor(&Act::Accumulation), 0.6);
        assert_eq!(get_act_intensity_factor(&Act::Crescendo), 1.0);
        assert_eq!(get_act_intensity_factor(&Act::Release), 0.7);
        assert_eq!(get_act_intensity_factor(&Act::Transcendence), 0.4);
    }

    #[test]
    fn test_get_amplitude_for_band() {
        let analysis = AudioAnalysis {
            amplitude_low: 0.1,
            amplitude_mid: 0.5,
            amplitude_high: 0.7,
            frequency_shimmer: 0.3,
            ..Default::default()
        };

        assert_eq!(get_amplitude_for_band(&analysis, FrequencyBand::Bass), 0.1);
        assert_eq!(get_amplitude_for_band(&analysis, FrequencyBand::Mid), 0.5);
        assert_eq!(get_amplitude_for_band(&analysis, FrequencyBand::High), 0.7);
        assert_eq!(
            get_amplitude_for_band(&analysis, FrequencyBand::Shimmer),
            0.3
        );
    }
}
