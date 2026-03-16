//! Test to verify that our hardcoded model lists match the actual adapter code
//! This test ensures consistency between our test expectations and the actual codebase
//!
//! NOTE: Since upstream v0.6.0-beta.8, DeepSeek, Groq, and ZAI no longer have static
//! model lists -- they use dynamic API-based model discovery. Only fork adapters that
//! still use static lists are checked here (Bedrock, Cerebras, Zhipu, Aliyun).

use std::collections::HashMap;

/// Get actual model lists from adapter source files
fn read_adapter_models() -> HashMap<String, Vec<String>> {
	let mut models = HashMap::new();

	// Read from actual adapter files
	let base_path = std::env::current_dir().unwrap();

	// Helper function to extract models from a const array
	fn extract_model_array(content: &str, start_marker: &str) -> Option<Vec<String>> {
		let start = content.find(start_marker)?;
		let array_start = start + start_marker.len();

		// Find the closing ]; that matches the opening [
		let mut depth = 0;
		let mut end_pos = array_start;

		for (i, ch) in content[array_start..].char_indices() {
			match ch {
				'[' => depth += 1,
				']' => {
					if depth == 0 {
						end_pos = array_start + i;
						break;
					}
					depth -= 1;
				}
				_ => {}
			}
		}

		let models_str = &content[array_start..end_pos];

		// Extract quoted strings, ignoring comments
		let mut model_list = Vec::new();
		for line in models_str.lines() {
			let cleaned = line.split("//").next().unwrap_or(line); // Remove comments
			for part in cleaned.split(',') {
				let trimmed = part.trim();
				if let Some(model) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
					&& !model.is_empty()
				{
					model_list.push(model.to_string());
				}
			}
		}

		Some(model_list)
	}

	// Bedrock models
	if let Ok(content) = std::fs::read_to_string(base_path.join("src/adapter/adapters/bedrock/adapter_impl.rs"))
		&& let Some(model_list) = extract_model_array(&content, "pub const MODELS: &[&str] = &[")
	{
		models.insert("Bedrock".to_string(), model_list);
	}

	// Cerebras models
	if let Ok(content) = std::fs::read_to_string(base_path.join("src/adapter/adapters/cerebras/adapter_impl.rs"))
		&& let Some(model_list) = extract_model_array(&content, "pub(in crate::adapter) const MODELS: &[&str] = &[")
	{
		models.insert("Cerebras".to_string(), model_list);
	}

	// Zhipu models
	if let Ok(content) = std::fs::read_to_string(base_path.join("src/adapter/adapters/zhipu/adapter_impl.rs"))
		&& let Some(model_list) = extract_model_array(&content, "pub(in crate::adapter) const MODELS: &[&str] = &[")
	{
		models.insert("Zhipu".to_string(), model_list);
	}

	// Aliyun models
	if let Ok(content) = std::fs::read_to_string(base_path.join("src/adapter/adapters/aliyun/adapter_impl.rs"))
		&& let Some(model_list) = extract_model_array(&content, "pub const MODELS: &[&str] = &[")
	{
		models.insert("Aliyun".to_string(), model_list);
	}

	models
}

