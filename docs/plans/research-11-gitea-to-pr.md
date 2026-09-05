# Research Document: Empty streaming tool-call id splits name and arguments

**Status**: Review
**Canonical Path**: `docs/plans/research-11-gitea-to-pr.md`
**Change Slug**: `11-gitea-to-pr`
**Author**: Terraphim Engineer (parent session unblocking gitea-to-pr-2)
**Date**: 2026-09-05
**Issue**: [terraphim/rust-genai#11](https://git.terraphim.cloud/terraphim/rust-genai/issues/11)
**Companion**: [terraphim/grok-build#41](https://git.terraphim.cloud/terraphim/grok-build/issues/41)
**Reviewers**: (pending)

## Executive Summary

The OpenAI-compatible streamer in this crate treats a present-but-empty `id` on a later tool-call delta as a new `call_id` (`""`) instead of reusing the id captured for that `index`. terraphim-llm-proxy re-emits OpenAI SSE keyed by `call_id`, so an empty id mints a new `index`. grok-build then dispatches two calls: `run_terminal_command` with empty arguments, and a nameless call holding the real JSON. That is the TUI error `Agent tried calling a tool that doesn't exist: ` (blank name).

`capture_tool_call` already merges by index **when** `capture_tool_calls` is true, and in that mode it keeps the original id if `fn_name` is empty. The proxy never sets that flag (`StreamerOptions` defaults it to `false`), so each `ToolCallChunk` is the unmerged fragment. The fix belongs in id resolution for **every** chunk, not only the capture-enabled path.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | Agent loops on `proxy-fastest` → `qwen3.8-flash` cannot run any tool. Observed 10 retries in session `01a07102-570c-79c2-b867-686596e41cdb`. |
| Leverages strengths? | Yes | Localised to `src/adapter/adapters/openai/streamer.rs`; Aliyun/DashScope already delegates here. |
| Meets real need? | Yes | grok-build#41 consumer-side coalesce is defensive only. The split is minted in this crate’s streamer when `id` is `""`. |

**Proceed**: Yes (3/3 YES).

## Problem Statement

### Description

OpenAI streaming tool calls arrive across chunks:

1. First chunk: `{index: 0, id: "call_abc", function: {name: "run_terminal_command", arguments: ""}}`
2. Later chunks: `{index: 0, function: {arguments: "{\"command\":...}"}}` — `id` omitted **or** `"id": ""`

Current code:

```rust
let call_id = tool_call_obj
    .x_take::<String>("id")
    .unwrap_or_else(|_| format!("call_{index}"));
```

`x_take::<String>("id")` succeeds on `""`. The fallback `call_{index}` never runs. The emitted `ToolCall.call_id` is empty.

### Impact

- grok-build via terraphim-llm-proxy sees two tool calls and cannot execute `run_terminal_command`.
- Any OpenAI-compat vendor that sends empty `id` on argument fragments is affected (DashScope/Qwen confirmed; Kimi/Zai use this same streamer when they speak OpenAI SSE).
- The TUI title blames a missing tool; the tool exists.

### Success Criteria

1. Synthetic SSE: chunk 0 with real id+name and empty args, chunk 1 with `id: ""` and argument JSON → **one** `ToolCallChunk` stream whose `call_id` stays `call_abc` and whose `fn_name` stays `run_terminal_command`, with concatenated arguments.
2. Missing `id` (field absent) on chunk 1 still reuses the captured id for that index; do not synthesise `call_0` if a real id was already seen.
3. Parallel tool calls (index 0 and 1) still remain two calls.
4. `capture_tool_calls = true` path still concatenates arguments and does not regress.
5. Default `capture_tool_calls = false` (proxy path) is fixed; that is the production path.

## Current State Analysis

### Existing Implementation

| Component | Location | Purpose |
|-----------|----------|---------|
| OpenAI streamer | `src/adapter/adapters/openai/streamer.rs` | Parses `delta.tool_calls`; mints `call_id`; emits `InterStreamEvent::ToolCallChunk` |
| `capture_tool_call` | same file, lines 43–74 | Merge-by-index **only if** `options.capture_tool_calls` |
| Streamer options | `src/adapter/adapters/support.rs:33` | `capture_tool_calls: options_set.capture_tool_calls().unwrap_or(false)` |
| Aliyun/DashScope adapter | `src/adapter/adapters/aliyun/adapter_impl.rs` | Delegates streaming to `OpenAIAdapter` |
| Proxy OpenAI SSE re-emit | `terraphim-llm-proxy/src/server.rs` ~2170 | `tool_call_indices.entry(call_id)` — empty id mints new index |
| grok-build assembler | `grok-build/.../stream/chat_completions.rs` | Keys by `ToolCallDelta.index`; no coalesce |

### Data Flow

```
DashScope/Qwen SSE
  → rust-genai OpenAIStreamer (this crate)
      call_id = x_take("id")  // "" is Ok("")
      capture_tool_calls=false → return unmerged ToolCall
  → terraphim-llm-proxy ChatStreamEvent::ToolCallChunk
      index = map[call_id] or next++   // "" is a new key
  → grok-build stream_chat_completions
      tool_call_acc[index]
  → dispatch: named empty-args + nameless full-args
```

### Integration Points

- terraphim-llm-proxy depends on `genai = { git = "https://github.com/terraphim/rust-genai", branch = "main" }`.
- Gitea canonical tracker: `https://git.terraphim.cloud/terraphim/rust-genai`.
- Local origin is GitHub `terraphim/rust-genai`. A Gitea remote may need adding for `gtr create-pull`.

## Constraints

### Technical Constraints

- Must not require `capture_tool_calls = true`. Proxy leaves it false; enabling capture would change event payload shape (accumulated args on later chunks) and is a separate decision.
- Must not collapse two genuine parallel tool calls that share empty ids (rare). Rule: reuse last **non-empty** id for the **same index** only.
- `x_take` removes the field; tests must use realistic JSON.

### Business Constraints

- One PR for this repo. grok-build#41 is a separate consumer-side PR.
- Do not merge. Do not restart ADF.

### Non-Functional Requirements

| Requirement | Target | Current |
|-------------|--------|---------|
| Extra per-chunk work | O(1) lookup by index | none |
| Live API tests | not required for this bug | existing SSE tests are `#[ignore]` live |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Empty `id` must be treated as absent | This is the observed wire | grok-build session payload `id: ""` on sibling call |
| Reuse captured id **per index**, even when capture is off | Proxy path is capture=false | `support.rs:33` unwrap_or(false); proxy never sets the flag |
| Keep parallel calls (distinct indices) separate | Must not merge two real tools | OpenAI `index` is the correlation key |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|---------------|
| grok-build assembler coalesce | Separate repo, grok-build#41 |
| MiniMax XML scanner | Different wire; grok-build xml_tool_calls |
| Enabling `capture_tool_calls` in the proxy | Behavioural change for all vendors; not required if id reuse is correct |
| proxy-fastest pool membership | Workaround, not a fix |
| Aliyun adapter rewrite | It already delegates to OpenAIStreamer |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `JsonValueExt::x_take` | Empty string vs missing field | Low — treat both as absent |
| `StreamerCapturedData.tool_calls` | Capture path already merges by index | Low — keep that path |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| terraphim-llm-proxy | git dep on this crate `main` | Medium — proxy will not pick up the fix until this lands on GitHub `main` | Pin a rev after merge |
| grok-build | independent | Low — defensive coalesce can land first | n/a |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Vendor sends empty id on the **first** chunk | Low | Would still synthesise `call_{index}` | Only reuse when a prior non-empty id exists for that index |
| `x_take` on `id: null` | Med | Depends on JsonValueExt | Test null, missing, and `""` |
| GitHub vs Gitea remotes | High | PR opened in the wrong host | Push to both; proxy consumes GitHub |
| `proxy-kimi` empty responses | High (this run) | Workflow agents stall | Parent wrote this artefact; design must read it from disk |

### Open Questions

1. Does DashScope `fastest` go through genai OpenAIStreamer or BailianClient’s own SSE parser? — Proxy has both. The OpenAI `/v1/chat/completions` streaming path in `server.rs:2165` is genai `ToolCallChunk`. BailianClient is a separate provider client. Session model was `qwen3.8-flash` via `fastest`; confirm in design which client `fastest` used. **The streamer bug is real either way** and is the issue scope.
2. Should later chunks with empty `fn_name` copy the captured name into the **emitted** ToolCall even when capture is off? Yes — otherwise the proxy’s first-chunk-for-new-id branch would still send `name: ""` if call_id reuse failed.

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| Observed sibling `id: ""` is produced by this streamer (or an equivalent empty-id pass-through) | Code path + proxy re-index by call_id | If Qwen already sends two indices, grok-build#41 still needed | Partially (no raw SSE dump) |
| `capture_tool_calls` is false on the proxy | `unwrap_or(false)` and no proxy call to `with_capture_tool_calls` | If some routes enable it, capture path already preserves id | Yes for default |
| Index is present on argument chunks | Streamer requires `Ok(index)` to enter the tool-call branch | If index missing, argument chunks are dropped not split | Yes (code) |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| A. Fix only capture=true merge | Proxy stays broken | Rejected — production path is capture=false |
| B. Always resolve empty/missing id against last non-empty id for this index | Fixes proxy and capture paths | Chosen |
| C. Synthesise `call_{index}` for empty id | Would give proxy a stable non-empty key **if index is reused** | Rejected as primary: empty id on index 0 would become `call_0`, which is a **different** key from `call_abc`, and the proxy would still split |

Interpretation C is the current fallback for **missing** id and is itself a split if the first chunk had a real id. The fix must reuse the captured real id, not `call_{index}`.

## Research Findings

### Key Insights

1. Empty string is not “missing”. `unwrap_or_else` never runs.
2. `capture_tool_call` already knows how to keep the original id when `fn_name` is empty — but only mutates `captured_data` when capture is on. The **returned** ToolCall for capture=false is the fragment with empty id.
3. Proxy keys by `call_id`, not upstream `index`. An empty id is a new tool.
4. AliyunAdapter is not a second implementation; it delegates to OpenAIAdapter.

### Relevant Prior Art

- grok-build XML scanner (`xml_tool_call_scanner.rs`) — unrelated wire.
- terraphim-llm-proxy#201 empty `tool_call_id` on Kimi — same family, Anthropic-shaped path.
- OpenAI streaming spec: later deltas omit `id` and `name`, correlate by `index`.

### Technical Spikes Needed

None. The failing payload is recorded. A unit test with two JSON chunks is sufficient.

## Recommendations

### Proceed/No-Proceed

Proceed. Root cause is identified in this crate. Fix is local to `OpenAIStreamer::poll_next` + `capture_tool_call` id resolution.

### Scope Recommendations

- Add a small per-streamer map `index -> (call_id, fn_name)` that is **always** updated, independent of `capture_tool_calls`.
- When `id` is missing or empty, look up that map; only then fall back to `call_{index}`.
- When `fn_name` is empty, look up the captured name for emit.
- Unit test in `src/adapter/adapters/openai/` (or `tests/`) with no live API.

### Risk Mitigation Recommendations

- Do not enable `capture_tool_calls` as the fix.
- Keep grok-build#41 as belt-and-braces for vendors that increment `index` on argument chunks.

## Next Steps

If approved:

1. Design: function signatures for id/name resolution helper; test table (empty, missing, null, parallel indices).
2. Implement TDD on branch `task/11-empty-tool-call-id` from `origin/main`.
3. Push to GitHub `origin` (proxy dep) and Gitea (tracker PR).
4. grok-build#41 lands independently.

## Appendix

### Reference Materials

- Issue: https://git.terraphim.cloud/terraphim/rust-genai/issues/11
- Companion: https://git.terraphim.cloud/terraphim/grok-build/issues/41
- Session: `~/.grok/sessions/%2Fhome%2Falex%2Fprojects%2Fterraphim/01a07102-570c-79c2-b867-686596e41cdb/`

### Code Snippets

```rust
// streamer.rs:276-280 — the defect
let call_id = tool_call_obj
    .x_take::<String>("id")
    .unwrap_or_else(|_| format!("call_{index}"));
let fn_name = function.x_take::<String>("name").unwrap_or_default();
```

```rust
// support.rs:33 — production default
capture_tool_calls: options_set.capture_tool_calls().unwrap_or(false),
```
