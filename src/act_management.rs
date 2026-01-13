//! Module: act_management
//! Purpose: Manages the five-act narrative structure and state transitions for Chromatic Elegy
//! Dependencies: types, resources, bevy::prelude

use bevy::prelude::*;

use crate::intro::AppState;
use crate::resources::{
    ActState, ActTimings, BackgroundGradients, CurrentBackground,
    CurrentInteractionMode, InterpolatedActValues, PostProcessSettings,
};
use crate::types::{Act, TOTAL_DURATION_SECONDS};
use crate::interaction::HyperspaceJumpEvent;

// =============================================================================
// EVENTS
// =============================================================================

/// Event sent when an act transition begins.
///
/// This event is fired at the start of a transition between acts,
/// allowing systems to prepare for visual changes.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActTransitionStarted {
    /// The act being transitioned from
    pub from: Act,
    /// The act being transitioned to
    pub to: Act,
}

/// Event sent when an act transition completes.
///
/// This event is fired when the transition interpolation reaches 1.0,
/// indicating the new act is now fully active.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActTransitionCompleted {
    /// The newly active act
    pub act: Act,
}

// =============================================================================
// ACT-SPECIFIC CONSTANTS
// =============================================================================

/// Saturation multipliers for each act.
/// Index corresponds to Act enum: Emergence=0, Accumulation=1, etc.
const ACT_SATURATION: [f32; 5] = [0.4, 0.6, 1.0, 0.7, 0.3];

/// Target particle density for each act.
/// Index corresponds to Act enum: Emergence=0, Accumulation=1, etc.
const ACT_DENSITY: [f32; 5] = [200.0, 1000.0, 5000.0, 2000.0, 100.0];

/// Chromatic aberration strength for each act.
/// Increases dramatically in Act III (Crescendo).
const ACT_CHROMATIC_ABERRATION: [f32; 5] = [0.0, 0.002, 0.008, 0.004, 0.001];

/// Vignette intensity for each act.
/// Strong early (intimate), dissolves by Act V (open, transcendent).
const ACT_VIGNETTE: [f32; 5] = [0.5, 0.4, 0.35, 0.2, 0.05];

/// Bloom intensity for each act following the emotional arc.
/// Peaks at Crescendo, gentle at Emergence, luminous at Transcendence.
const ACT_BLOOM: [f32; 5] = [0.2, 0.35, 0.6, 0.45, 0.5];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Performs smooth ease-in-out-cubic interpolation.
///
/// This easing function provides a natural acceleration and deceleration
/// curve for smooth visual transitions between acts.
///
/// # Arguments
/// * `t` - Input value in range [0.0, 1.0]
///
/// # Returns
/// Eased value in range [0.0, 1.0]
#[inline]
#[must_use]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Linearly interpolates between two f32 values.
///
/// # Arguments
/// * `a` - Start value
/// * `b` - End value
/// * `t` - Interpolation factor (0.0 = a, 1.0 = b)
#[inline]
#[must_use]
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linearly interpolates between two colors.
///
/// Interpolation is performed in sRGB space for each component.
///
/// # Arguments
/// * `a` - Start color
/// * `b` - End color
/// * `t` - Interpolation factor (0.0 = a, 1.0 = b)
#[inline]
#[must_use]
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a_srgb = a.to_srgba();
    let b_srgb = b.to_srgba();
    Color::srgba(
        lerp_f32(a_srgb.red, b_srgb.red, t),
        lerp_f32(a_srgb.green, b_srgb.green, t),
        lerp_f32(a_srgb.blue, b_srgb.blue, t),
        lerp_f32(a_srgb.alpha, b_srgb.alpha, t),
    )
}

// =============================================================================
// SYSTEMS
// =============================================================================

