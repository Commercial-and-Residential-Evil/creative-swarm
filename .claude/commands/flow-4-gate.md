# Flow 4: Gate

The Gate flow reviews built artifacts for quality, performance, and safety before deployment approval.

## Purpose

Conduct thorough review of generated code and shaders to ensure they meet quality standards, perform acceptably, and have no safety or licensing issues.

## Lane

gate

## Agents Called

1. run-prep (if not already initialized for this lane)
2. performance-critic
3. safety-license-critic
4. gatekeeper

## Execution Order

First, verify the run directory exists and build lane is complete.

If the gate lane directory does not exist, invoke run-prep:
- run_id: Existing run_id
- lane: gate
- project_root: Repository root path

Second, invoke performance-critic and safety-license-critic in parallel:

For performance-critic:
- run_id: The current run_id
- source_files: All .rs files from .runs/<run-id>/build/src/
- shader_files: All .wgsl files from .runs/<run-id>/build/shaders/
- target_fps: 60 (default)
- target_entity_count: 10000 (default)

For safety-license-critic:
- run_id: The current run_id
- source_files: All .rs files from .runs/<run-id>/build/src/
- cargo_toml_path: Cargo.toml
- target_license: MIT (default)

Wait for both critics to complete.

Third, invoke gatekeeper:
- run_id: The current run_id
- performance_review_path: .runs/<run-id>/gate/performance_review.md
- safety_review_path: .runs/<run-id>/gate/safety_review.md
- build_artifacts: List of all built files

## Routing Logic

If performance-critic returns status CANNOT_PROCEED:
- Action: ESCALATE
- Verify build artifacts exist
- May need to rerun Build flow

If performance-critic returns status UNVERIFIED:
- Action: RERUN
- Provide additional performance context
- Check for missing source files

If safety-license-critic returns status CANNOT_PROCEED:
- Action: ESCALATE
- Verify Cargo.toml exists
- Check dependency resolution

If safety-license-critic returns status UNVERIFIED:
- Action: RERUN
- Verify source file paths
- Check for access issues

If gatekeeper returns status CANNOT_PROCEED:
- Action: ESCALATE
- Verify both review documents exist
- May need to rerun critics

If gatekeeper decision is REJECTED:
- Action: FAIL
- Return blocking reasons to user
- Options:
  - Fix issues and rerun Build flow
  - Request human override
  - Abandon run

If gatekeeper decision is APPROVED:
- Action: PASS
- Gate flow complete
- Ready to proceed to Flow 5: Deploy

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/gate/performance_review.md
- .runs/<run-id>/gate/safety_review.md
- .runs/<run-id>/gate/gate_decision.md

## Next Flow

If APPROVED, proceed to flow-5-deploy.md with the same run_id.
If REJECTED, return to flow-3-build.md after addressing issues.
