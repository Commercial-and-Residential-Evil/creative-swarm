//! Module: particle
//! Purpose: Particle lifecycle, pooling, motion simulation, and behavior systems
//! Dependencies: components, resources, types

use bevy::prelude::*;

use crate::components::{
    Particle, ParticleBehavior, ParticleBundle, ParticleMotion, ParticleState, ParticleVisual,
    PulseResponder, Spawnable,
};
use crate::resources::{
    ActState, ColorPalette, CurrentInteractionMode, InterpolatedActValues,
    MouseState, ParticlePool, ParticleSpawnQueue, ParticleSpawnRequest, PeaTexture,
};
use crate::types::{Act, BeatStrength, InteractionMode, ParticleBehaviorType, SpawnSource};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Pool capacity - total pre-allocated particle entities.
const POOL_CAPACITY: u32 = 15000;

/// Maximum active particles at any time.
const MAX_ACTIVE: u32 = 10000;

/// Base size for pea particles (in pixels, before any scaling).
/// Sized for visibility on mobile devices.
const PEA_BASE_SIZE: f32 = 80.0;

/// Base particle lifetime in milliseconds.
const BASE_LIFETIME_MS: f32 = 5000.0;

/// Minimum spawn rate (particles per second) from mouse movement.
const MOUSE_SPAWN_RATE_MIN: f32 = 8.0;

/// Maximum spawn rate (particles per second) from mouse movement.
const MOUSE_SPAWN_RATE_MAX: f32 = 15.0;

/// Spawn rate multiplier when holding mouse button/finger while moving.
const HELD_SPAWN_RATE_MULTIPLIER: f32 = 8.0;

/// Maximum spawn rate when holding (particles per second).
const HELD_SPAWN_RATE_MAX: f32 = 120.0;

/// Spawn count range for soft beats.
const SOFT_BEAT_SPAWN_RANGE: (u32, u32) = (5, 10);

/// Spawn count range for medium beats.
const MEDIUM_BEAT_SPAWN_RANGE: (u32, u32) = (10, 20);

/// Spawn count range for strong beats.
const STRONG_BEAT_SPAWN_RANGE: (u32, u32) = (20, 40);

/// Base turbulence strength.
const BASE_TURBULENCE_STRENGTH: f32 = 15.0;

/// Turbulence time scale for noise evolution.
const TURBULENCE_TIME_SCALE: f32 = 0.5;

// =============================================================================
// EVENTS
// =============================================================================

/// Event fired when a beat is detected for particle spawning.
#[derive(Event, Debug, Clone)]
pub struct BeatDetected {
    /// The strength of the detected beat.
    pub strength: BeatStrength,
}

// =============================================================================
// STARTUP SYSTEMS
// =============================================================================

/// Loads the pea texture from assets and stores it as a resource.
pub fn load_pea_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("pea.png");
    commands.insert_resource(PeaTexture {
        handle: texture_handle,
    });
}

/// Pre-allocates particle entities for object pooling.
///
/// Creates 15000 particle entities in an inactive, hidden state and adds them
/// to the `ParticlePool.available_entities` for efficient recycling during gameplay.
/// This avoids runtime allocations and despawns, ensuring smooth performance.
pub fn setup_particle_pool(
    mut commands: Commands,
    mut pool: ResMut<ParticlePool>,
    pea_texture: Res<PeaTexture>,
) {
    pool.available_entities.clear();
    pool.active_count = 0;
    pool.pool_capacity = POOL_CAPACITY;
    pool.max_active = MAX_ACTIVE;

    // Pre-allocate entity IDs for the pool
    let mut entities = Vec::with_capacity(POOL_CAPACITY as usize);

    for id in 0..POOL_CAPACITY {
        let entity = commands
            .spawn(ParticleBundle::new(id))
            .insert(Sprite {
                image: pea_texture.handle.clone(),
                custom_size: Some(Vec2::splat(PEA_BASE_SIZE)),
                ..default()
            })
            .id();
        entities.push(entity);
    }

    pool.available_entities = entities;
}

// =============================================================================
// SPAWN SYSTEMS
// =============================================================================

