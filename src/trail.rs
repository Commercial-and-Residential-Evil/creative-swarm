//! Module: trail
//! Purpose: Trail rendering system with exponential opacity decay for particle visualization
//! Dependencies: components, resources, particle

use bevy::prelude::*;

use crate::components::{Particle, ParticleState, ParticleVisual, Trail, TrailRenderer, TrailSegment};
use crate::resources::MotionTiming;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Number of segments in each trail's circular buffer.
pub const TRAIL_SEGMENTS: usize = 12;

/// Duration for trail to fully fade out in milliseconds.
pub const TRAIL_FADE_DURATION_MS: f32 = 1500.0;

/// Base width of trails at the head (newest segment).
pub const TRAIL_BASE_WIDTH: f32 = 4.0;

/// Taper factor for trail width from head to tail.
/// Width at segment n = base_width * taper_factor^n
pub const TRAIL_TAPER_FACTOR: f32 = 0.7;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Calculates the width of a trail segment based on its position in the trail.
///
/// The trail tapers from the head (newest segment) to the tail (oldest segment).
/// Width decreases geometrically: width(n) = base_width * taper_factor^n
///
/// # Arguments
/// * `segment_index` - Zero-based index from head (0 = newest, 11 = oldest)
/// * `base_width` - Width at the head of the trail
/// * `taper_factor` - Geometric decay factor (0.0 to 1.0)
///
/// # Returns
/// The calculated width for this segment.
#[inline]
pub fn calculate_trail_width(segment_index: usize, base_width: f32, taper_factor: f32) -> f32 {
    base_width * taper_factor.powi(segment_index as i32)
}

/// Calculates the decay rate for exponential opacity decay.
///
/// Given a target fade duration, computes the decay rate constant `k` such that
/// opacity will reach approximately 1% of original after `duration_ms` milliseconds.
///
/// The decay formula is: opacity *= exp(-k * dt)
/// For opacity to reach 0.01 after duration: k = -ln(0.01) / duration = 4.605 / duration
///
/// # Arguments
/// * `duration_ms` - Target duration for full fade in milliseconds
///
/// # Returns
/// The decay rate constant (per millisecond).
#[inline]
fn calculate_decay_rate(duration_ms: f32) -> f32 {
    if duration_ms > 0.0 {
        // -ln(0.01) approximately equals 4.605
        4.605 / duration_ms
    } else {
        1.0 // Instant decay if duration is zero
    }
}

// =============================================================================
// UPDATE SYSTEMS
// =============================================================================

/// Records current particle positions to trail circular buffers.
///
/// This system runs after `integrate_particle_motion` to capture the new positions.
/// For each active particle with a Trail component:
/// 1. Gets the current world position from Transform
/// 2. Creates a new TrailSegment with the position and full opacity
/// 3. Pushes the segment to the Trail circular buffer (advancing head_index)
/// 4. Sets the timestamp for age tracking
///
/// # System Ordering
/// - Stage: Update
/// - After: integrate_particle_motion
/// - Before: decay_trail_opacity
pub fn update_trails(
    mut query: Query<(&Transform, &ParticleState, &TrailRenderer, &mut Trail), With<Particle>>,
    time: Res<Time>,
) {
    let current_time_ms = time.elapsed_secs() * 1000.0;

    for (transform, state, renderer, mut trail) in query.iter_mut() {
        // Skip inactive particles
        if !state.active {
            continue;
        }

        // Skip if trail rendering is disabled for this particle
        if !renderer.enabled {
            continue;
        }

        // Extract 2D position from transform
        let position = transform.translation.truncate();

        // Calculate width for the head segment (index 0 = newest)
        let width = calculate_trail_width(0, renderer.base_width, renderer.taper_factor);

        // Create new trail segment at current position
        let segment = TrailSegment {
            position,
            opacity: 1.0,
            width,
            timestamp_ms: current_time_ms,
        };

        // Push to circular buffer (this advances head_index)
        trail.push_segment(segment);
    }
}

