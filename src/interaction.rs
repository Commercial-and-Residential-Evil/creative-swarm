//! Module: interaction
//! Purpose: Mouse, keyboard, and touch input handling for particle interaction across all acts
//! Dependencies: bevy, crate::types, crate::resources, crate::components

use bevy::prelude::*;
use bevy::input::touch::Touches;
use bevy::window::PrimaryWindow;

use crate::components::{MouseInfluence, Particle, ParticleMotion, ParticleState, ParticleVisual};
use crate::resources::{CurrentInteractionMode, InteractionConfig, MouseState};
use crate::types::InteractionMode;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Cooldown period for breath pulse trigger in seconds.
const BREATH_PULSE_COOLDOWN_SECONDS: f32 = 0.4;

/// Duration of the gentle fade exit sequence in seconds.
const GENTLE_FADE_DURATION_SECONDS: f32 = 3.0;

/// Base attraction force strength for Attract mode.
const ATTRACT_FORCE_BASE: f32 = 80.0;

/// Base repulsion force strength for Disperse mode.
const DISPERSE_FORCE_BASE: f32 = 100.0;

/// Upward velocity bias added during Disperse mode.
const DISPERSE_UPWARD_BIAS: f32 = 30.0;

/// Saturation boost for Intensify mode.
const INTENSIFY_SATURATION_BOOST: f32 = 0.3;

/// Scale boost for Intensify mode.
const INTENSIFY_SCALE_BOOST: f32 = 0.15;

/// Ripple force strength for Ripple mode.
const RIPPLE_FORCE_BASE: f32 = 25.0;

/// Velocity threshold below which mouse is considered stationary (pixels/second).
const VELOCITY_THRESHOLD_LOW: f32 = 50.0;

/// Velocity threshold at which mouse influence is maximum (pixels/second).
const VELOCITY_THRESHOLD_HIGH: f32 = 400.0;

/// Explosion force strength when left-clicking.
const EXPLOSION_FORCE: f32 = 800.0;

/// Explosion radius of effect.
const EXPLOSION_RADIUS: f32 = 400.0;

/// Hyperspace acceleration strength.
const HYPERSPACE_ACCELERATION: f32 = 2000.0;

/// Duration of hyperspace effect in seconds.
const HYPERSPACE_DURATION: f32 = 1.5;

/// Maximum time for a touch to count as a "tap" (seconds).
const TAP_MAX_DURATION: f32 = 0.3;

/// Maximum distance a touch can move and still count as a tap (pixels).
const TAP_MAX_DISTANCE: f32 = 30.0;

/// Minimum time for a press-and-hold to trigger explosion (seconds).
const HOLD_MIN_DURATION: f32 = 0.5;

/// Time window to detect a second finger for two-finger tap (seconds).
#[allow(dead_code)]
const TWO_FINGER_WINDOW: f32 = 0.15;

// =============================================================================
// EVENTS
// =============================================================================

/// Event triggered when the user presses spacebar to create a breath pulse.
///
/// This causes a concentric wave to expand from the screen center (or mouse position),
/// making all particles pulse outward then contract.
#[derive(Event, Debug, Clone, Copy)]
pub struct BreathPulse {
    /// World position where the pulse originates.
    pub origin: Vec2,
    /// Strength of the pulse (0.0 to 1.0).
    pub strength: f32,
}

impl Default for BreathPulse {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            strength: 1.0,
        }
    }
}

/// Event triggered when the user presses Escape to initiate a graceful exit.
///
/// This begins a gentle fade sequence where particle spawning ceases,
/// existing particles decay faster, and the application closes after the fade completes.
#[derive(Event, Debug, Clone, Copy)]
pub struct GentleFade {
    /// Duration of the fade in seconds.
    pub duration_seconds: f32,
}

impl Default for GentleFade {
    fn default() -> Self {
        Self {
            duration_seconds: GENTLE_FADE_DURATION_SECONDS,
        }
    }
}

/// Event triggered when the user left-clicks to create an explosion effect.
///
/// Particles near the click position are forcefully pushed away radially.
#[derive(Event, Debug, Clone, Copy)]
pub struct ExplosionEvent {
    /// World position where the explosion originates.
    pub origin: Vec2,
    /// Strength multiplier for the explosion force.
    pub strength: f32,
}

/// Event triggered when the user right-clicks to initiate hyperspace jump.
///
/// All particles stretch into line segments and accelerate away from the
/// vanishing point (click position), creating a Star Wars hyperspace effect.
#[derive(Event, Debug, Clone, Copy)]
pub struct HyperspaceJumpEvent {
    /// World position that serves as the perspective vanishing point.
    pub vanishing_point: Vec2,
}

// =============================================================================
// RESOURCES
// =============================================================================

/// Tracks the cooldown state for breath pulse input.
#[derive(Resource, Debug, Clone)]
pub struct BreathPulseCooldown {
    /// Remaining cooldown time in seconds.
    pub remaining_seconds: f32,
}

impl Default for BreathPulseCooldown {
    fn default() -> Self {
        Self {
            remaining_seconds: 0.0,
        }
    }
}

