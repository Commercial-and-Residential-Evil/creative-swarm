# Whirled Peas Visualiser

A wordless poem in light and sound - an interactive particle visualization experience built with Bevy for Android.

## Overview

Whirled Peas Visualiser is a meditative fidget app where users create and interact with flowing particle "peas" through touch. The experience responds to user engagement: more interaction brings ambient audio to life, while neglect lets the soundscape fade away.

## Features

### Touch Interaction
- **Single tap**: Creates an explosion of particles at the touch point
- **Two-finger tap**: Triggers a hyperspace effect, scattering all particles
- **Hold and drag**: Continuously spawns particles with increasing rate
- **Works in all modes**: Touch interaction is always available throughout the experience

### Particle System
- **Object pooling**: 15,000 pre-allocated particles for smooth performance
- **Age-based pulsing**: Each particle breathes independently based on its lifetime
- **Organic motion**: Particles drift with turbulence and subtle physics
- **Crisp rendering**: Nearest-neighbor filtering keeps pixel art sharp at any scale

### Five-Act Narrative Structure
The visualizer progresses through five acts, each with distinct visual characteristics:

1. **Act I: Emergence** (0-3 min) - Sparse peas drift slowly in deep navy void
2. **Act II: Accumulation** (3-7 min) - Particles gather with increasing density
3. **Act III: Crescendo** (7-10 min) - Peak intensity with dramatic visual effects
4. **Act IV: Release** (10-13 min) - Particles disperse in bittersweet transition
5. **Act V: Transcendence** (13-15 min) - Weightless luminosity, peaceful dissolution

After completing, the experience cycles back to Act I with a hyperspace transition.

### Adaptive Audio
- **Engagement-responsive**: Ambient audio volume scales with particle count
- **Non-intrusive**: Audio only plays when interacting, preserving device audio focus
- **Smooth transitions**: Volume fades in/out gracefully as particles appear/disappear
- **Mix-friendly**: Designed to blend with podcasts or music at 70% max volume

### Visual Polish
- **Dynamic backgrounds**: Color gradients transition through acts
- **Act titles**: Current act displayed at top of screen
- **Post-processing**: Bloom, chromatic aberration, and vignette effects
- **Orientation support**: Works in both portrait and landscape

## Technical Architecture

### Core Modules
- `particle.rs` - Particle lifecycle, pooling, motion, and spawning
- `visual.rs` - Camera, colors, background rendering, UI
- `interaction.rs` - Touch/mouse input, gestures, hyperspace effect
- `act_management.rs` - Five-act progression and transitions
- `audio_reactive.rs` - Ambient audio with particle-based volume control
- `post_process.rs` - Visual effects pipeline

### Performance
- Object pooling eliminates runtime allocations
- ECS architecture enables parallel system execution
- GPU-accelerated sprite rendering
- Efficient touch state tracking with gesture recognition

## Building for Android

### Prerequisites
- Rust toolchain with Android targets
- Android NDK
- Android SDK with build tools

### Build Command
```bash
./build-android.sh
```

### Install
```bash
adb install android/app/build/outputs/apk/release/app-release-unsigned.apk
```

## Configuration

Key parameters can be adjusted in `src/resources.rs`:

```rust
// Ambient audio settings
max_volume: 0.7,           // Maximum blend volume
particle_threshold: 5,      // Particles before audio starts
particle_full_volume: 50,   // Particles for full volume

// Particle settings (in particle.rs)
PEA_BASE_SIZE: 80.0,       // Base particle size in pixels
POOL_CAPACITY: 15000,      // Pre-allocated particle count
MAX_ACTIVE: 10000,         // Maximum concurrent particles
```

## License

MIT License - See LICENSE file for details.