/// Applies exponential opacity decay to all trail segments.
///
/// Uses the formula: opacity *= exp(-decay_rate * dt)
/// where decay_rate is calculated to fade trails over TRAIL_FADE_DURATION_MS.
///
/// The decay rate can optionally be derived from MotionTiming for synchronization
/// with the overall experience timing, but defaults to TRAIL_FADE_DURATION_MS.
///
/// # System Ordering
/// - Stage: Update
/// - After: update_trails
/// - Before: render_trails
pub fn decay_trail_opacity(
    mut query: Query<&mut Trail, With<Particle>>,
    time: Res<Time>,
    motion_timing: Option<Res<MotionTiming>>,
) {
    let dt_ms = time.delta_secs() * 1000.0;

    // Calculate decay rate based on fade duration
    // If MotionTiming is available, we could sync to its timing, but we use
    // the constant TRAIL_FADE_DURATION_MS for consistent trail behavior
    let fade_duration = if let Some(ref timing) = motion_timing {
        // Use slow_transition_duration_ms as a reference but cap at our max
        timing.slow_transition_duration_ms.min(TRAIL_FADE_DURATION_MS)
    } else {
        TRAIL_FADE_DURATION_MS
    };

    let decay_rate = calculate_decay_rate(fade_duration);
    let decay_factor = (-decay_rate * dt_ms).exp();

    for mut trail in query.iter_mut() {
        // Apply decay to all segments in the trail
        for segment in trail.segments.iter_mut() {
            // Apply exponential decay
            segment.opacity *= decay_factor;

            // Clamp to zero when effectively invisible (avoids denormals)
            if segment.opacity < 0.001 {
                segment.opacity = 0.0;
            }
        }
    }
}

// =============================================================================
// POST-UPDATE SYSTEMS
// =============================================================================