/// Tracks the state of the gentle fade exit sequence.
#[derive(Resource, Debug, Clone)]
pub struct GentleFadeState {
    /// Whether the fade sequence is active.
    pub is_active: bool,
    /// Remaining time until application exit.
    pub remaining_seconds: f32,
    /// Total duration of the fade sequence.
    pub total_duration_seconds: f32,
}

impl Default for GentleFadeState {
    fn default() -> Self {
        Self {
            is_active: false,
            remaining_seconds: 0.0,
            total_duration_seconds: GENTLE_FADE_DURATION_SECONDS,
        }
    }
}

impl GentleFadeState {
    /// Returns the fade progress from 0.0 (just started) to 1.0 (complete).
    #[must_use]
    pub fn progress(&self) -> f32 {
        if !self.is_active || self.total_duration_seconds <= 0.0 {
            return 0.0;
        }
        1.0 - (self.remaining_seconds / self.total_duration_seconds).clamp(0.0, 1.0)
    }
}

/// Tracks the state of an active hyperspace jump effect.
#[derive(Resource, Debug, Clone)]
pub struct HyperspaceState {
    /// Whether the hyperspace effect is currently active.
    pub is_active: bool,
    /// The vanishing point (perspective origin) for the effect.
    pub vanishing_point: Vec2,
    /// Remaining time for the effect.
    pub remaining_seconds: f32,
    /// Total duration of the effect.
    pub total_duration: f32,
}

impl Default for HyperspaceState {
    fn default() -> Self {
        Self {
            is_active: false,
            vanishing_point: Vec2::ZERO,
            remaining_seconds: 0.0,
            total_duration: HYPERSPACE_DURATION,
        }
    }
}

impl HyperspaceState {
    /// Returns the effect progress from 0.0 (just started) to 1.0 (complete).
    #[must_use]
    pub fn progress(&self) -> f32 {
        if !self.is_active || self.total_duration <= 0.0 {
            return 0.0;
        }
        1.0 - (self.remaining_seconds / self.total_duration).clamp(0.0, 1.0)
    }
}

