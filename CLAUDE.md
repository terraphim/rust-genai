# rust-genai Agent Instructions

## Your Task
You are implementing issue #3 for the rust-genai Rust library.
Focus ALL your effort on writing production-quality Rust code and tests.
Do NOT run gitea-robot, tea, or any task-tracking commands.
The orchestrator handles all task tracking.

## Project Overview
rust-genai is a multi-AI provider Rust client library supporting OpenAI, Anthropic, Gemini,
Bedrock, Cerebras, DeepSeek, Groq, and more via a unified Adapter trait.

This is the Terraphim fork which adds: AWS Bedrock, Cerebras, Z.AI/Zhipu, OpenRouter adapters,
and Bearer token authentication.

The fork needs to be synced with upstream v0.6.0-beta.8 which has breaking API changes.

## Upstream Breaking Changes (v0.6.0-beta.8)
- Adapter trait: added const DEFAULT_API_KEY_ENV_NAME: &str
- Adapter trait: all_model_names now takes (kind, endpoint, auth) -- 3 params instead of 1
- Adapter trait: default_auth() now returns AuthData (AuthData::None variant added)
- ChatResponse: added stop_reason: Option<StopReason> field
- InterStreamEnd: added captured_stop_reason: Option<StopReason> field
- ChatMessage::tool renamed to ChatMessage::tool_response
- ContentPart::ReasoningContent added alongside ContentPart::Text
- ReasoningEffort::Max variant added
- OpenRouter removed as dedicated adapter upstream (namespace dispatch only)
- Aliyun adapter added upstream

## Key Files
- src/adapter/adapter_types.rs -- Adapter trait definition (source of truth for signatures)
- src/adapter/adapter_kind.rs -- AdapterKind enum (all adapter variants)
- src/adapter/dispatcher.rs -- Static dispatch to adapter implementations
- src/resolver/auth_data.rs -- AuthData enum (BearerToken + None coexistence)
- src/chat/chat_response.rs -- ChatResponse with stop_reason
- src/adapter/inter_stream.rs -- InterStreamEnd with captured_stop_reason
- src/adapter/adapters/bedrock/ -- Bedrock adapter (fork addition)
- src/adapter/adapters/cerebras/ -- Cerebras adapter (fork addition)
- src/adapter/adapters/openrouter/ -- OpenRouter adapter (fork addition)
- src/adapter/adapters/zai/ -- Z.AI adapter (fork, has URL trailing slash issues)

## Quality Requirements
- cargo build must succeed
- cargo test must pass
- cargo clippy -- -D warnings must pass
- cargo fmt -- --check must pass
- Use British English in documentation
- Never use mocks in tests
