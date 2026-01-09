# Rust Art Swarm

An interactive generative art creation system powered by Rust, Bevy, and Claude AI agents.

## What Is This?

Rust Art Swarm is a creative coding framework that combines:

- Bevy game engine for high-performance real-time graphics
- bevy_kira_audio for reactive audio integration
- Claude AI agent swarm for automated creative development
- WGSL shaders for GPU-accelerated visual effects

The system transforms creative prompts into fully-realized interactive audiovisual experiences through a structured pipeline of specialized AI agents.

## Demo of Agentic Art Swarm: Whirled Peas Visualiser

[Whirled Peas Visualiser](WHIRLED_PEAS.md) is the first creative work produced by the Rust Art Swarm system. It demonstrates the full pipeline from creative vision to deployed Android application.

Whirled Peas is a meditative fidget app featuring:
- Touch-responsive particle system with 15,000 pooled entities
- Five-act narrative structure with evolving visuals
- Engagement-adaptive ambient audio that respects device audio focus
- Crisp pixel art rendering with organic particle motion

See [WHIRLED_PEAS.md](WHIRLED_PEAS.md) for full documentation.

## Project Structure

    rust-art-swarm/
    ├── .claude/
    │   ├── agents/          # AI agent definitions
    │   └── commands/        # Flow orchestration commands
    ├── .runs/               # Run artifacts and outputs
    ├── src/                 # Rust source code
    ├── assets/
    │   ├── shaders/         # WGSL shader files
    │   └── audio/           # Audio assets
    ├── Cargo.toml           # Rust dependencies
    └── README.md

## Running Flows in Claude Desktop

The creative pipeline consists of six flows, each orchestrated through Claude Desktop.

### Flow 1: Signal

Start a new creative run by providing a creative prompt. The art-vision-author agent transforms your intent into structured art direction.

To run: Open Claude Desktop and describe your creative vision. Claude will invoke flow-1-signal to capture and structure your intent.

### Flow 2: Plan

Technical planning translates art vision into Bevy architecture, audio design, and interaction specifications.

Agents involved: rust-art-architect, audio-designer, interaction-designer

### Flow 3: Build

Code generation produces Rust modules and WGSL shaders based on the technical plans.

Agents involved: shader-sketcher, bevy-implementer

### Flow 4: Gate

Quality assurance reviews performance, safety, and licensing before deployment approval.

Agents involved: performance-critic, safety-license-critic, gatekeeper

### Flow 5: Deploy

Packaging creates release artifacts for distribution across platforms.

Agents involved: packager

### Flow 6: Wisdom

Retrospective analysis extracts learnings to improve future creative runs.

Agents involved: wisdom-retro

## Building and Running Locally

### Prerequisites

- Rust toolchain (rustup.rs)
- Cargo package manager (included with Rust)

### Build Commands

To check compilation without building:

    cargo check

To build in debug mode:

    cargo build

To build optimized release:

    cargo build --release

To run the application:

    cargo run

To run the optimized release:

    cargo run --release

### Platform Notes

The default configuration targets desktop platforms. For WebAssembly builds, enable the wasm feature and use wasm-pack or trunk for bundling.

## Limitless Creativity Model

The system achieves creative limitlessness through four extensible dimensions:

### Shaders

Visual effects are implemented as WGSL shaders in assets/shaders/. The shader-sketcher agent generates new shaders based on art direction, enabling:

- Particle system rendering with custom behaviors
- Post-processing effects (bloom, distortion, color grading)
- Procedural pattern generation (noise, fractals, voronoi)
- Material shaders for surface appearance
- Compute shaders for GPU-accelerated simulation

Add new visual capabilities by creating shader files and corresponding Rust bindings.

### Audio Voices

Sound design leverages bevy_kira_audio for reactive audio. The audio-designer agent specifies:

- Synthesized voices with parameter modulation
- Sample-based sounds with dynamic triggering
- Generative audio driven by visual state
- Frequency analysis for audio-reactive visuals
- Spatial audio for immersive experiences

Extend audio capabilities by adding new voice types and reactive mappings.

### Interaction Modes

User interaction is defined by the interaction-designer agent, supporting:

- Mouse and touch input with configurable responses
- Keyboard mappings for parameter control
- Gesture recognition for complex inputs
- Multi-modal interaction combining inputs
- Accessibility modes for inclusive experiences

Add new interaction paradigms by defining input mappings and feedback systems.

### Feature Backlog

The Cargo.toml features section enables modular capability expansion:

- wasm: WebAssembly compilation for browser deployment
- midi: MIDI input for external controller support
- postfx: Advanced post-processing effect chain

Future features can include:

- osc: Open Sound Control for networked communication
- video: Video input and output capabilities
- ai: Real-time AI model integration
- vr: Virtual reality rendering support
- multiuser: Networked collaborative experiences

Enable features by adding them to the default array or specifying at build time.

## Run Artifacts

Each creative run produces artifacts in .runs/<run-id>/ organized by lane:

- signal/: Art vision and creative direction
- plan/: Technical architecture and design documents
- build/: Generated source code and shaders
- gate/: Quality review documents and decisions
- deploy/: Release manifests and packages
- wisdom/: Retrospective analysis and learnings

Runs are immutable records of the creative process, enabling iteration and learning across sessions.

## Agent Architecture

Agents follow a consistent structure:

- YAML frontmatter with name, description, model, and color
- Invariants that must always hold
- Input parameters required for execution
- Single output file path
- Structured output format
- Control plane result block

Agents communicate through the filesystem, reading inputs from prior lanes and writing outputs to their designated lane.

## Contributing

To add new agents:

1. Create a new .md file in .claude/agents/
2. Follow the agent structure template
3. Define clear inputs, outputs, and invariants
4. Update relevant flow commands to invoke the agent

To add new flows:

1. Create a new .md file in .claude/commands/
2. Specify the lane name and agents involved
3. Define execution order and routing logic
4. Document outputs and next flow transitions

## License

MIT License - See LICENSE file for details.