/// Tracks active touch state for gesture detection.
#[derive(Resource, Debug, Clone)]
pub struct TouchState {
    /// Primary touch ID (first finger down).
    pub primary_touch_id: Option<u64>,
    /// Start position of primary touch (screen coordinates).
    pub primary_start_pos: Vec2,
    /// Current position of primary touch (screen coordinates).
    pub primary_current_pos: Vec2,
    /// Time when primary touch started.
    pub primary_start_time: f32,
    /// Secondary touch ID (second finger for two-finger gestures).
    pub secondary_touch_id: Option<u64>,
    /// Time when secondary touch started.
    pub secondary_start_time: f32,
    /// Number of active touches (current frame).
    pub touch_count: usize,
    /// Peak number of simultaneous touches in current gesture.
    pub peak_touch_count: usize,
    /// Time when we first had 2+ touches.
    pub multi_touch_start_time: f32,
    /// Whether a hold has been triggered (to avoid repeat triggers).
    pub hold_triggered: bool,
    /// Whether a two-finger gesture was already triggered this gesture.
    pub two_finger_triggered: bool,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            primary_touch_id: None,
            primary_start_pos: Vec2::ZERO,
            primary_current_pos: Vec2::ZERO,
            primary_start_time: 0.0,
            secondary_touch_id: None,
            secondary_start_time: 0.0,
            touch_count: 0,
            peak_touch_count: 0,
            multi_touch_start_time: 0.0,
            hold_triggered: false,
            two_finger_triggered: false,
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Calculates quadratic falloff based on distance from the influence center.
///
/// Returns a value in the range [0.0, 1.0] where 1.0 is full influence
/// (at distance 0) and 0.0 is no influence (at or beyond max_distance).
///
/// The quadratic falloff creates a natural, organic feel where influence
/// drops off more rapidly as distance increases.
///
/// # Arguments
/// * `distance` - Current distance from the influence source.
/// * `max_distance` - Maximum distance at which influence reaches zero.
///
/// # Returns
/// Falloff value between 0.0 and 1.0.
#[inline]
#[must_use]
pub fn quadratic_falloff(distance: f32, max_distance: f32) -> f32 {
    if distance >= max_distance || max_distance <= 0.0 {
        return 0.0;
    }
    let normalized = distance / max_distance;
    (1.0 - normalized * normalized).max(0.0)
}

/// Converts a screen position to world coordinates using the camera transform.
///
/// Takes into account the camera's projection and global transform to
/// accurately map 2D screen coordinates to 2D world space.
///
/// # Arguments
/// * `screen_pos` - Position in screen/window coordinates (pixels from top-left).
/// * `camera` - The camera component for projection calculations.
/// * `camera_transform` - The global transform of the camera entity.
///
/// # Returns
/// The corresponding world position, or `None` if conversion fails.
#[must_use]
pub fn world_position_from_screen(
    screen_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    camera
        .viewport_to_world_2d(camera_transform, screen_pos)
        .ok()
}

/// Calculates interaction strength based on mouse velocity.
///
/// Slower movements result in lower strength, faster movements result in higher strength,
/// with linear scaling between the low and high thresholds.
///
/// # Arguments
/// * `velocity` - Current mouse velocity vector.
///
/// # Returns
/// Interaction strength between 0.0 and 1.0.
#[inline]
#[must_use]
fn velocity_to_strength(velocity: Vec2) -> f32 {
    let speed = velocity.length();
    if speed <= VELOCITY_THRESHOLD_LOW {
        0.0
    } else if speed >= VELOCITY_THRESHOLD_HIGH {
        1.0
    } else {
        (speed - VELOCITY_THRESHOLD_LOW) / (VELOCITY_THRESHOLD_HIGH - VELOCITY_THRESHOLD_LOW)
    }
}

// =============================================================================
// SYSTEMS - PreUpdate
// =============================================================================

/// Updates the mouse state resource with current position and velocity.
///
/// This system runs in PreUpdate to ensure mouse state is available for
/// all subsequent interaction systems. It converts window mouse position
/// to world coordinates using the camera transform.
///
/// # Stage
/// PreUpdate
///
/// # Ordering
/// Runs before `calculate_interaction_radius`.
pub fn update_mouse_state(
    mut mouse_state: ResMut<MouseState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    let Ok(window) = windows.get_single() else {
        mouse_state.is_active = false;
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        mouse_state.is_active = false;
        return;
    };

    // Get cursor position if available
    let Some(cursor_position) = window.cursor_position() else {
        mouse_state.is_active = false;
        mouse_state.velocity = Vec2::ZERO;
        return;
    };

    // Convert screen position to world coordinates
    let Some(world_position) = world_position_from_screen(cursor_position, camera, camera_transform)
    else {
        mouse_state.is_active = false;
        return;
    };

    // Calculate velocity from position delta
    let delta = world_position - mouse_state.position;
    let delta_seconds = time.delta_secs();

    if delta_seconds > 0.0 {
        // Smooth velocity calculation to avoid jitter
        let instant_velocity = delta / delta_seconds;
        mouse_state.velocity = mouse_state.velocity.lerp(instant_velocity, 0.3);
    }

    // Update position
    mouse_state.position = world_position;
    mouse_state.is_active = true;

    // Accumulate interaction time when mouse is active and moving
    let velocity_factor = (mouse_state.velocity.length() / 200.0).clamp(0.1, 1.0);
    mouse_state.accumulated_interaction += delta_seconds * velocity_factor;
}

/// Calculates and updates the interaction radius based on accumulated interaction.
///
/// The radius grows from base_radius toward max_radius as the user spends more
/// time interacting with the experience, creating a sense of growing connection.
///
/// # Stage
/// PreUpdate
///
/// # Ordering
/// Runs after `update_mouse_state`.
pub fn calculate_interaction_radius(
    mouse_state: Res<MouseState>,
    mut interaction_config: ResMut<InteractionConfig>,
) {
    // Calculate target radius based on accumulated interaction
    // Growth formula: base + (accumulated / 60) * (max - base)
    // This means full radius is reached after about 60 seconds of active interaction
    let accumulated = mouse_state.accumulated_interaction;
    let base = interaction_config.base_radius;
    let max = interaction_config.max_radius;

    let target_radius = base + (accumulated / 60.0) * (max - base);
    let target_radius = target_radius.clamp(base, max);

    // Smoothly interpolate toward target radius
    interaction_config.current_radius = interaction_config
        .current_radius
        .lerp(target_radius, 0.05);
}

/// Handles keyboard input for breath pulse and gentle exit.
///
/// - Space key: Triggers a BreathPulse event (with 400ms cooldown).
/// - Escape key: Triggers a GentleFade event for graceful exit.
///
/// # Stage
/// PreUpdate
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_state: Res<MouseState>,
    mut breath_cooldown: ResMut<BreathPulseCooldown>,
    mut gentle_fade_state: ResMut<GentleFadeState>,
    mut breath_pulse_events: EventWriter<BreathPulse>,
    mut gentle_fade_events: EventWriter<GentleFade>,
    time: Res<Time>,
) {
    // Update cooldown timer
    breath_cooldown.remaining_seconds = (breath_cooldown.remaining_seconds - time.delta_secs()).max(0.0);

    // Handle spacebar for breath pulse
    if keyboard.just_pressed(KeyCode::Space) && breath_cooldown.remaining_seconds <= 0.0 {
        breath_pulse_events.send(BreathPulse {
            origin: mouse_state.position,
            strength: 1.0,
        });
        breath_cooldown.remaining_seconds = BREATH_PULSE_COOLDOWN_SECONDS;
    }

    // Handle escape for gentle fade (only trigger once)
    if keyboard.just_pressed(KeyCode::Escape) && !gentle_fade_state.is_active {
        gentle_fade_state.is_active = true;
        gentle_fade_state.remaining_seconds = GENTLE_FADE_DURATION_SECONDS;
        gentle_fade_state.total_duration_seconds = GENTLE_FADE_DURATION_SECONDS;

        gentle_fade_events.send(GentleFade::default());
    }
}