/// Activates pooled particles from the spawn queue.
///
/// This is a CRITICAL PATH system that processes the `ParticleSpawnQueue` and
/// activates available particles from the pool. It respects `ParticlePool.max_active`
/// to prevent performance degradation from too many active particles.
pub fn spawn_particles_from_queue(
    mut pool: ResMut<ParticlePool>,
    mut spawn_queue: ResMut<ParticleSpawnQueue>,
    mut query: Query<
        (
            &mut ParticleState,
            &mut Transform,
            &mut ParticleVisual,
            &mut ParticleMotion,
            &mut ParticleBehavior,
            &mut Spawnable,
            &mut Visibility,
        ),
        With<Particle>,
    >,
    interpolated: Res<InterpolatedActValues>,
) {
    // Process pending spawn requests
    let pending = std::mem::take(&mut spawn_queue.pending_spawns);

    for request in pending {
        // Check if we can spawn more particles
        if pool.active_count >= pool.max_active {
            break;
        }

        // Get an available entity from the pool
        let Some(entity) = pool.available_entities.pop() else {
            break;
        };

        // Activate the particle
        if let Ok((
            mut state,
            mut transform,
            mut visual,
            mut motion,
            mut behavior,
            mut spawnable,
            mut visibility,
        )) = query.get_mut(entity)
        {
            // Set particle state to active
            state.active = true;
            state.lifetime_remaining_ms = request.lifetime_ms;
            state.lifetime_total_ms = request.lifetime_ms;

            // Set position
            transform.translation = request.position.extend(0.0);
            transform.scale = Vec3::splat(visual.scale);

            // Set visual properties
            visual.base_color = request.color;
            visual.current_color = request.color;
            visual.opacity = 1.0;

            // Set motion properties
            motion.velocity = request.initial_velocity;
            motion.acceleration = Vec2::ZERO;
            motion.drag = interpolated.particle_behavior.base_drag();
            motion.turbulence_seed = fastrand::f32() * 1000.0;

            // Set behavior based on current act
            behavior.behavior_type = interpolated.particle_behavior;
            behavior.behavior_strength = 1.0;
            behavior.target_position = None;

            // Set spawn source
            spawnable.spawn_source = request.source;

            // Make visible
            *visibility = Visibility::Visible;

            pool.active_count += 1;
        } else {
            // Entity query failed, return it to pool
            pool.available_entities.push(entity);
        }
    }
}

/// Spawns particles from mouse/touch interaction.
///
/// When in Paint mode, spawns particles at the pointer position.
/// When holding mouse button or finger while moving, dramatically increases
/// spawn rate for a "spray paint" effect.
///
/// Spawn rate varies from 8-15 particles/sec normally, up to 120 particles/sec
/// when holding and moving quickly.
pub fn spawn_particles_from_mouse(
    mouse: Res<MouseState>,
    mode: Res<CurrentInteractionMode>,
    mut spawn_queue: ResMut<ParticleSpawnQueue>,
    interpolated: Res<InterpolatedActValues>,
    palette: Res<ColorPalette>,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    touch_state: Res<crate::interaction::TouchState>,
) {
    // Spawn particles when touching/clicking in any mode (fidget app behavior)
    let _ = mode; // Mode no longer restricts spawning
    if !mouse.is_active {
        return;
    }

    // Check if holding mouse button or touch
    let is_holding = mouse_button.pressed(MouseButton::Left)
        || touch_state.primary_touch_id.is_some();

    // Calculate spawn rate based on mouse velocity
    let mouse_speed = mouse.velocity.length();
    let speed_factor = (mouse_speed / 500.0).clamp(0.0, 1.0);

    // Base spawn rate
    let base_spawn_rate =
        MOUSE_SPAWN_RATE_MIN + (MOUSE_SPAWN_RATE_MAX - MOUSE_SPAWN_RATE_MIN) * speed_factor;

    // Apply multiplier when holding and moving
    let spawn_rate = if is_holding && mouse_speed > 10.0 {
        // Holding while moving: dramatically increase spawn rate
        // Rate scales with movement speed
        let held_rate = base_spawn_rate * HELD_SPAWN_RATE_MULTIPLIER * (0.5 + speed_factor);
        held_rate.min(HELD_SPAWN_RATE_MAX)
    } else {
        base_spawn_rate
    };

    // Accumulate time for spawn timing
    spawn_queue.spawn_accumulator += time.delta_secs();

    let spawn_interval = 1.0 / spawn_rate;
    while spawn_queue.spawn_accumulator >= spawn_interval {
        spawn_queue.spawn_accumulator -= spawn_interval;

        // Calculate initial velocity based on mouse velocity with some randomization
        let base_velocity = mouse.velocity * 0.3;
        let random_offset = Vec2::new(
            (fastrand::f32() - 0.5) * 50.0,
            (fastrand::f32() - 0.5) * 50.0,
        );
        let initial_velocity = base_velocity + random_offset;

        // Select color from palette with some variation
        let color = select_spawn_color(&palette, &interpolated, SpawnSource::Mouse);

        // Calculate lifetime with source multiplier
        let lifetime = BASE_LIFETIME_MS * SpawnSource::Mouse.lifetime_multiplier();

        spawn_queue.pending_spawns.push(ParticleSpawnRequest {
            position: mouse.position,
            initial_velocity,
            color,
            lifetime_ms: lifetime,
            source: SpawnSource::Mouse,
        });
    }
}

