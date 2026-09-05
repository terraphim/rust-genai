# Implementation Plan: Reuse captured tool-call id when later deltas send empty id

**Status**: Review
**Research Doc**: `docs/plans/research-11-gitea-to-pr.md`
**Change Slug**: `11-gitea-to-pr`
**Author**: Terraphim Engineer
**Date**: 2026-09-05
**Estimated Effort**: 2–3 hours
**Issue**: terraphim/rust-genai#11

## Overview

### Summary

Resolve OpenAI streaming tool-call `id` and `name` **per index** on every chunk, including when `capture_tool_calls` is false (the terraphim-llm-proxy path). Empty or missing `id` reuses the last non-empty id for that index instead of emitting `call_id == ""` or synthesising `call_{index}` after a real id was seen.

### Approach

Keep a small `HashMap<u32, (String, String)>` on `OpenAIStreamer` (`index -> (call_id, fn_name)`), always updated, independent of `capture_tool_calls`. Resolve id/name before `capture_tool_call`.

### Scope

**In Scope:**
- `src/adapter/adapters/openai/streamer.rs` resolution + unit tests
- Empty, missing, and (if `x_take` yields it) null `id`
- Empty `fn_name` reuse

**Out of Scope:**
- Enabling `capture_tool_calls` in the proxy
- grok-build assembler coalesce (grok-build#41)
- MiniMax XML
- Aliyun adapter rewrite (already delegates here)

**Avoid At All Cost:**
- Using `call_{index}` as the primary id after a real id was captured for that index
- Turning `capture_tool_calls` on as the fix
- Merging distinct indices

## Architecture

### Data Flow

```
delta.tool_calls[0]
  → parse index, function
  → raw_id = take id; treat "" as absent
  → raw_name = take name; treat "" as absent
  → lookup map[index]
  → call_id = raw_id or captured_id or format!("call_{index}")
  → fn_name = raw_name or captured_name or ""
  → store map[index] = (call_id, fn_name)
  → capture_tool_call(index, call_id, fn_name, arguments)
  → emit ToolCallChunk
```

### Key Design Decisions

| Decision | Rationale | Rejected |
|----------|-----------|----------|
| Always-on per-index map | Proxy uses capture=false | Enable capture_tool_calls |
| Empty string == absent | Observed wire `"id": ""` | Only handle missing field |
| Reuse captured real id, not `call_{index}` | `call_0` ≠ `call_abc`; proxy would still split | Synthesise call_{index} for empty id |

### Simplicity Check

What if this could be easy? Ten lines of lookup before the existing `capture_tool_call`. No new crate, no proxy change required for this PR.

## File Changes

### Modified Files

| File | Changes |
|------|---------|
| `src/adapter/adapters/openai/streamer.rs` | Add `tool_id_by_index: HashMap<u32, (String, String)>` on `OpenAIStreamer`. Resolve id/name in the tool-call branch. Unit tests for empty/missing id. |

### New Files

None.

## API Design

No public API change. Internal helper:

```rust
fn resolve_tool_identity(
    map: &mut HashMap<u32, (String, String)>,
    index: u32,
    raw_id: Option<String>,
    raw_name: Option<String>,
) -> (String, String)
```

- `raw_id`/`raw_name` are `None` when the field is missing, empty, or unreadable.
- Returns the id and name to put on this chunk’s `ToolCall`.

## Test Strategy

### Unit Tests (in `streamer.rs` `mod tests`)

| Test | Purpose |
|------|---------|
| `empty_id_on_second_chunk_reuses_captured_id` | Chunk0: id=call_abc, name=run_terminal_command, args="". Chunk1: id="", args=`{"command":"true"}`. Both emitted ToolCalls have call_id=call_abc and fn_name=run_terminal_command. |
| `missing_id_on_second_chunk_reuses_captured_id` | Same with id field omitted. |
| `parallel_indices_stay_separate` | Index 0 and 1 keep distinct ids. |
| `first_chunk_empty_id_synthesises_call_index` | No prior capture → `call_0`. |
| `capture_tool_calls_true_still_concatenates_args` | Existing merge behaviour. |

If driving the full `Stream` is heavy, extract `resolve_tool_identity` and test it directly, plus one streamer integration if a fake EventSource is already available. Prefer testing the helper plus the capture_tool_call return value.

No live API. No mocks of the HTTP client — table-driven helper tests are real logic tests.

## Implementation Steps

### Step 1: Helper + tests (RED)

Add `resolve_tool_identity` and the empty-id tests. Tests fail until Step 2.

### Step 2: Wire into poll_next (GREEN)

In the tool-call branch, replace the current `x_take::<String>("id").unwrap_or_else(...)` with the helper. Initialise the HashMap in `OpenAIStreamer::new`.

### Step 3: fmt / clippy / tests

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets --no-fail-fast` for this crate.

## Rollback Plan

Revert the single-file commit. No data migration. No feature flag.

## Open Items

| Item | Status |
|------|--------|
| Push to GitHub origin (proxy git dep) **and** Gitea | Required at PR time |
| Local checkout is on `fix/anthropic-...-v2`; implement from `origin/main` | Do not mix |

## Approval

- [ ] Design agent / human review
- [ ] Implement on `task/11-empty-tool-call-id` from `origin/main`