/// Handles mouse button clicks for explosion and hyperspace effects.
///
/// - Left click: Triggers an explosion at the cursor position
/// - Right click: Triggers a hyperspace jump with vanishing point at cursor
///
/// # Stage
/// PreUpdate
pub fn handle_mouse_clicks(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    mut hyperspace_state: ResMut<HyperspaceState>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut hyperspace_events: EventWriter<HyperspaceJumpEvent>,
) {
    if !mouse_state.is_active {
        return;
    }

    // Left click: Explosion
    if mouse_button.just_pressed(MouseButton::Left) {
        explosion_events.send(ExplosionEvent {
            origin: mouse_state.position,
            strength: 1.0,
        });
    }

    // Right click: Hyperspace jump (only if not already active)
    if mouse_button.just_pressed(MouseButton::Right) && !hyperspace_state.is_active {
        hyperspace_state.is_active = true;
        hyperspace_state.vanishing_point = mouse_state.position;
        hyperspace_state.remaining_seconds = HYPERSPACE_DURATION;
        hyperspace_state.total_duration = HYPERSPACE_DURATION;

        hyperspace_events.send(HyperspaceJumpEvent {
            vanishing_point: mouse_state.position,
        });
    }
}

/// Updates mouse state from touch input (for mobile/tablet devices).
///
/// Maps single-finger touch position to mouse position, enabling the same
/// particle interaction effects on touch devices.
///
/// # Stage
/// PreUpdate
pub fn update_touch_state(
    mut mouse_state: ResMut<MouseState>,
    mut touch_state: ResMut<TouchState>,
    touches: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let elapsed = time.elapsed_secs();

    // Count active touches
    let active_touches: Vec<_> = touches.iter().collect();
    let prev_touch_count = touch_state.touch_count;
    touch_state.touch_count = active_touches.len();

    // Track peak touch count for gesture detection
    if touch_state.touch_count > touch_state.peak_touch_count {
        touch_state.peak_touch_count = touch_state.touch_count;
        // Record when we first hit 2+ touches
        if touch_state.touch_count >= 2 && prev_touch_count < 2 {
            touch_state.multi_touch_start_time = elapsed;
        }
    }

    // Reset peak and gesture flags when all touches end
    if touch_state.touch_count == 0 && prev_touch_count > 0 {
        touch_state.peak_touch_count = 0;
        touch_state.two_finger_triggered = false;
    }

    // Handle touch start
    for touch in touches.iter_just_pressed() {
        if touch_state.primary_touch_id.is_none() {
            // First finger down
            touch_state.primary_touch_id = Some(touch.id());
            touch_state.primary_start_pos = touch.position();
            touch_state.primary_current_pos = touch.position();
            touch_state.primary_start_time = elapsed;
            touch_state.hold_triggered = false;
        } else if touch_state.secondary_touch_id.is_none() {
            // Second finger down (for two-finger tap)
            touch_state.secondary_touch_id = Some(touch.id());
            touch_state.secondary_start_time = elapsed;
        }
    }

    // Handle touch movement - update position for primary touch
    if let Some(primary_id) = touch_state.primary_touch_id {
        if let Some(touch) = touches.get_pressed(primary_id) {
            let screen_pos = touch.position();
            touch_state.primary_current_pos = screen_pos;

            // Convert to world coordinates and update mouse state
            if let Some(world_pos) = world_position_from_screen(screen_pos, camera, camera_transform) {
                // Calculate velocity
                let delta = world_pos - mouse_state.position;
                let delta_seconds = time.delta_secs();

                if delta_seconds > 0.0 {
                    let instant_velocity = delta / delta_seconds;
                    mouse_state.velocity = mouse_state.velocity.lerp(instant_velocity, 0.3);
                }

                mouse_state.position = world_pos;
                mouse_state.is_active = true;

                // Accumulate interaction time
                let velocity_factor = (mouse_state.velocity.length() / 200.0).clamp(0.1, 1.0);
                mouse_state.accumulated_interaction += delta_seconds * velocity_factor;
            }
        }
    }

    // Handle touch end
    for touch in touches.iter_just_released() {
        if Some(touch.id()) == touch_state.primary_touch_id {
            touch_state.primary_touch_id = None;
            touch_state.hold_triggered = false;

            // Also clear secondary if primary released
            touch_state.secondary_touch_id = None;

            // When no touches, mouse becomes inactive
            if touch_state.touch_count <= 1 {
                mouse_state.is_active = false;
                mouse_state.velocity = Vec2::ZERO;
            }
        } else if Some(touch.id()) == touch_state.secondary_touch_id {
            touch_state.secondary_touch_id = None;
        }
    }

    // Handle cancelled touches (e.g., interrupted by system gesture)
    for touch in touches.iter_just_canceled() {
        if Some(touch.id()) == touch_state.primary_touch_id {
            touch_state.primary_touch_id = None;
            touch_state.hold_triggered = false;
            mouse_state.is_active = false;
        }
        if Some(touch.id()) == touch_state.secondary_touch_id {
            touch_state.secondary_touch_id = None;
        }
    }
}