/// Condition function for run_if: returns true when interaction mode is Paint.
pub fn interaction_mode_is_paint(mode: Res<CurrentInteractionMode>) -> bool {
    mode.mode == InteractionMode::Paint
}

/// Spawns particles in response to detected beats.
///
/// Different beat strengths trigger different spawn patterns:
/// - Soft: 5-10 particles in a gentle scatter
/// - Medium: 10-20 particles in a ripple pattern
/// - Strong: 20-40 particles in a radial burst
pub fn spawn_particles_from_beat(
    mut events: EventReader<BeatDetected>,
    mut spawn_queue: ResMut<ParticleSpawnQueue>,
    _act_state: Res<ActState>,
    interpolated: Res<InterpolatedActValues>,
    palette: Res<ColorPalette>,
    mouse: Res<MouseState>,
) {
    for event in events.read() {
        let (min_count, max_count, pattern) = match event.strength {
            BeatStrength::Silence => continue,
            BeatStrength::Soft => (
                SOFT_BEAT_SPAWN_RANGE.0,
                SOFT_BEAT_SPAWN_RANGE.1,
                SpawnPattern::Scatter,
            ),
            BeatStrength::Medium => (
                MEDIUM_BEAT_SPAWN_RANGE.0,
                MEDIUM_BEAT_SPAWN_RANGE.1,
                SpawnPattern::Ripple,
            ),
            BeatStrength::Strong => (
                STRONG_BEAT_SPAWN_RANGE.0,
                STRONG_BEAT_SPAWN_RANGE.1,
                SpawnPattern::Burst,
            ),
        };

        let count = fastrand::u32(min_count..=max_count);

        // Use mouse position as spawn center if active, otherwise use screen center
        let center = if mouse.is_active {
            mouse.position
        } else {
            Vec2::ZERO
        };

        // Spawn particles according to pattern
        for i in 0..count {
            let (position, velocity) = match pattern {
                SpawnPattern::Scatter => {
                    let offset = Vec2::new(
                        (fastrand::f32() - 0.5) * 200.0,
                        (fastrand::f32() - 0.5) * 200.0,
                    );
                    let vel = Vec2::new(
                        (fastrand::f32() - 0.5) * 100.0,
                        (fastrand::f32() - 0.5) * 100.0,
                    );
                    (center + offset, vel)
                }
                SpawnPattern::Ripple => {
                    let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                    let radius = 50.0 + fastrand::f32() * 50.0;
                    let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                    let vel = offset.normalize_or_zero() * (30.0 + fastrand::f32() * 50.0);
                    (center + offset, vel)
                }
                SpawnPattern::Burst => {
                    let angle = (i as f32 / count as f32) * std::f32::consts::TAU
                        + (fastrand::f32() - 0.5) * 0.3;
                    let speed = 100.0 + fastrand::f32() * 150.0;
                    let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
                    let offset = vel.normalize_or_zero() * (10.0 + fastrand::f32() * 30.0);
                    (center + offset, vel)
                }
            };

            let color = select_spawn_color(&palette, &interpolated, SpawnSource::Beat);
            let lifetime = BASE_LIFETIME_MS
                * SpawnSource::Beat.lifetime_multiplier()
                * (0.8 + fastrand::f32() * 0.4);

            spawn_queue.pending_spawns.push(ParticleSpawnRequest {
                position,
                initial_velocity: velocity,
                color,
                lifetime_ms: lifetime,
                source: SpawnSource::Beat,
            });
        }
    }
}

