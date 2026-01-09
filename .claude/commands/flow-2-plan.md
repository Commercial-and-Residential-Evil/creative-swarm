# Flow 2: Plan

The Plan flow transforms art vision into concrete technical architecture and design documents.

## Purpose

Take the art vision from Signal flow and produce detailed technical plans for implementation including Bevy architecture, audio design, and interaction design.

## Lane

plan

## Agents Called

1. run-prep (if not already initialized for this lane)
2. rust-art-architect
3. audio-designer
4. interaction-designer

## Execution Order

First, verify the run directory exists and signal lane is complete.

If the plan lane directory does not exist, invoke run-prep:
- run_id: Existing run_id from Signal flow
- lane: plan
- project_root: Repository root path

Second, invoke rust-art-architect:
- run_id: The current run_id
- art_vision_path: .runs/<run-id>/signal/art_vision.md
- existing_systems: Empty array for new runs

Wait for rust-art-architect to complete with status VERIFIED.

Third, invoke audio-designer and interaction-designer in parallel:

For audio-designer:
- run_id: The current run_id
- art_vision_path: .runs/<run-id>/signal/art_vision.md
- architecture_path: .runs/<run-id>/plan/architecture.md

For interaction-designer:
- run_id: The current run_id
- art_vision_path: .runs/<run-id>/signal/art_vision.md
- architecture_path: .runs/<run-id>/plan/architecture.md
- target_platforms: [desktop] (default, or as specified)

Wait for both to complete.

## Routing Logic

If rust-art-architect returns status CANNOT_PROCEED:
- Action: ESCALATE
- Check art_vision.md exists and is valid
- May need to rerun Signal flow

If rust-art-architect returns status UNVERIFIED:
- Action: RERUN
- Review art vision for clarity
- Re-invoke with additional context

If rust-art-architect returns status VERIFIED:
- Action: PROCEED to parallel design agents

If audio-designer returns status UNVERIFIED:
- Action: RERUN just audio-designer
- Does not block interaction-designer

If interaction-designer returns status UNVERIFIED:
- Action: RERUN just interaction-designer
- Does not block audio-designer

If all three agents return status VERIFIED:
- Action: PASS
- Plan flow complete
- Ready to proceed to Flow 3: Build

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/plan/architecture.md
- .runs/<run-id>/plan/audio_design.md
- .runs/<run-id>/plan/interaction_design.md

## Next Flow

Proceed to flow-3-build.md with the same run_id.