/// Updates the act progression based on elapsed time.
///
/// This system:
/// - Advances `total_elapsed_seconds` using the Bevy Time resource
/// - Determines the current act from ActTimings boundaries
/// - Sets `is_transitioning` and `transition_progress` during act changes
/// - Sends `ActTransitionStarted` and `ActTransitionCompleted` events
///
/// # Priority
/// HIGH - Must run before other act-dependent systems.
pub fn update_act_progression(
    time: Res<Time>,
    mut act_state: ResMut<ActState>,
    act_timings: Res<ActTimings>,
    mut transition_started_events: EventWriter<ActTransitionStarted>,
    mut transition_completed_events: EventWriter<ActTransitionCompleted>,
    mut hyperspace_events: EventWriter<HyperspaceJumpEvent>,
) {
    // Advance elapsed time
    act_state.total_elapsed_seconds += time.delta_secs();

    // Cycle back to beginning when reaching the end (fidget app loop)
    if act_state.total_elapsed_seconds >= TOTAL_DURATION_SECONDS + 2.0 {
        // Trigger hyperspace effect at screen center before cycling
        hyperspace_events.send(HyperspaceJumpEvent {
            vanishing_point: Vec2::ZERO,
        });

        // Reset to beginning
        act_state.total_elapsed_seconds = 0.0;
        act_state.current_act = Act::Emergence;
        act_state.act_progress = 0.0;
        act_state.is_transitioning = false;
        act_state.transition_progress = 0.0;

        info!("Experience cycling back to Act I: Emergence");
    }

    // Determine current act from elapsed time
    let elapsed = act_state.total_elapsed_seconds;
    let boundaries = &act_timings.act_boundaries_seconds;
    let transition_duration_secs = act_timings.transition_duration_ms / 1000.0;

    let new_act = Act::from_elapsed_seconds(elapsed);
    let act_index = new_act.index();

    // Calculate progress within current act
    let act_start = boundaries[act_index];
    let act_end = boundaries[(act_index + 1).min(5)];
    let act_duration = act_end - act_start;

    if act_duration > 0.0 {
        act_state.act_progress = ((elapsed - act_start) / act_duration).clamp(0.0, 1.0);
    } else {
        act_state.act_progress = 1.0;
    }

    // Handle act transitions
    if new_act != act_state.current_act {
        // Starting a new transition
        let from_act = act_state.current_act;

        if !act_state.is_transitioning {
            // Begin transition
            act_state.is_transitioning = true;
            act_state.transition_progress = 0.0;

            transition_started_events.send(ActTransitionStarted {
                from: from_act,
                to: new_act,
            });
        }

        // Update transition progress
        let time_into_new_act = elapsed - boundaries[act_index];
        act_state.transition_progress = (time_into_new_act / transition_duration_secs).clamp(0.0, 1.0);

        // Check if transition completed
        if act_state.transition_progress >= 1.0 {
            act_state.is_transitioning = false;
            act_state.transition_progress = 1.0;
            act_state.current_act = new_act;

            transition_completed_events.send(ActTransitionCompleted { act: new_act });
        } else {
            // Still transitioning, but update current act reference
            act_state.current_act = new_act;
        }
    } else if act_state.is_transitioning {
        // Continue ongoing transition
        let time_into_act = elapsed - boundaries[act_index];
        act_state.transition_progress = (time_into_act / transition_duration_secs).clamp(0.0, 1.0);

        if act_state.transition_progress >= 1.0 {
            act_state.is_transitioning = false;
            act_state.transition_progress = 1.0;

            transition_completed_events.send(ActTransitionCompleted {
                act: act_state.current_act,
            });
        }
    }
}

/// Interpolates act-dependent values during transitions.
///
/// This system:
/// - Reads ActState, ActTimings, ColorPalette, BackgroundGradients
/// - Writes to InterpolatedActValues, CurrentInteractionMode, CurrentBackground
/// - Uses smooth ease-in-out-cubic interpolation during transitions
/// - Sets particle_behavior, interaction_mode, saturation_multiplier, density_target per act
///
/// # Ordering
/// Runs after `update_act_progression`.
pub fn interpolate_act_values(
    act_state: Res<ActState>,
    background_gradients: Res<BackgroundGradients>,
    mut interpolated_values: ResMut<InterpolatedActValues>,
    mut current_interaction_mode: ResMut<CurrentInteractionMode>,
    mut current_background: ResMut<CurrentBackground>,
) {
    let current_act = act_state.current_act;
    let act_index = current_act.index();

    if act_state.is_transitioning {
        // Get previous act for interpolation
        let prev_act = current_act.previous().unwrap_or(current_act);
        let prev_index = prev_act.index();

        // Apply easing to transition progress
        let t = ease_in_out_cubic(act_state.transition_progress);

        // Interpolate saturation and density
        interpolated_values.saturation_multiplier = lerp_f32(
            ACT_SATURATION[prev_index],
            ACT_SATURATION[act_index],
            t,
        );
        interpolated_values.density_target = lerp_f32(
            ACT_DENSITY[prev_index],
            ACT_DENSITY[act_index],
            t,
        );

        // Interpolate background colors
        let prev_gradient = &background_gradients.act_gradients[prev_index];
        let curr_gradient = &background_gradients.act_gradients[act_index];

        interpolated_values.background_color_start = lerp_color(
            prev_gradient[0],
            curr_gradient[0],
            t,
        );
        interpolated_values.background_color_end = lerp_color(
            prev_gradient[1],
            curr_gradient[1],
            t,
        );

        current_background.gradient_start = interpolated_values.background_color_start;
        current_background.gradient_end = interpolated_values.background_color_end;

        // For behavior and interaction mode, switch at halfway point
        if t < 0.5 {
            interpolated_values.particle_behavior = prev_act.default_behavior();
            interpolated_values.interaction_mode = prev_act.default_interaction_mode();
            current_interaction_mode.mode = prev_act.default_interaction_mode();
        } else {
            interpolated_values.particle_behavior = current_act.default_behavior();
            interpolated_values.interaction_mode = current_act.default_interaction_mode();
            current_interaction_mode.mode = current_act.default_interaction_mode();
        }
    } else {
        // Not transitioning - use current act values directly
        interpolated_values.saturation_multiplier = ACT_SATURATION[act_index];
        interpolated_values.density_target = ACT_DENSITY[act_index];
        interpolated_values.particle_behavior = current_act.default_behavior();
        interpolated_values.interaction_mode = current_act.default_interaction_mode();

        // Set background from current act gradient
        let gradient = &background_gradients.act_gradients[act_index];
        interpolated_values.background_color_start = gradient[0];
        interpolated_values.background_color_end = gradient[1];

        current_background.gradient_start = gradient[0];
        current_background.gradient_end = gradient[1];

        current_interaction_mode.mode = current_act.default_interaction_mode();
    }
}