/// Spawn patterns for beat-triggered particle emission.
#[derive(Clone, Copy, Debug)]
enum SpawnPattern {
    /// Random scatter around center point.
    Scatter,
    /// Concentric ripple outward.
    Ripple,
    /// Radial burst emission.
    Burst,
}

/// Selects a spawn color from the palette based on source and act state.
fn select_spawn_color(
    palette: &ColorPalette,
    interpolated: &InterpolatedActValues,
    source: SpawnSource,
) -> Color {
    let base_color = match source {
        SpawnSource::Mouse => {
            // Mouse spawns use accent colors
            let r = fastrand::f32();
            if r < 0.4 {
                palette.accent_spark
            } else if r < 0.7 {
                palette.accent_deep
            } else {
                palette.accent_hope
            }
        }
        SpawnSource::Beat => {
            // Beat spawns use primary colors with saturation
            let r = fastrand::f32();
            if r < 0.5 {
                palette.primary_midpoint
            } else if r < 0.8 {
                palette.accent_spark
            } else {
                palette.accent_deep
            }
        }
        SpawnSource::Automatic => {
            // Automatic spawns use secondary colors
            let r = fastrand::f32();
            if r < 0.4 {
                palette.secondary_cool
            } else if r < 0.7 {
                palette.secondary_warm
            } else {
                palette.secondary_ethereal
            }
        }
    };

    // Apply saturation multiplier from act state
    apply_saturation_multiplier(base_color, interpolated.saturation_multiplier)
}

/// Applies a saturation multiplier to a color.
fn apply_saturation_multiplier(color: Color, multiplier: f32) -> Color {
    let srgba = color.to_srgba();
    // Simple saturation adjustment: blend toward grayscale
    let gray = srgba.red * 0.299 + srgba.green * 0.587 + srgba.blue * 0.114;
    let adjusted_multiplier = multiplier.clamp(0.0, 2.0);

    if adjusted_multiplier >= 1.0 {
        // Increase saturation
        let factor = adjusted_multiplier - 1.0;
        Color::srgba(
            srgba.red + (srgba.red - gray) * factor,
            srgba.green + (srgba.green - gray) * factor,
            srgba.blue + (srgba.blue - gray) * factor,
            srgba.alpha,
        )
    } else {
        // Decrease saturation
        Color::srgba(
            gray + (srgba.red - gray) * adjusted_multiplier,
            gray + (srgba.green - gray) * adjusted_multiplier,
            gray + (srgba.blue - gray) * adjusted_multiplier,
            srgba.alpha,
        )
    }
}

// =============================================================================
// LIFETIME SYSTEMS
// =============================================================================

/// Updates particle lifetime for all active particles.
///
/// This is a CRITICAL PATH system that decrements `lifetime_remaining_ms` by
/// delta time for all active particles. Runs on up to 10k particles per frame.
pub fn update_particle_lifetime(
    mut query: Query<&mut ParticleState, With<Particle>>,
    time: Res<Time>,
) {
    let delta_ms = time.delta_secs() * 1000.0;

    for mut state in query.iter_mut() {
        if state.active {
            state.lifetime_remaining_ms -= delta_ms;
        }
    }
}