/// Handles touch gestures for explosion and hyperspace effects.
///
/// - Single tap: Quick tap triggers explosion at tap position
/// - Press and hold: Hold for 0.5s+ triggers explosion
/// - Two-finger tap: Triggers hyperspace jump
///
/// # Stage
/// PreUpdate
pub fn handle_touch_gestures(
    mut touch_state: ResMut<TouchState>,
    mut hyperspace_state: ResMut<HyperspaceState>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut hyperspace_events: EventWriter<HyperspaceJumpEvent>,
    touches: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let elapsed = time.elapsed_secs();

    // Check for two-finger tap (hyperspace)
    // Trigger when: we had 2+ fingers, they're now being released, and it was quick
    let any_just_released = touches.iter_just_released().count() > 0;

    if touch_state.peak_touch_count >= 2
        && any_just_released
        && !touch_state.two_finger_triggered
        && !hyperspace_state.is_active
    {
        let tap_duration = elapsed - touch_state.multi_touch_start_time;

        // Quick two-finger tap triggers hyperspace
        if tap_duration < TAP_MAX_DURATION * 1.5 {
            // Get position from primary touch or center of screen
            let screen_pos = touch_state.primary_current_pos;
            let world_pos = world_position_from_screen(screen_pos, camera, camera_transform);

            if let Some(pos) = world_pos {
                hyperspace_state.is_active = true;
                hyperspace_state.vanishing_point = pos;
                hyperspace_state.remaining_seconds = HYPERSPACE_DURATION;
                hyperspace_state.total_duration = HYPERSPACE_DURATION;

                hyperspace_events.send(HyperspaceJumpEvent {
                    vanishing_point: pos,
                });

                // Mark as triggered so we don't trigger again until all fingers lift
                touch_state.two_finger_triggered = true;

                // Clear touch IDs
                touch_state.primary_touch_id = None;
                touch_state.secondary_touch_id = None;
                return;
            }
        }
    }

    // Check for single-finger tap (explosion)
    // Only process if this was a single-finger gesture (peak_touch_count == 1)
    if let Some(primary_id) = touch_state.primary_touch_id {
        // Only process single-finger gestures (not after multi-touch)
        if touch_state.peak_touch_count <= 1 && touch_state.secondary_touch_id.is_none() {
            let touch_duration = elapsed - touch_state.primary_start_time;
            let touch_distance = (touch_state.primary_current_pos - touch_state.primary_start_pos).length();

            // Check for tap on release
            if touches.just_released(primary_id) {
                if touch_duration < TAP_MAX_DURATION && touch_distance < TAP_MAX_DISTANCE {
                    // Quick tap - explosion!
                    if let Some(world_pos) = world_position_from_screen(
                        touch_state.primary_current_pos,
                        camera,
                        camera_transform,
                    ) {
                        explosion_events.send(ExplosionEvent {
                            origin: world_pos,
                            strength: 1.0,
                        });
                    }
                }
            }

            // Check for press-and-hold (explosion after holding)
            if touch_duration >= HOLD_MIN_DURATION
                && touch_distance < TAP_MAX_DISTANCE
                && !touch_state.hold_triggered
            {
                if let Some(world_pos) = world_position_from_screen(
                    touch_state.primary_current_pos,
                    camera,
                    camera_transform,
                ) {
                    explosion_events.send(ExplosionEvent {
                        origin: world_pos,
                        strength: 1.5, // Stronger explosion for held touch
                    });
                    touch_state.hold_triggered = true;
                }
            }
        }
    }
}

// =============================================================================
// SYSTEMS - Update
// =============================================================================

