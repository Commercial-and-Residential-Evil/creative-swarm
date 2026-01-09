# Flow 1: Signal

The Signal flow captures creative intent and transforms it into actionable art direction. This is the entry point for all creative runs.

## Purpose

Convert a creative prompt or inspiration into a structured art vision document that downstream agents can execute against.

## Lane

signal

## Agents Called

1. run-prep
2. art-vision-author

## Execution Order

First, invoke run-prep to establish the run directory structure.

Pass the following parameters to run-prep:
- run_id: Generate from timestamp or accept from user
- lane: signal
- project_root: Repository root path

Wait for run-prep to complete with status VERIFIED.

Second, invoke art-vision-author to generate the art vision.

Pass the following parameters to art-vision-author:
- run_id: Same run_id from run-prep
- creative_prompt: The user's creative input
- constraints: Any technical or aesthetic constraints provided

Wait for art-vision-author to complete.

## Routing Logic

If run-prep returns status CANNOT_PROCEED:
- Action: FIX_ENV
- Do not proceed to art-vision-author
- Report environment issue to user

If run-prep returns status VERIFIED:
- Action: PROCEED to art-vision-author

If art-vision-author returns status VERIFIED:
- Action: PASS
- Signal flow complete
- Ready to proceed to Flow 2: Plan

If art-vision-author returns status UNVERIFIED:
- Action: RERUN
- Allow user to refine creative prompt
- Re-invoke art-vision-author with updated input

If art-vision-author returns status CANNOT_PROCEED:
- Action: ESCALATE
- Report issue to user
- May require manual art direction input

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/run_meta.json
- .runs/<run-id>/signal/art_vision.md

## Next Flow

Proceed to flow-2-plan.md with the same run_id.