/// Returns expired particles to the pool.
///
/// Particles with `lifetime_remaining_ms <= 0` are deactivated, hidden, and
/// returned to `ParticlePool.available_entities` for reuse.
pub fn despawn_expired_particles(
    mut pool: ResMut<ParticlePool>,
    mut query: Query<(Entity, &mut ParticleState, &mut Visibility), With<Particle>>,
) {
    for (entity, mut state, mut visibility) in query.iter_mut() {
        if state.active && state.lifetime_remaining_ms <= 0.0 {
            // Deactivate particle
            state.active = false;
            state.lifetime_remaining_ms = 0.0;

            // Hide particle
            *visibility = Visibility::Hidden;

            // Return to pool
            pool.available_entities.push(entity);
            pool.active_count = pool.active_count.saturating_sub(1);
        }
    }
}

// =============================================================================
// MOTION SYSTEMS
// =============================================================================

/// Applies act-specific behavior to particle motion.
///
/// This is a CRITICAL PATH system implementing different behaviors per act:
/// - Drift: Random walk with low speed (Act I)
/// - Swarm: Move toward center of nearby particles (Act II)
/// - Orbit: Circular motion around center (Act III)
/// - Disperse: Move upward and outward (Act IV)
/// - Float: Very slow drift with minimal forces (Act V)
pub fn apply_particle_behavior(
    mut query: Query<
        (
            &ParticleBehavior,
            &ParticleState,
            &Transform,
            &mut ParticleMotion,
        ),
        With<Particle>,
    >,
    time: Res<Time>,
) {
    let _dt = time.delta_secs();

    for (behavior, state, transform, mut motion) in query.iter_mut() {
        if !state.active {
            continue;
        }

        let pos = transform.translation.truncate();
        let strength = behavior.behavior_strength;

        match behavior.behavior_type {
            ParticleBehaviorType::Drift => {
                // Random walk: add small random acceleration
                let random_accel = Vec2::new(
                    (fastrand::f32() - 0.5) * 30.0,
                    (fastrand::f32() - 0.5) * 30.0,
                ) * strength;
                motion.acceleration = random_accel;
            }
            ParticleBehaviorType::Swarm => {
                // Swarm toward target or screen center
                let target = behavior.target_position.unwrap_or(Vec2::ZERO);
                let to_target = target - pos;
                let distance = to_target.length();

                if distance > 1.0 {
                    // Attraction force diminishes with distance
                    let attraction_strength = (300.0 / (distance + 100.0)) * 50.0;
                    motion.acceleration = to_target.normalize() * attraction_strength * strength;
                }
            }
            ParticleBehaviorType::Orbit => {
                // Circular motion around target or center
                let target = behavior.target_position.unwrap_or(Vec2::ZERO);
                let to_center = target - pos;
                let distance = to_center.length();

                if distance > 10.0 {
                    // Centripetal acceleration toward center
                    let centripetal = to_center.normalize() * 40.0;
                    // Tangential component for orbital motion
                    let tangent = Vec2::new(-to_center.y, to_center.x).normalize() * 60.0;

                    motion.acceleration = (centripetal + tangent) * strength;
                }
            }
            ParticleBehaviorType::Disperse => {
                // Rise upward and spread outward
                let upward = Vec2::new(0.0, 50.0);
                let outward = if pos.length() > 1.0 {
                    pos.normalize() * 20.0
                } else {
                    Vec2::ZERO
                };
                motion.acceleration = (upward + outward) * strength;
            }
            ParticleBehaviorType::Float => {
                // Very gentle drift with slight upward tendency
                let gentle_drift = Vec2::new(
                    (fastrand::f32() - 0.5) * 10.0,
                    5.0 + (fastrand::f32() - 0.5) * 5.0,
                );
                motion.acceleration = gentle_drift * strength;
            }
        }
    }
}