/// Applies mouse influence to particles based on current interaction mode.
///
/// This is a critical path system that queries all particles within the
/// interaction radius and applies appropriate forces based on the active mode:
///
/// - Paint: No force (handled by spawn system)
/// - Attract: Pull particles toward cursor
/// - Intensify: Increase saturation and scale near cursor
/// - Disperse: Push particles away and upward
/// - Ripple: Gentle outward wave from cursor
///
/// # Stage
/// Update
///
/// # Performance
/// CRITICAL PATH - Iterates over all active particles.
/// Consider spatial acceleration structure for large particle counts.
pub fn apply_mouse_influence(
    mouse_state: Res<MouseState>,
    interaction_config: Res<InteractionConfig>,
    current_mode: Res<CurrentInteractionMode>,
    mut particles: Query<
        (
            &Transform,
            &mut ParticleMotion,
            &mut MouseInfluence,
            &mut ParticleVisual,
            &ParticleState,
        ),
        With<Particle>,
    >,
    time: Res<Time>,
) {
    // Skip if mouse is not active or gentle fade is happening
    if !mouse_state.is_active {
        // Reset influence state for all particles when mouse inactive
        for (_, _, mut influence, _, _) in particles.iter_mut() {
            influence.affected = false;
            influence.influence_strength = 0.0;
        }
        return;
    }

    let cursor_pos = mouse_state.position;
    let radius = interaction_config.current_radius;
    let delta_seconds = time.delta_secs();
    let velocity_strength = velocity_to_strength(mouse_state.velocity);

    // Process each particle
    for (transform, mut motion, mut influence, mut visual, state) in particles.iter_mut() {
        // Skip inactive particles
        if !state.active {
            influence.affected = false;
            influence.influence_strength = 0.0;
            continue;
        }

        let particle_pos = transform.translation.truncate();
        let to_cursor = cursor_pos - particle_pos;
        let distance = to_cursor.length();

        // Update distance tracking
        influence.distance_to_cursor = distance;

        // Check if within influence radius
        if distance >= radius {
            influence.affected = false;
            influence.influence_strength = 0.0;
            continue;
        }

        // Calculate influence strength with quadratic falloff
        let falloff = quadratic_falloff(distance, radius);
        influence.affected = true;
        influence.influence_strength = falloff;

        // Direction from particle to cursor (normalized)
        let direction = if distance > 0.001 {
            to_cursor / distance
        } else {
            Vec2::ZERO
        };

        // Apply mode-specific behavior
        match current_mode.mode {
            InteractionMode::Paint => {
                // Paint mode doesn't apply forces; spawning is handled separately
            }

            InteractionMode::Attract => {
                // Pull particles toward cursor
                let force_strength = ATTRACT_FORCE_BASE * falloff * (0.5 + 0.5 * velocity_strength);
                let force = direction * force_strength * delta_seconds;
                motion.velocity += force;
            }

            InteractionMode::Intensify => {
                // Mild attraction plus visual enhancement
                let mild_attraction = direction * ATTRACT_FORCE_BASE * 0.3 * falloff * delta_seconds;
                motion.velocity += mild_attraction;

                // Boost saturation and scale based on proximity
                let boost_amount = INTENSIFY_SATURATION_BOOST * falloff;
                let scale_boost = INTENSIFY_SCALE_BOOST * falloff;

                // Apply saturation boost by pushing color toward more saturated version
                let current_srgba = visual.current_color.to_srgba();

                // Simple saturation approximation: increase color component differences
                let avg = (current_srgba.red + current_srgba.green + current_srgba.blue) / 3.0;
                let new_red = current_srgba.red + (current_srgba.red - avg) * boost_amount;
                let new_green = current_srgba.green + (current_srgba.green - avg) * boost_amount;
                let new_blue = current_srgba.blue + (current_srgba.blue - avg) * boost_amount;

                visual.current_color = Color::srgba(
                    new_red.clamp(0.0, 1.0),
                    new_green.clamp(0.0, 1.0),
                    new_blue.clamp(0.0, 1.0),
                    current_srgba.alpha,
                );

                // Boost scale
                visual.scale = (visual.scale + scale_boost * delta_seconds).min(2.5);

                // Boost bloom contribution
                visual.bloom_contribution = (visual.bloom_contribution + falloff * 0.2 * delta_seconds).min(1.0);
            }

            InteractionMode::Disperse => {
                // Push particles away from cursor
                let force_strength = DISPERSE_FORCE_BASE * falloff * (0.6 + 0.4 * velocity_strength);
                let repulsion = -direction * force_strength * delta_seconds;

                // Add upward bias
                let upward = Vec2::new(0.0, DISPERSE_UPWARD_BIAS * falloff * delta_seconds);

                motion.velocity += repulsion + upward;

                // Shift colors toward luminous pastels (increase brightness)
                let current_srgba = visual.current_color.to_srgba();
                let lighten_amount = 0.1 * falloff * delta_seconds;
                visual.current_color = Color::srgba(
                    (current_srgba.red + lighten_amount).min(1.0),
                    (current_srgba.green + lighten_amount).min(1.0),
                    (current_srgba.blue + lighten_amount).min(1.0),
                    current_srgba.alpha,
                );
            }

            InteractionMode::Ripple => {
                // Gentle outward wave from cursor
                let wave_strength = RIPPLE_FORCE_BASE * falloff * (0.3 + 0.7 * velocity_strength);

                // Create a ripple effect that pushes particles outward then pulls back
                let ripple_phase = (distance / 80.0 - time.elapsed_secs() * 2.0).sin();
                let ripple_force = -direction * wave_strength * ripple_phase * delta_seconds;

                motion.velocity += ripple_force;

                // Gentle opacity modulation
                let opacity_mod = 0.05 * falloff * ripple_phase;
                visual.opacity = (visual.opacity + opacity_mod * delta_seconds).clamp(0.1, 1.0);
            }
        }
    }
}

/// Updates the gentle fade state and handles application exit.
///
/// When gentle fade is active, this system counts down the remaining time
/// and triggers application exit when complete.
pub fn update_gentle_fade(
    mut gentle_fade_state: ResMut<GentleFadeState>,
    time: Res<Time>,
    mut exit_events: EventWriter<AppExit>,
) {
    if !gentle_fade_state.is_active {
        return;
    }

    gentle_fade_state.remaining_seconds -= time.delta_secs();

    if gentle_fade_state.remaining_seconds <= 0.0 {
        exit_events.send(AppExit::Success);
    }
}