/// Updates post-processing settings based on act state.
///
/// This system adjusts:
/// - Chromatic aberration: increases in Act III (Crescendo)
/// - Vignette: strong early, dissolves by Act V (Transcendence)
/// - Bloom intensity: follows the emotional arc
///
/// # Ordering
/// Runs after `interpolate_act_values`.
pub fn update_post_process_for_act(
    act_state: Res<ActState>,
    mut post_process: ResMut<PostProcessSettings>,
) {
    let current_act = act_state.current_act;
    let act_index = current_act.index();

    if act_state.is_transitioning {
        // Get previous act for interpolation
        let prev_act = current_act.previous().unwrap_or(current_act);
        let prev_index = prev_act.index();

        // Apply easing to transition progress
        let t = ease_in_out_cubic(act_state.transition_progress);

        // Interpolate post-processing values
        post_process.chromatic_aberration_strength = lerp_f32(
            ACT_CHROMATIC_ABERRATION[prev_index],
            ACT_CHROMATIC_ABERRATION[act_index],
            t,
        );

        post_process.vignette_intensity = lerp_f32(
            ACT_VIGNETTE[prev_index],
            ACT_VIGNETTE[act_index],
            t,
        );

        post_process.bloom_intensity = lerp_f32(
            ACT_BLOOM[prev_index],
            ACT_BLOOM[act_index],
            t,
        );
    } else {
        // Not transitioning - use current act values directly
        post_process.chromatic_aberration_strength = ACT_CHROMATIC_ABERRATION[act_index];
        post_process.vignette_intensity = ACT_VIGNETTE[act_index];
        post_process.bloom_intensity = ACT_BLOOM[act_index];
    }
}

// =============================================================================
// SYSTEM SETS
// =============================================================================

/// System set for act management systems, ensuring proper ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActManagementSet {
    /// Update act progression from elapsed time
    UpdateProgression,
    /// Interpolate act-dependent values
    InterpolateValues,
    /// Update post-processing for current act
    UpdatePostProcess,
}

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that manages the five-act narrative structure.
///
/// This plugin registers:
/// - Act transition events (`ActTransitionStarted`, `ActTransitionCompleted`)
/// - Systems for progression, interpolation, and post-processing updates
/// - Proper system ordering to ensure consistent state
///
/// # Systems
/// - `update_act_progression` - Advances time and determines current act
/// - `interpolate_act_values` - Smoothly transitions act-dependent values
/// - `update_post_process_for_act` - Adjusts visual effects per act
pub struct ActManagementPlugin;

