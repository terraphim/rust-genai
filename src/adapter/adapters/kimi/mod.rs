//! Kimi (Moonshot AI) adapter - Anthropic-compatible API at https://api.kimi.com/coding/v1/
//!
//! Delegates to AnthropicAdapter for protocol handling, but uses a different
//! default endpoint and API key environment variable.

// region:    --- Modules

mod adapter_impl;

pub use adapter_impl::*;

// endregion: --- Modules
