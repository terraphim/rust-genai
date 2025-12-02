//! AWS Bedrock integration tests
//!
//! These tests require the AWS Bedrock API key to be set:
//! - AWS_BEARER_TOKEN_BEDROCK (Bearer token API key)
//! - AWS_REGION (optional, defaults to us-east-1)
//!
//! To run these tests:
//! cargo test --test tests_p_bedrock -- --nocapture

mod support;

use crate::support::{TestResult, common_tests};
use genai::Client;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, Tool};
use serial_test::serial;

// Default models for testing - using Claude models via Bedrock
// Note: Model IDs for Bedrock use provider.model format
const MODEL: &str = "anthropic.claude-3-5-haiku-20241022-v1:0";
const MODEL_NS: &str = "bedrock::anthropic.claude-3-5-haiku-20241022-v1:0";

// Helper to check if AWS Bedrock API key is set
fn has_aws_credentials() -> bool {
	std::env::var("AWS_BEARER_TOKEN_BEDROCK").is_ok()
}

// region:    --- Basic Chat Tests

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_chat_simple_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_chat_simple_ok(MODEL_NS, None).await
}

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_chat_temperature_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_chat_temperature_ok(MODEL_NS).await
}

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_chat_multi_system_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_chat_multi_system_ok(MODEL_NS).await
}

// endregion: --- Basic Chat Tests

// region:    --- Streaming Tests

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_chat_stream_simple_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_chat_stream_simple_ok(MODEL_NS, None).await
}

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_chat_stream_capture_content_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_chat_stream_capture_content_ok(MODEL_NS).await
}

// endregion: --- Streaming Tests

// region:    --- Tool Tests

#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_tool_simple_ok() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}
	common_tests::common_test_tool_simple_ok(MODEL_NS).await
}

// endregion: --- Tool Tests

// region:    --- Model List Tests

#[tokio::test]
async fn test_bedrock_list_models() -> TestResult<()> {
	common_tests::common_test_list_models(AdapterKind::Bedrock, MODEL).await
}

// endregion: --- Model List Tests

// region:    --- Manual Tests (with custom assertions)

/// Test basic chat request to Bedrock
#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_basic_chat() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![
		ChatMessage::system("You are a helpful assistant."),
		ChatMessage::user("Say 'Hello from Bedrock!' and nothing else."),
	]);

	let result = client.exec_chat(MODEL_NS, chat_req, None).await?;
	let content = result.first_text().ok_or("Should have content")?;

	assert!(!content.is_empty(), "Content should not be empty");
	println!("Bedrock response: {}", content);

	Ok(())
}

/// Test streaming response from Bedrock
#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_streaming_chat() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}

	let client = Client::default();
	let chat_req = ChatRequest::new(vec![ChatMessage::user("Count from 1 to 5.")]);

	let options = ChatOptions::default().with_capture_content(true);

	let chat_res = client.exec_chat_stream(MODEL_NS, chat_req, Some(&options)).await?;
	let stream_extract = support::extract_stream_end(chat_res.stream).await?;
	let content = stream_extract.content.ok_or("Should have content")?;

	assert!(!content.is_empty(), "Streaming content should not be empty");
	println!("Bedrock streaming response: {}", content);

	Ok(())
}

/// Test tool/function calling with Bedrock
#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_tool_calling() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}

	let client = Client::default();

	let tool = Tool::new("get_weather")
		.with_description("Get the current weather in a location")
		.with_schema(serde_json::json!({
			"type": "object",
			"properties": {
				"location": {
					"type": "string",
					"description": "The city and state, e.g. San Francisco, CA"
				}
			},
			"required": ["location"]
		}));

	let chat_req = ChatRequest::new(vec![ChatMessage::user("What's the weather in Seattle?")]).append_tool(tool);

	let result = client.exec_chat(MODEL_NS, chat_req, None).await?;

	// Check if we got a response (either text or tool call)
	let has_content = result.first_text().is_some() || !result.content.tool_calls().is_empty();
	assert!(has_content, "Should have either text or tool call response");

	println!("Bedrock tool response: {:?}", result.content);

	Ok(())
}

/// Test with different Bedrock model (Meta Llama)
#[tokio::test]
#[serial(bedrock)]
async fn test_bedrock_llama_model() -> TestResult<()> {
	if !has_aws_credentials() {
		println!("Skipping Bedrock test - AWS_BEARER_TOKEN_BEDROCK not set");
		return Ok(());
	}

	let client = Client::default();
	let llama_model = "bedrock::meta.llama3-8b-instruct-v1:0";

	let chat_req = ChatRequest::new(vec![ChatMessage::user("What is 2 + 2? Answer with just the number.")]);

	let result = client.exec_chat(llama_model, chat_req, None).await?;
	let content = result.first_text().ok_or("Should have content")?;

	assert!(!content.is_empty(), "Content should not be empty");
	println!("Bedrock Llama response: {}", content);

	Ok(())
}

// endregion: --- Manual Tests

// region:    --- Model Resolution Tests

#[tokio::test]
async fn test_bedrock_model_resolution() -> TestResult<()> {
	use genai::adapter::AdapterKind;

	// Test that Bedrock models are correctly resolved
	let models = vec![
		"anthropic.claude-3-5-sonnet-20241022-v2:0",
		"anthropic.claude-3-haiku-20240307-v1:0",
		"meta.llama3-70b-instruct-v1:0",
		"amazon.titan-text-express-v1",
		"mistral.mistral-7b-instruct-v0:2",
	];

	for model in models {
		let kind = AdapterKind::from_model(model)?;
		assert_eq!(kind, AdapterKind::Bedrock, "Model {} should resolve to Bedrock", model);
	}

	// Test explicit namespace
	let kind = AdapterKind::from_model("bedrock::some-custom-model")?;
	assert_eq!(kind, AdapterKind::Bedrock, "Namespaced model should resolve to Bedrock");

	Ok(())
}

// endregion: --- Model Resolution Tests