/// Placeholder system for trail rendering.
///
/// Actual mesh generation for trails is complex and requires:
/// - Building triangle strip geometry from trail segments
/// - Applying per-vertex colors with opacity
/// - Managing GPU buffers for efficient updates
///
/// This placeholder system:
/// - Respects TrailRenderer.enabled status
/// - Can log trail data for debugging
/// - Sets up the framework for future mesh generation
///
/// # Implementation Notes
/// A full implementation would:
/// 1. Create a Mesh with vertex positions forming a ribbon
/// 2. Apply UV coordinates for texture mapping
/// 3. Set vertex colors including opacity from segments
/// 4. Update mesh buffers each frame (or use GPU instancing)
///
/// # System Ordering
/// - Stage: PostUpdate
/// - After: decay_trail_opacity
pub fn render_trails(
    query: Query<(Entity, &Trail, &TrailRenderer, &ParticleVisual, &ParticleState), With<Particle>>,
) {
    // Count active trails for potential debugging/metrics
    let mut _active_trail_count = 0;
    let mut _total_visible_segments = 0;

    for (_entity, trail, renderer, _visual, state) in query.iter() {
        // Skip inactive particles
        if !state.active {
            continue;
        }

        // Skip if trail rendering is disabled
        if !renderer.enabled {
            continue;
        }

        _active_trail_count += 1;

        // Count visible segments (opacity > threshold)
        for segment in trail.iter_segments() {
            if segment.opacity > 0.01 {
                _total_visible_segments += 1;
            }
        }

        // Note: Actual mesh generation would happen here
        // The implementation would involve:
        //
        // 1. Build vertex positions:
        //    - For each pair of adjacent segments, create a quad
        //    - Calculate perpendicular vectors for ribbon width
        //    - Apply segment width (with taper factor)
        //
        // 2. Build vertex colors:
        //    - Blend particle visual color with segment opacity
        //    - Apply gradient along trail length
        //
        // 3. Build indices for triangle strip
        //
        // 4. Update or create Mesh2d handle
        //
        // For now, trails are represented by the Trail component data
        // and can be visualized through debug gizmos or custom rendering
    }

    // Debug logging (uncomment for development)
    // if _active_trail_count > 0 {
    //     debug!(
    //         "Trails: {} active, {} visible segments",
    //         _active_trail_count, _total_visible_segments
    //     );
    // }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Resets a trail to its default state.
///
/// Clears all segments and resets head_index. Useful when recycling
/// particles from the pool.
///
/// # Arguments
/// * `trail` - Mutable reference to the trail to reset
pub fn reset_trail(trail: &mut Trail) {
    trail.head_index = 0;
    for segment in trail.segments.iter_mut() {
        segment.position = Vec2::ZERO;
        segment.opacity = 0.0;
        segment.width = 0.0;
        segment.timestamp_ms = 0.0;
    }
}

/// Gets the total visible length of a trail.
///
/// Calculates the sum of distances between consecutive visible segments.
/// Useful for metrics and debugging.
///
/// # Arguments
/// * `trail` - Reference to the trail to measure
///
/// # Returns
/// The total length of visible trail segments in world units.
pub fn get_trail_length(trail: &Trail) -> f32 {
    let segments: Vec<&TrailSegment> = trail
        .iter_segments()
        .filter(|s| s.opacity > 0.01)
        .collect();

    if segments.len() < 2 {
        return 0.0;
    }

    let mut total_length = 0.0;
    for window in segments.windows(2) {
        let distance = window[0].position.distance(window[1].position);
        total_length += distance;
    }

    total_length
}

/// Gets the age of the oldest visible segment in milliseconds.
///
/// # Arguments
/// * `trail` - Reference to the trail to inspect
/// * `current_time_ms` - Current time in milliseconds
///
/// # Returns
/// Age of the oldest visible segment, or 0.0 if no visible segments.
pub fn get_oldest_segment_age(trail: &Trail, current_time_ms: f32) -> f32 {
    trail
        .iter_segments()
        .filter(|s| s.opacity > 0.01 && s.timestamp_ms > 0.0)
        .map(|s| current_time_ms - s.timestamp_ms)
        .fold(0.0_f32, |max, age| max.max(age))
}

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin bundling trail rendering systems.
///
/// Registers the following systems:
/// - Update: update_trails (after integrate_particle_motion)
/// - Update: decay_trail_opacity (after update_trails)
/// - PostUpdate: render_trails
///
/// The TrailPlugin works in conjunction with the ParticlePlugin to provide
/// visual trails that follow particle movement with exponential opacity decay.
pub struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_trails,
                decay_trail_opacity,
            )
                .chain()
                // These systems should run after particle motion is integrated
                // The particle module's integrate_particle_motion runs in Update
                .after(crate::particle::integrate_particle_motion),
        )
        .add_systems(PostUpdate, render_trails);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trail_constants() {
        assert_eq!(TRAIL_SEGMENTS, 12);
        assert_eq!(TRAIL_FADE_DURATION_MS, 1500.0);
        assert_eq!(TRAIL_BASE_WIDTH, 4.0);
        assert_eq!(TRAIL_TAPER_FACTOR, 0.7);
    }

    #[test]
    fn test_calculate_trail_width() {
        // Head segment (index 0) should have full width
        let width_0 = calculate_trail_width(0, TRAIL_BASE_WIDTH, TRAIL_TAPER_FACTOR);
        assert_eq!(width_0, TRAIL_BASE_WIDTH);

        // Second segment should be width * taper
        let width_1 = calculate_trail_width(1, TRAIL_BASE_WIDTH, TRAIL_TAPER_FACTOR);
        assert!((width_1 - TRAIL_BASE_WIDTH * TRAIL_TAPER_FACTOR).abs() < 0.001);

        // Third segment should be width * taper^2
        let width_2 = calculate_trail_width(2, TRAIL_BASE_WIDTH, TRAIL_TAPER_FACTOR);
        assert!((width_2 - TRAIL_BASE_WIDTH * TRAIL_TAPER_FACTOR.powi(2)).abs() < 0.001);

        // Last segment should be much smaller
        let width_11 = calculate_trail_width(11, TRAIL_BASE_WIDTH, TRAIL_TAPER_FACTOR);
        assert!(width_11 < width_0);
        assert!(width_11 > 0.0);
    }

    #[test]
    fn test_calculate_trail_width_with_zero_taper() {
        // With taper factor 0, only head has width
        let width_0 = calculate_trail_width(0, 4.0, 0.0);
        assert_eq!(width_0, 4.0);

        let width_1 = calculate_trail_width(1, 4.0, 0.0);
        assert_eq!(width_1, 0.0);
    }

    #[test]
    fn test_calculate_trail_width_with_full_taper() {
        // With taper factor 1, all segments have same width
        let width_0 = calculate_trail_width(0, 4.0, 1.0);
        let width_5 = calculate_trail_width(5, 4.0, 1.0);
        let width_11 = calculate_trail_width(11, 4.0, 1.0);

        assert_eq!(width_0, 4.0);
        assert_eq!(width_5, 4.0);
        assert_eq!(width_11, 4.0);
    }

    #[test]
    fn test_decay_rate_calculation() {
        // Verify decay rate gives expected fade behavior
        let decay_rate = calculate_decay_rate(TRAIL_FADE_DURATION_MS);

        // After TRAIL_FADE_DURATION_MS, opacity should be ~1%
        let initial_opacity = 1.0;
        let final_opacity = initial_opacity * (-decay_rate * TRAIL_FADE_DURATION_MS).exp();

        assert!(final_opacity < 0.02, "Should be near 1%: {}", final_opacity);
        assert!(final_opacity > 0.005, "Should not be too small: {}", final_opacity);
    }

    #[test]
    fn test_decay_rate_zero_duration() {
        // Zero duration should give rate of 1.0 (instant decay)
        let decay_rate = calculate_decay_rate(0.0);
        assert_eq!(decay_rate, 1.0);
    }

    #[test]
    fn test_decay_rate_different_durations() {
        // Shorter duration should have higher decay rate
        let rate_short = calculate_decay_rate(500.0);
        let rate_long = calculate_decay_rate(2000.0);

        assert!(rate_short > rate_long);
    }

    #[test]
    fn test_reset_trail() {
        let mut trail = Trail::default();

        // Add some segments
        trail.push_segment(TrailSegment {
            position: Vec2::new(10.0, 20.0),
            opacity: 0.8,
            width: 4.0,
            timestamp_ms: 100.0,
        });
        trail.push_segment(TrailSegment {
            position: Vec2::new(30.0, 40.0),
            opacity: 0.6,
            width: 3.0,
            timestamp_ms: 200.0,
        });

        // Reset the trail
        reset_trail(&mut trail);

        // Verify all segments are cleared
        assert_eq!(trail.head_index, 0);
        for segment in trail.segments.iter() {
            assert_eq!(segment.position, Vec2::ZERO);
            assert_eq!(segment.opacity, 0.0);
            assert_eq!(segment.width, 0.0);
            assert_eq!(segment.timestamp_ms, 0.0);
        }
    }

    #[test]
    fn test_get_trail_length() {
        let mut trail = Trail::default();

        // Empty trail should have zero length
        assert_eq!(get_trail_length(&trail), 0.0);

        // Add visible segments in a line
        trail.push_segment(TrailSegment {
            position: Vec2::new(0.0, 0.0),
            opacity: 1.0,
            width: 4.0,
            timestamp_ms: 100.0,
        });
        trail.push_segment(TrailSegment {
            position: Vec2::new(3.0, 4.0), // Distance 5 from origin
            opacity: 1.0,
            width: 3.0,
            timestamp_ms: 200.0,
        });

        let length = get_trail_length(&trail);
        assert!((length - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_get_trail_length_ignores_invisible() {
        let mut trail = Trail::default();

        // Add a visible segment
        trail.push_segment(TrailSegment {
            position: Vec2::new(0.0, 0.0),
            opacity: 1.0,
            width: 4.0,
            timestamp_ms: 100.0,
        });

        // Add an invisible segment (should be ignored)
        trail.push_segment(TrailSegment {
            position: Vec2::new(100.0, 0.0),
            opacity: 0.001, // Below visibility threshold
            width: 3.0,
            timestamp_ms: 200.0,
        });

        // Only one visible segment, so length should be 0
        let length = get_trail_length(&trail);
        assert_eq!(length, 0.0);
    }

    #[test]
    fn test_get_oldest_segment_age() {
        let mut trail = Trail::default();

        // Empty trail should have zero age
        assert_eq!(get_oldest_segment_age(&trail, 1000.0), 0.0);

        // Add segments at different times
        trail.push_segment(TrailSegment {
            position: Vec2::ZERO,
            opacity: 1.0,
            width: 4.0,
            timestamp_ms: 100.0,
        });
        trail.push_segment(TrailSegment {
            position: Vec2::ZERO,
            opacity: 1.0,
            width: 3.0,
            timestamp_ms: 200.0,
        });

        // At time 1000ms, oldest segment (100ms) should be 900ms old
        let age = get_oldest_segment_age(&trail, 1000.0);
        assert!((age - 900.0).abs() < 0.001);
    }

    #[test]
    fn test_exponential_decay_behavior() {
        // Simulate decay over multiple frames
        let decay_rate = calculate_decay_rate(TRAIL_FADE_DURATION_MS);
        let mut opacity = 1.0;

        // Simulate 60 FPS for 1.5 seconds
        let dt_ms = 1000.0 / 60.0;
        let frames = (TRAIL_FADE_DURATION_MS / dt_ms) as u32;

        for _ in 0..frames {
            opacity *= (-decay_rate * dt_ms).exp();
        }

        // After fade duration, should be near 1%
        assert!(opacity < 0.02);
        assert!(opacity > 0.005);
    }

    #[test]
    fn test_trail_width_sequence() {
        // Verify width sequence is monotonically decreasing
        let mut prev_width = f32::MAX;

        for i in 0..TRAIL_SEGMENTS {
            let width = calculate_trail_width(i, TRAIL_BASE_WIDTH, TRAIL_TAPER_FACTOR);
            assert!(width <= prev_width, "Width should decrease: {} vs {}", width, prev_width);
            assert!(width > 0.0, "Width should be positive at index {}", i);
            prev_width = width;
        }
    }
}
