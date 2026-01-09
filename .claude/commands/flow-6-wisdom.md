# Flow 6: Wisdom

The Wisdom flow conducts retrospective analysis and captures learnings for future creative runs.

## Purpose

Analyze the completed run to extract insights, identify patterns, and generate guidance that improves future creative iterations.

## Lane

wisdom

## Agents Called

1. run-prep (if not already initialized for this lane)
2. wisdom-retro

## Execution Order

First, verify the run directory exists.

Note: Wisdom flow can be run after any lane completion, not just after Deploy. It is valuable for learning from both successful and failed runs.

If the wisdom lane directory does not exist, invoke run-prep:
- run_id: Existing run_id
- lane: wisdom
- project_root: Repository root path

Second, collect all outputs from all completed lanes:
- .runs/<run-id>/signal/* (if exists)
- .runs/<run-id>/plan/* (if exists)
- .runs/<run-id>/build/* (if exists)
- .runs/<run-id>/gate/* (if exists)
- .runs/<run-id>/deploy/* (if exists)

Third, invoke wisdom-retro:
- run_id: The current run_id
- all_lane_outputs: Paths to all collected documents
- previous_retrospectives: Paths to wisdom/retrospective.md from past runs (if available)
- creative_goals: The original creative prompt from art_vision.md

## Routing Logic

If wisdom-retro returns status CANNOT_PROCEED:
- Action: ESCALATE
- Verify at least signal lane has outputs
- Cannot generate wisdom from empty run

If wisdom-retro returns status UNVERIFIED:
- Action: RERUN
- Provide additional context about run goals
- May need human input on creative assessment

If wisdom-retro returns status VERIFIED:
- Action: PASS
- Wisdom flow complete
- Full creative run cycle complete

## Outputs

Upon successful completion, the following files exist:
- .runs/<run-id>/wisdom/retrospective.md

## Using Wisdom

The retrospective document should be used to:

1. Inform the next creative prompt based on learnings
2. Update agent configurations if patterns emerge
3. Track creativity metrics over time
4. Build institutional knowledge about what works

## Archiving

After wisdom extraction, the run can be archived:

1. Update run_meta.json status to COMPLETED
2. Optionally compress .runs/<run-id>/ for storage
3. Index key learnings in a cross-run knowledge base

## Full Cycle Complete

With Wisdom flow complete, the full creative cycle is finished:

Signal -> Plan -> Build -> Gate -> Deploy -> Wisdom

To start a new creative run, return to flow-1-signal.md with a new run_id.

Consider feeding learnings from retrospective.md into the next creative prompt.