impl Plugin for ActManagementPlugin {
    fn build(&self, app: &mut App) {
        // Register events
        app.add_event::<ActTransitionStarted>()
            .add_event::<ActTransitionCompleted>();

        // Configure system sets (only run in Fidget state)
        app.configure_sets(
            Update,
            (
                ActManagementSet::UpdateProgression,
                ActManagementSet::InterpolateValues,
                ActManagementSet::UpdatePostProcess,
            )
                .chain()
                .run_if(in_state(AppState::Fidget)),
        );

        // Add systems with ordering
        app.add_systems(
            Update,
            (
                update_act_progression.in_set(ActManagementSet::UpdateProgression),
                interpolate_act_values.in_set(ActManagementSet::InterpolateValues),
                update_post_process_for_act.in_set(ActManagementSet::UpdatePostProcess),
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
    fn test_ease_in_out_cubic() {
        // Test boundary values
        assert!((ease_in_out_cubic(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((ease_in_out_cubic(1.0) - 1.0).abs() < f32::EPSILON);

        // Test midpoint
        assert!((ease_in_out_cubic(0.5) - 0.5).abs() < f32::EPSILON);

        // Test symmetry: f(0.25) + f(0.75) should equal 1.0
        let quarter = ease_in_out_cubic(0.25);
        let three_quarter = ease_in_out_cubic(0.75);
        assert!((quarter + three_quarter - 1.0).abs() < 0.001);

        // Test clamping
        assert!((ease_in_out_cubic(-0.5) - 0.0).abs() < f32::EPSILON);
        assert!((ease_in_out_cubic(1.5) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_f32() {
        assert!((lerp_f32(0.0, 100.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((lerp_f32(0.0, 100.0, 1.0) - 100.0).abs() < f32::EPSILON);
        assert!((lerp_f32(0.0, 100.0, 0.5) - 50.0).abs() < f32::EPSILON);
        assert!((lerp_f32(10.0, 20.0, 0.25) - 12.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_color() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        let mid = lerp_color(black, white, 0.5);
        let mid_srgb = mid.to_srgba();

        assert!((mid_srgb.red - 0.5).abs() < 0.001);
        assert!((mid_srgb.green - 0.5).abs() < 0.001);
        assert!((mid_srgb.blue - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_act_constants_length() {
        // Ensure all act constant arrays have correct length
        assert_eq!(ACT_SATURATION.len(), 5);
        assert_eq!(ACT_DENSITY.len(), 5);
        assert_eq!(ACT_CHROMATIC_ABERRATION.len(), 5);
        assert_eq!(ACT_VIGNETTE.len(), 5);
        assert_eq!(ACT_BLOOM.len(), 5);
    }

    #[test]
    fn test_act_saturation_values() {
        // Verify saturation values match requirements
        assert!((ACT_SATURATION[0] - 0.4).abs() < f32::EPSILON); // Act I
        assert!((ACT_SATURATION[1] - 0.6).abs() < f32::EPSILON); // Act II
        assert!((ACT_SATURATION[2] - 1.0).abs() < f32::EPSILON); // Act III
        assert!((ACT_SATURATION[3] - 0.7).abs() < f32::EPSILON); // Act IV
        assert!((ACT_SATURATION[4] - 0.3).abs() < f32::EPSILON); // Act V
    }

    #[test]
    fn test_act_density_values() {
        // Verify density values match requirements
        assert!((ACT_DENSITY[0] - 200.0).abs() < f32::EPSILON);  // Act I
        assert!((ACT_DENSITY[1] - 1000.0).abs() < f32::EPSILON); // Act II
        assert!((ACT_DENSITY[2] - 5000.0).abs() < f32::EPSILON); // Act III
        assert!((ACT_DENSITY[3] - 2000.0).abs() < f32::EPSILON); // Act IV
        assert!((ACT_DENSITY[4] - 100.0).abs() < f32::EPSILON);  // Act V
    }

    #[test]
    fn test_chromatic_aberration_peaks_at_crescendo() {
        // Act III (Crescendo) should have highest chromatic aberration
        let crescendo_index = Act::Crescendo.index();
        for (i, &value) in ACT_CHROMATIC_ABERRATION.iter().enumerate() {
            if i != crescendo_index {
                assert!(
                    ACT_CHROMATIC_ABERRATION[crescendo_index] > value,
                    "Crescendo should have highest chromatic aberration"
                );
            }
        }
    }

    #[test]
    fn test_vignette_decreases_toward_end() {
        // Vignette should be strongest in Act I and weakest in Act V
        assert!(ACT_VIGNETTE[0] > ACT_VIGNETTE[4]);
        // Should generally decrease
        assert!(ACT_VIGNETTE[0] >= ACT_VIGNETTE[1]);
        assert!(ACT_VIGNETTE[3] >= ACT_VIGNETTE[4]);
    }

    #[test]
    fn test_act_transition_started_event() {
        let event = ActTransitionStarted {
            from: Act::Emergence,
            to: Act::Accumulation,
        };
        assert_eq!(event.from, Act::Emergence);
        assert_eq!(event.to, Act::Accumulation);
    }

    #[test]
    fn test_act_transition_completed_event() {
        let event = ActTransitionCompleted {
            act: Act::Crescendo,
        };
        assert_eq!(event.act, Act::Crescendo);
    }
}