/// Applies explosion effects when ExplosionEvent is received.
///
/// Particles within the explosion radius are forcefully pushed away
/// from the explosion origin with a radial force that falls off with distance.
pub fn apply_explosion(
    mut explosion_events: EventReader<ExplosionEvent>,
    mut particles: Query<
        (&Transform, &mut ParticleMotion, &mut ParticleVisual, &ParticleState),
        With<Particle>,
    >,
) {
    for event in explosion_events.read() {
        let origin = event.origin;
        let strength = event.strength;

        for (transform, mut motion, mut visual, state) in particles.iter_mut() {
            if !state.active {
                continue;
            }

            let particle_pos = transform.translation.truncate();
            let to_particle = particle_pos - origin;
            let distance = to_particle.length();

            // Skip particles outside explosion radius
            if distance >= EXPLOSION_RADIUS || distance < 0.001 {
                continue;
            }

            // Calculate force with inverse-square-ish falloff (but capped near origin)
            let normalized_dist = (distance / EXPLOSION_RADIUS).max(0.1);
            let force_magnitude = EXPLOSION_FORCE * strength * (1.0 - normalized_dist).powi(2);

            // Direction away from explosion origin
            let direction = to_particle / distance;
            let impulse = direction * force_magnitude;

            // Apply velocity impulse
            motion.velocity += impulse;

            // Visual feedback: brief brightness boost
            visual.bloom_contribution = (visual.bloom_contribution + 0.5 * (1.0 - normalized_dist)).min(1.0);

            // Shift color toward white/yellow briefly
            let current_srgba = visual.current_color.to_srgba();
            let flash_amount = 0.3 * (1.0 - normalized_dist);
            visual.current_color = Color::srgba(
                (current_srgba.red + flash_amount).min(1.0),
                (current_srgba.green + flash_amount * 0.8).min(1.0),
                (current_srgba.blue + flash_amount * 0.3).min(1.0),
                current_srgba.alpha,
            );
        }
    }
}

/// Updates the hyperspace state and applies the effect to particles.
///
/// During hyperspace, particles accelerate away from the vanishing point,
/// creating the Star Wars-style jump to lightspeed effect where stars
/// become elongated streaks rushing past the viewer.
pub fn apply_hyperspace(
    mut hyperspace_state: ResMut<HyperspaceState>,
    mut particles: Query<
        (
            &Transform,
            &mut ParticleMotion,
            &mut ParticleVisual,
            &ParticleState,
        ),
        With<Particle>,
    >,
    time: Res<Time>,
) {
    if !hyperspace_state.is_active {
        return;
    }

    let delta = time.delta_secs();
    hyperspace_state.remaining_seconds -= delta;

    // Check if effect has ended
    if hyperspace_state.remaining_seconds <= 0.0 {
        hyperspace_state.is_active = false;
        return;
    }

    let vanishing_point = hyperspace_state.vanishing_point;
    let progress = hyperspace_state.progress();

    // Acceleration increases as effect progresses (exponential ramp-up for dramatic effect)
    let acceleration_multiplier = 1.0 + progress * progress * 8.0;

    for (transform, mut motion, mut visual, state) in particles.iter_mut() {
        if !state.active {
            continue;
        }

        let particle_pos = transform.translation.truncate();
        let from_vanishing = particle_pos - vanishing_point;
        let distance = from_vanishing.length();

        if distance < 1.0 {
            continue;
        }

        // Direction away from vanishing point (perspective effect)
        let direction = from_vanishing / distance;

        // Apply acceleration away from vanishing point
        // Particles further from center accelerate faster (perspective foreshortening)
        let distance_factor = (distance / 400.0).clamp(0.3, 3.0);
        let acceleration = direction * HYPERSPACE_ACCELERATION * acceleration_multiplier * distance_factor * delta;

        motion.velocity += acceleration;

        // Stretch effect: dramatically reduce drag during hyperspace to maintain velocity
        motion.drag = 0.9995;

        // Visual effect: shift toward white/blue and increase brightness (lightspeed colors)
        let current_srgba = visual.current_color.to_srgba();
        let streak_intensity = progress * 0.4;
        visual.current_color = Color::srgba(
            (current_srgba.red * 0.95 + streak_intensity * 0.6).min(1.0),
            (current_srgba.green * 0.95 + streak_intensity * 0.8).min(1.0),
            (current_srgba.blue + streak_intensity).min(1.0),
            current_srgba.alpha,
        );

        // Boost bloom for the streaking effect
        visual.bloom_contribution = (0.6 + progress * 0.4).min(1.0);

        // Full opacity for maximum visibility of the streaks
        visual.opacity = 1.0;

        // Scale particles slightly larger as they streak
        visual.scale = (visual.scale * (1.0 + progress * 0.3)).min(3.0);
    }
}

// =============================================================================
// SYSTEM SETS
// =============================================================================