/// Applies turbulence using noise for organic particle movement.
///
/// Uses a simplified noise function based on the particle's turbulence_seed
/// and current time. Turbulence strength varies by act:
/// - Higher in Acts III (Crescendo) and IV (Release)
/// - Lower in Act V (Transcendence)
pub fn apply_turbulence(
    mut query: Query<(&mut ParticleMotion, &ParticleState, &Transform), With<Particle>>,
    act_state: Res<ActState>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();

    // Turbulence strength varies by act
    let act_turbulence_multiplier = match act_state.current_act {
        Act::Emergence => 0.5,
        Act::Accumulation => 0.6,
        Act::Crescendo => 1.0,
        Act::Release => 0.8,
        Act::Transcendence => 0.3,
    };

    let turbulence_strength = BASE_TURBULENCE_STRENGTH * act_turbulence_multiplier;

    for (mut motion, state, transform) in query.iter_mut() {
        if !state.active {
            continue;
        }

        let pos = transform.translation.truncate();

        // Simple noise approximation using sine waves with different frequencies
        let t = elapsed * TURBULENCE_TIME_SCALE + motion.turbulence_seed;
        let noise_x = (t * 1.3 + pos.x * 0.01).sin() * 0.5
            + (t * 2.7 + pos.y * 0.02).sin() * 0.3
            + (t * 0.7 + pos.x * 0.03 + pos.y * 0.02).sin() * 0.2;
        let noise_y = (t * 1.7 + pos.y * 0.01).sin() * 0.5
            + (t * 2.3 + pos.x * 0.02).sin() * 0.3
            + (t * 0.9 + pos.y * 0.03 + pos.x * 0.02).sin() * 0.2;

        let turbulence = Vec2::new(noise_x, noise_y) * turbulence_strength;
        motion.velocity += turbulence * time.delta_secs();
    }
}

/// Integrates particle motion: applies velocity and acceleration to position.
///
/// This is a CRITICAL PATH system that runs on all active particles:
/// - Updates position: position += velocity * dt
/// - Updates velocity: velocity += acceleration * dt
/// - Applies drag: velocity *= drag
pub fn integrate_particle_motion(
    mut query: Query<(&ParticleMotion, &ParticleState, &mut Transform), With<Particle>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (motion, state, mut transform) in query.iter_mut() {
        if !state.active {
            continue;
        }

        // Apply velocity to position
        transform.translation.x += motion.velocity.x * dt;
        transform.translation.y += motion.velocity.y * dt;
    }
}

/// Post-integration system to apply acceleration and drag to velocity.
///
/// Separated from integrate_particle_motion for clearer system ordering.
pub fn apply_velocity_changes(
    mut query: Query<(&mut ParticleMotion, &ParticleState), With<Particle>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut motion, state) in query.iter_mut() {
        if !state.active {
            continue;
        }

        // Apply acceleration (copy to avoid borrow issues)
        let accel = motion.acceleration;
        motion.velocity += accel * dt;

        // Apply drag (exponential decay) - copy drag to avoid borrow issues
        let drag = motion.drag;
        motion.velocity *= drag.powf(dt * 60.0);

        // Clamp velocity to prevent runaway speeds
        let max_speed = 500.0;
        if motion.velocity.length() > max_speed {
            motion.velocity = motion.velocity.normalize() * max_speed;
        }

        // Reset acceleration for next frame
        motion.acceleration = Vec2::ZERO;
    }
}

// =============================================================================
// VISUAL SYSTEMS
// =============================================================================