/// Expected model lists from our test expectations
fn get_expected_models() -> std::collections::HashMap<String, Vec<String>> {
	let mut expected = std::collections::HashMap::new();

	// Bedrock models
	expected.insert(
		"Bedrock".to_string(),
		vec![
			"anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
			"anthropic.claude-3-5-haiku-20241022-v1:0".to_string(),
			"anthropic.claude-3-opus-20240229-v1:0".to_string(),
			"anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
			"anthropic.claude-3-haiku-20240307-v1:0".to_string(),
			"meta.llama3-2-90b-instruct-v1:0".to_string(),
			"meta.llama3-2-11b-instruct-v1:0".to_string(),
			"meta.llama3-2-3b-instruct-v1:0".to_string(),
			"meta.llama3-2-1b-instruct-v1:0".to_string(),
			"meta.llama3-1-405b-instruct-v1:0".to_string(),
			"meta.llama3-1-70b-instruct-v1:0".to_string(),
			"meta.llama3-1-8b-instruct-v1:0".to_string(),
			"meta.llama3-70b-instruct-v1:0".to_string(),
			"meta.llama3-8b-instruct-v1:0".to_string(),
			"amazon.titan-text-premier-v1:0".to_string(),
			"amazon.titan-text-express-v1".to_string(),
			"amazon.titan-text-lite-v1".to_string(),
			"mistral.mistral-large-2407-v1:0".to_string(),
			"mistral.mistral-large-2402-v1:0".to_string(),
			"mistral.mistral-small-2402-v1:0".to_string(),
			"mistral.mistral-7b-instruct-v0:2".to_string(),
			"mistral.mixtral-8x7b-instruct-v0:1".to_string(),
			"cohere.command-r-plus-v1:0".to_string(),
			"cohere.command-r-v1:0".to_string(),
			"ai21.jamba-1-5-large-v1:0".to_string(),
			"ai21.jamba-1-5-mini-v1:0".to_string(),
		],
	);

	// Cerebras models
	expected.insert(
		"Cerebras".to_string(),
		vec![
			"llama-3.3-70b".to_string(),
			"llama-3.1-70b".to_string(),
			"llama-3.1-8b".to_string(),
			"llama-3.2-11b-vision".to_string(),
			"llama-3.2-90b-vision".to_string(),
			"llama-guard-3-8b".to_string(),
		],
	);

	// Zhipu models
	expected.insert(
		"Zhipu".to_string(),
		vec![
			"glm-4-plus".to_string(),
			"glm-4-air-250414".to_string(),
			"glm-4-flashx-250414".to_string(),
			"glm-4-flash-250414".to_string(),
			"glm-4-air".to_string(),
			"glm-4-airx".to_string(),
			"glm-4-long".to_string(),
			"glm-4-flash".to_string(),
			"glm-4v-plus-0111".to_string(),
			"glm-4v-flash".to_string(),
			"glm-z1-air".to_string(),
			"glm-z1-airx".to_string(),
			"glm-z1-flash".to_string(),
			"glm-z1-flashx".to_string(),
			"glm-4.1v-thinking-flash".to_string(),
			"glm-4.1v-thinking-flashx".to_string(),
			"glm-4.5".to_string(),
		],
	);

	// Aliyun models
	expected.insert(
		"Aliyun".to_string(),
		vec![
			"qwen-turbo".to_string(),
			"qwen-plus".to_string(),
			"qwen-max".to_string(),
			"qwen-max-longcontext".to_string(),
			"qwen-turbo-latest".to_string(),
			"qwen-plus-latest".to_string(),
			"qwen-max-latest".to_string(),
			"qwen-vl-plus".to_string(),
			"qwen-vl-max".to_string(),
			"qwen-vl-plus-latest".to_string(),
			"qwen-7b-chat".to_string(),
			"qwen-14b-chat".to_string(),
			"qwen-72b-chat".to_string(),
			"qwen-72b-chat-int4".to_string(),
			"qwen-math-plus".to_string(),
			"qwen-math-turbo".to_string(),
			"qwen-math-plus-latest".to_string(),
			"qwen-math-turbo-latest".to_string(),
			"qwen-audio-turbo".to_string(),
			"qwen-audio-plus".to_string(),
			"qwen-audio-chat-v1".to_string(),
			"qwen-coder-plus".to_string(),
			"qwen-coder-turbo".to_string(),
			"qwen-coder-latest".to_string(),
		],
	);

	expected
}

/// Test that our test expectations match the actual adapter code
#[test]
fn test_adapter_code_consistency() -> Result<(), Box<dyn std::error::Error>> {
	println!("Verifying test expectations match actual adapter code...\n");

	let actual_models = read_adapter_models();
	let test_models = get_expected_models();

	let mut all_consistent = true;

	for (provider, expected_list) in &test_models {
		println!("=== Checking {} ===", provider);

		match actual_models.get(provider) {
			Some(actual_list) => {
				// Check for differences
				let expected_set: std::collections::HashSet<_> = expected_list.iter().collect();
				let actual_set: std::collections::HashSet<_> = actual_list.iter().collect();

				if expected_set == actual_set {
					println!("  Test and code models match");
					println!("  {} models", actual_set.len());
				} else {
					println!("  Mismatch found!");

					// Show differences
					let missing: Vec<_> = expected_set.difference(&actual_set).collect();
					let extra: Vec<_> = actual_set.difference(&expected_set).collect();

					if !missing.is_empty() {
						println!("  In test but not in code: {:?}", missing);
					}
					if !extra.is_empty() {
						println!("  In code but not in test: {:?}", extra);
					}
					all_consistent = false;
				}
			}
			None => {
				println!("  Provider {} not found in adapter code", provider);
				all_consistent = false;
			}
		}

		println!();
	}

	if all_consistent {
		println!("All model lists are consistent!");
	} else {
		println!("Some inconsistencies found");
		panic!("Model lists in tests don't match adapter code");
	}

	Ok(())
}
