# Flow 3: Build

The Build flow generates production code and shaders from the technical plans.

## Purpose

Transform architecture and design documents into working Rust code, WGSL shaders, and audio integration code that can be compiled and run.

## Lane

build

## Agents Called

1. run-prep (if not already initialized for this lane)
2. shader-sketcher (multiple invocations as needed)
3. bevy-implementer (multiple invocations for each module)

## Execution Order

First, verify the run directory exists and plan lane is complete.

If the build lane directory does not exist, invoke run-prep:
- run_id: Existing run_id
- lane: build
- project_root: Repository root path

Second, analyze architecture.md to identify required modules and shaders.

Third, invoke shader-sketcher for each required shader:
- run_id: The current run_id
- art_vision_path: .runs/<run-id>/signal/art_vision.md
- architecture_path: .runs/<run-id>/plan/architecture.md
- shader_type: As determined by architecture (vertex, fragment, compute, material)

Shaders can be generated in parallel if independent.

Fourth, invoke bevy-implementer for each module:
- run_id: The current run_id
- architecture_path: .runs/<run-id>/plan/architecture.md
- audio_design_path: .runs/<run-id>/plan/audio_design.md
- interaction_design_path: .runs/<run-id>/plan/interaction_design.md
- target_module: The specific module name

Modules with dependencies must be generated in dependency order.
Independent modules can be generated in parallel.

## Routing Logic

If shader-sketcher returns status CANNOT_PROCEED:
- Action: ESCALATE
- Review architecture for shader requirements
- May need manual shader authoring

If shader-sketcher returns status UNVERIFIED:
- Action: RERUN
- Provide additional context about visual requirements
- Check art_vision for clarity on effects

If bevy-implementer returns status CANNOT_PROCEED:
- Action: ESCALATE
- Review architecture for completeness
- Check for missing design documents

If bevy-implementer returns status UNVERIFIED:
- Action: RERUN
- Provide additional implementation context
- May need architecture clarification

If all shaders and modules return status VERIFIED:
- Action: PASS
- Build flow complete
- Ready to proceed to Flow 4: Gate

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/build/shaders/*.wgsl (one or more shader files)
- .runs/<run-id>/build/src/*.rs (one or more Rust modules)

## Compilation Check

Before marking Build as complete, verify that the generated code compiles:

1. Copy generated .rs files to src/ directory
2. Copy generated .wgsl files to assets/shaders/ directory
3. Run cargo check to verify compilation
4. If compilation fails, invoke bevy-implementer with error context

## Next Flow

Proceed to flow-4-gate.md with the same run_id.
