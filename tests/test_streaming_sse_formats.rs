//! Streaming SSE format verification tests for Kimi and Zai adapters.
//!
//! Different providers use different SSE (Server-Sent Events) formats:
//!
//! - Kimi uses Anthropic-style SSE (event: content_block_delta, event: message_stop)
//!   and delegates streaming entirely to AnthropicAdapter::to_chat_stream.
//!
//! - Zai uses OpenAI-style SSE (data: {"choices":[...]}, data: [DONE])
//!   and delegates streaming entirely to OpenAIAdapter::to_chat_stream.
//!
//! No custom streamer implementations are needed for either adapter, since
//! both providers are fully compatible with their respective base adapters.
//!
//! These tests verify streaming works end-to-end through the delegation pattern
//! for both regular and namespaced model identifiers.

mod support;

use genai::Client;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use support::{TestResult, extract_stream_end};

/// Check if an environment variable is set (for skipping tests without API keys).
fn has_env_key(key: &str) -> bool {
	std::env::var(key).is_ok()
}

// region:    --- Kimi Streaming (Anthropic-style SSE)

/// Verify Kimi streaming produces content via Anthropic SSE protocol.
///
/// Kimi's to_chat_stream delegates to AnthropicAdapter::to_chat_stream,
/// which uses AnthropicStreamer to parse event: content_block_delta messages.
#[tokio::test]
#[ignore] // Requires KIMI_API_KEY
async fn test_kimi_streaming_produces_content() -> TestResult<()> {
	if !has_env_key("KIMI_API_KEY") {
		println!("Skipping: KIMI_API_KEY not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user("Say 'hello' and count to 3")]);
	let options = ChatOptions::default().with_capture_content(true).with_capture_usage(true);

	let stream_res = client
		.exec_chat_stream("kimi::claude-3-5-sonnet-20241022", chat_req, Some(&options))
		.await?;

	let extract = extract_stream_end(stream_res.stream).await?;
	let content = extract.content.ok_or("Kimi stream should produce content")?;
	assert!(!content.trim().is_empty(), "Kimi streamed content should not be empty");

	// Verify stream_end metadata is populated
	let stream_end = extract.stream_end;
	assert!(
		stream_end.captured_content.is_some(),
		"Kimi stream should capture content when option is set"
	);

	Ok(())
}

/// Verify Kimi streaming works with multiple chunks accumulated.
#[tokio::test]
#[ignore] // Requires KIMI_API_KEY
async fn test_kimi_streaming_accumulates_chunks() -> TestResult<()> {
	if !has_env_key("KIMI_API_KEY") {
		println!("Skipping: KIMI_API_KEY not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user(
		"List the numbers 1 through 5, each on a new line",
	)]);
	let options = ChatOptions::default().with_capture_content(true);

	let stream_res = client
		.exec_chat_stream("kimi::claude-3-5-sonnet-20241022", chat_req, Some(&options))
		.await?;

	let extract = extract_stream_end(stream_res.stream).await?;
	let content = extract.content.ok_or("Should have content")?;

	// Content should contain multiple numbers, confirming chunks were accumulated
	assert!(content.contains('1'), "Should contain number 1");
	assert!(content.contains('5'), "Should contain number 5");

	Ok(())
}

// endregion: --- Kimi Streaming (Anthropic-style SSE)

// region:    --- Zai Streaming (OpenAI-style SSE)

/// Verify Zai streaming produces content via OpenAI SSE protocol.
///
/// Zai's to_chat_stream delegates to OpenAIAdapter::to_chat_stream,
/// which uses OpenAIStreamer to parse data: {"choices":[...]} messages.
#[tokio::test]
#[ignore] // Requires ZAI_API_KEY
async fn test_zai_streaming_produces_content() -> TestResult<()> {
	if !has_env_key("ZAI_API_KEY") {
		println!("Skipping: ZAI_API_KEY not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user("Say 'hello' and count to 3")]);
	let options = ChatOptions::default().with_capture_content(true).with_capture_usage(true);

	let stream_res = client.exec_chat_stream("glm-4-plus", chat_req, Some(&options)).await?;

	let extract = extract_stream_end(stream_res.stream).await?;
	let content = extract.content.ok_or("Zai stream should produce content")?;
	assert!(!content.trim().is_empty(), "Zai streamed content should not be empty");

	// Verify stream_end metadata is populated
	let stream_end = extract.stream_end;
	assert!(
		stream_end.captured_content.is_some(),
		"Zai stream should capture content when option is set"
	);

	Ok(())
}

/// Verify Zai streaming works with namespaced model (zai:: prefix).
///
/// The zai:: namespace routes to the regular API endpoint.
/// Streaming should work identically to the non-namespaced model.
#[tokio::test]
#[ignore] // Requires ZAI_API_KEY
async fn test_zai_streaming_namespaced_produces_content() -> TestResult<()> {
	if !has_env_key("ZAI_API_KEY") {
		println!("Skipping: ZAI_API_KEY not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user("Say 'hello' and count to 3")]);
	let options = ChatOptions::default().with_capture_content(true);

	let stream_res = client.exec_chat_stream("zai::glm-4-plus", chat_req, Some(&options)).await?;

	let extract = extract_stream_end(stream_res.stream).await?;
	let content = extract.content.ok_or("Zai namespaced stream should produce content")?;
	assert!(
		!content.trim().is_empty(),
		"Zai namespaced streamed content should not be empty"
	);

	Ok(())
}

/// Verify Zai streaming accumulates multiple chunks correctly.
#[tokio::test]
#[ignore] // Requires ZAI_API_KEY
async fn test_zai_streaming_accumulates_chunks() -> TestResult<()> {
	if !has_env_key("ZAI_API_KEY") {
		println!("Skipping: ZAI_API_KEY not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user(
		"List the numbers 1 through 5, each on a new line",
	)]);
	let options = ChatOptions::default().with_capture_content(true);

	let stream_res = client.exec_chat_stream("glm-4-plus", chat_req, Some(&options)).await?;

	let extract = extract_stream_end(stream_res.stream).await?;
	let content = extract.content.ok_or("Should have content")?;

	// Content should contain multiple numbers, confirming chunks were accumulated
	assert!(content.contains('1'), "Should contain number 1");
	assert!(content.contains('5'), "Should contain number 5");

	Ok(())
}

// endregion: --- Zai Streaming (OpenAI-style SSE)
