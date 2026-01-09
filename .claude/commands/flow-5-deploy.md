# Flow 5: Deploy

The Deploy flow packages approved builds and prepares them for distribution.

## Purpose

Create release packages for approved builds, generate distribution manifests, and prepare artifacts for publication to various channels.

## Lane

deploy

## Agents Called

1. run-prep (if not already initialized for this lane)
2. packager

## Execution Order

First, verify the run directory exists and gate lane is complete with APPROVED decision.

Read .runs/<run-id>/gate/gate_decision.md and verify status is APPROVED.

If status is not APPROVED:
- Action: FAIL
- Cannot proceed to deployment without gate approval
- Direct user to resolve gate issues first

If the deploy lane directory does not exist, invoke run-prep:
- run_id: Existing run_id
- lane: deploy
- project_root: Repository root path

Second, invoke packager:
- run_id: The current run_id
- gate_decision_path: .runs/<run-id>/gate/gate_decision.md
- build_artifacts: All files from .runs/<run-id>/build/
- target_platforms: [linux, windows, macos, wasm] (or as specified)
- version: Determine from Cargo.toml or user input

## Routing Logic

If gate decision is not APPROVED:
- Action: FAIL
- Cannot package unapproved builds
- Return to Gate flow or Build flow

If packager returns status CANNOT_PROCEED:
- Action: ESCALATE
- Verify build artifacts exist
- Check gate decision file
- May need to rerun Gate flow

If packager returns status UNVERIFIED:
- Action: RERUN
- Provide additional packaging context
- Verify target platforms are valid

If packager returns status VERIFIED:
- Action: PASS
- Deploy flow complete
- Ready to proceed to Flow 6: Wisdom

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/deploy/release_manifest.md
- Platform-specific packages as specified in manifest

## Post-Deploy Actions

After successful packaging:

1. Copy finalized source to src/ directory (if not already done)
2. Copy finalized shaders to assets/shaders/ directory
3. Update Cargo.toml version if changed
4. Optionally commit changes to git
5. Optionally push packages to distribution channels

## Next Flow

Proceed to flow-6-wisdom.md with the same run_id for retrospective analysis.