/// Syncs particle visual state to sprite components for rendering.
///
/// Copies ParticleVisual properties (color, opacity, scale) to the Sprite
/// component so the rendering system displays the correct appearance.
/// Applies pulse scale modifier through custom_size to avoid blurry transform scaling.
pub fn sync_sprite_visuals(
    mut query: Query<
        (&ParticleVisual, &ParticleState, &PulseResponder, &mut Sprite, &mut Transform),
        With<Particle>,
    >,
) {
    for (visual, state, pulse_responder, mut sprite, mut transform) in query.iter_mut() {
        if !state.active {
            continue;
        }

        // Calculate opacity based on lifetime remaining
        let lifetime_factor = if state.lifetime_total_ms > 0.0 {
            (state.lifetime_remaining_ms / state.lifetime_total_ms).clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Fade out in the last 20% of lifetime
        let fade_factor = if lifetime_factor < 0.2 {
            lifetime_factor / 0.2
        } else {
            1.0
        };

        let final_opacity = visual.opacity * fade_factor;

        // Apply color with opacity
        let color = visual.current_color.to_srgba();
        sprite.color = Color::srgba(color.red, color.green, color.blue, final_opacity);

        // Apply scale to the pea sprite via custom_size (including pulse modifier)
        // All scaling through custom_size keeps rendering crisp (no transform.scale blurring)
        let scaled_size = PEA_BASE_SIZE * visual.scale * pulse_responder.current_scale_modifier;
        sprite.custom_size = Some(Vec2::splat(scaled_size));

        // Keep transform scale at 1.0 to avoid blurry texture filtering
        transform.scale = Vec3::ONE;
    }
}

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin bundling all particle-related systems.
///
/// Registers the following systems:
/// - Startup: setup_particle_pool
/// - Update: spawn_particles_from_queue, spawn_particles_from_mouse,
///           spawn_particles_from_beat, update_particle_lifetime,
///           despawn_expired_particles, apply_particle_behavior,
///           apply_turbulence, integrate_particle_motion, sync_sprite_visuals
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BeatDetected>()
            // Startup systems: load texture first, then setup pool
            .add_systems(Startup, (load_pea_texture, setup_particle_pool).chain())
            // Update systems with proper ordering
            .add_systems(
                Update,
                (
                    // Spawn systems - run before motion
                    spawn_particles_from_mouse, // Works in all acts for fidget app behavior
                    spawn_particles_from_beat,
                    spawn_particles_from_queue,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    // Lifecycle systems
                    update_particle_lifetime,
                    despawn_expired_particles,
                )
                    .chain()
                    .after(spawn_particles_from_queue),
            )
            .add_systems(
                Update,
                (
                    // Motion systems - CRITICAL PATH
                    apply_particle_behavior,
                    apply_turbulence,
                    integrate_particle_motion,
                    apply_velocity_changes,
                )
                    .chain()
                    .after(spawn_particles_from_queue),
            )
            .add_systems(PostUpdate, sync_sprite_visuals);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_pattern_constants() {
        assert!(SOFT_BEAT_SPAWN_RANGE.0 < SOFT_BEAT_SPAWN_RANGE.1);
        assert!(MEDIUM_BEAT_SPAWN_RANGE.0 < MEDIUM_BEAT_SPAWN_RANGE.1);
        assert!(STRONG_BEAT_SPAWN_RANGE.0 < STRONG_BEAT_SPAWN_RANGE.1);

        // Medium should spawn more than soft
        assert!(MEDIUM_BEAT_SPAWN_RANGE.0 >= SOFT_BEAT_SPAWN_RANGE.0);
        // Strong should spawn more than medium
        assert!(STRONG_BEAT_SPAWN_RANGE.0 >= MEDIUM_BEAT_SPAWN_RANGE.0);
    }

    #[test]
    fn test_pool_constants() {
        assert!(MAX_ACTIVE <= POOL_CAPACITY);
        assert!(POOL_CAPACITY > 0);
        assert!(MAX_ACTIVE > 0);
    }

    #[test]
    fn test_mouse_spawn_rate_range() {
        assert!(MOUSE_SPAWN_RATE_MIN > 0.0);
        assert!(MOUSE_SPAWN_RATE_MAX > MOUSE_SPAWN_RATE_MIN);
    }

    #[test]
    fn test_saturation_multiplier() {
        let white = Color::WHITE;

        // Multiplier of 1.0 should not change the color significantly
        let result = apply_saturation_multiplier(white, 1.0);
        let result_srgba = result.to_srgba();
        assert!((result_srgba.red - 1.0).abs() < 0.01);

        // Multiplier of 0.0 should result in grayscale
        let colored = Color::srgb(1.0, 0.0, 0.0);
        let gray_result = apply_saturation_multiplier(colored, 0.0);
        let gray_srgba = gray_result.to_srgba();
        // Red component should equal gray value
        let expected_gray = 0.299; // Luminance of pure red
        assert!((gray_srgba.red - expected_gray).abs() < 0.01);
    }

    #[test]
    fn test_beat_strength_spawning() {
        // Silence should not spawn
        assert!(!BeatStrength::Silence.should_spawn());

        // All other strengths should spawn
        assert!(BeatStrength::Soft.should_spawn());
        assert!(BeatStrength::Medium.should_spawn());
        assert!(BeatStrength::Strong.should_spawn());
    }
}