/// System set for interaction input processing (PreUpdate).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionInputSet;

/// System set for interaction influence application (Update).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionInfluenceSet;

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that handles all mouse and keyboard interaction for particle effects.
///
/// This plugin registers:
/// - Input systems for mouse state tracking and keyboard handling (PreUpdate)
/// - Influence systems for applying mode-specific particle effects (Update)
/// - Events for breath pulse, gentle fade, explosion, and hyperspace jump
///
/// # Systems
/// - `update_mouse_state` (PreUpdate): Tracks mouse position and velocity
/// - `calculate_interaction_radius` (PreUpdate, after update_mouse_state): Grows radius with use
/// - `handle_keyboard_input` (PreUpdate): Processes spacebar and escape
/// - `handle_mouse_clicks` (PreUpdate): Processes left/right mouse clicks for explosion/hyperspace
/// - `apply_mouse_influence` (Update): Applies mode-specific forces to particles
/// - `apply_explosion` (Update): Applies radial force from explosion events
/// - `apply_hyperspace` (Update): Applies hyperspace acceleration effect
/// - `update_gentle_fade` (Update): Handles graceful exit countdown
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<BreathPulse>()
            .add_event::<GentleFade>()
            .add_event::<ExplosionEvent>()
            .add_event::<HyperspaceJumpEvent>()
            // Register resources
            .init_resource::<BreathPulseCooldown>()
            .init_resource::<GentleFadeState>()
            .init_resource::<HyperspaceState>()
            .init_resource::<TouchState>()
            // Configure system sets
            .configure_sets(
                PreUpdate,
                InteractionInputSet,
            )
            .configure_sets(
                Update,
                InteractionInfluenceSet,
            )
            // Add PreUpdate systems
            .add_systems(
                PreUpdate,
                (
                    update_mouse_state,
                    update_touch_state.after(update_mouse_state),
                    calculate_interaction_radius.after(update_touch_state),
                    handle_keyboard_input,
                    handle_mouse_clicks,
                    handle_touch_gestures.after(update_touch_state),
                )
                    .in_set(InteractionInputSet),
            )
            // Add Update systems
            .add_systems(
                Update,
                (
                    apply_mouse_influence,
                    apply_explosion,
                    apply_hyperspace,
                    update_gentle_fade,
                )
                    .in_set(InteractionInfluenceSet),
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
    fn test_quadratic_falloff() {
        // At distance 0, falloff should be 1.0
        assert!((quadratic_falloff(0.0, 100.0) - 1.0).abs() < 0.001);

        // At half distance, falloff should be 0.75 (1 - 0.25)
        assert!((quadratic_falloff(50.0, 100.0) - 0.75).abs() < 0.001);

        // At max distance, falloff should be 0.0
        assert!((quadratic_falloff(100.0, 100.0) - 0.0).abs() < 0.001);

        // Beyond max distance, falloff should be 0.0
        assert!((quadratic_falloff(150.0, 100.0) - 0.0).abs() < 0.001);

        // Edge cases
        assert_eq!(quadratic_falloff(50.0, 0.0), 0.0);
        assert_eq!(quadratic_falloff(-10.0, 100.0), 0.0); // Negative distance treated as >= max
    }

    #[test]
    fn test_velocity_to_strength() {
        // Below low threshold
        assert_eq!(velocity_to_strength(Vec2::new(10.0, 0.0)), 0.0);
        assert_eq!(velocity_to_strength(Vec2::new(50.0, 0.0)), 0.0);

        // Above high threshold
        assert_eq!(velocity_to_strength(Vec2::new(400.0, 0.0)), 1.0);
        assert_eq!(velocity_to_strength(Vec2::new(500.0, 0.0)), 1.0);

        // Mid-range (225 is midpoint between 50 and 400)
        let mid_strength = velocity_to_strength(Vec2::new(225.0, 0.0));
        assert!(mid_strength > 0.4 && mid_strength < 0.6);
    }

    #[test]
    fn test_breath_pulse_event() {
        let pulse = BreathPulse {
            origin: Vec2::new(100.0, 200.0),
            strength: 0.8,
        };
        assert_eq!(pulse.origin, Vec2::new(100.0, 200.0));
        assert_eq!(pulse.strength, 0.8);
    }

    #[test]
    fn test_gentle_fade_state_progress() {
        let mut state = GentleFadeState::default();
        assert_eq!(state.progress(), 0.0);

        state.is_active = true;
        state.remaining_seconds = 3.0;
        state.total_duration_seconds = 3.0;
        assert_eq!(state.progress(), 0.0);

        state.remaining_seconds = 1.5;
        assert!((state.progress() - 0.5).abs() < 0.001);

        state.remaining_seconds = 0.0;
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn test_breath_cooldown_default() {
        let cooldown = BreathPulseCooldown::default();
        assert_eq!(cooldown.remaining_seconds, 0.0);
    }

    #[test]
    fn test_gentle_fade_default() {
        let fade = GentleFade::default();
        assert_eq!(fade.duration_seconds, GENTLE_FADE_DURATION_SECONDS);
    }
}
