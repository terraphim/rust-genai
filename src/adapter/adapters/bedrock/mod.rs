//! AWS Bedrock Adapter Implementation
//!
//! API Documentation: https://docs.aws.amazon.com/bedrock/latest/APIReference/
//! Converse API: https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_Converse.html
//! ConverseStream API: https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_ConverseStream.html
//!
//! Supported Models:
//! - anthropic.claude-3-5-sonnet-20241022-v2:0
//! - anthropic.claude-3-5-haiku-20241022-v1:0
//! - anthropic.claude-3-opus-20240229-v1:0
//! - anthropic.claude-3-sonnet-20240229-v1:0
//! - anthropic.claude-3-haiku-20240307-v1:0
//! - meta.llama3-70b-instruct-v1:0
//! - meta.llama3-8b-instruct-v1:0
//! - amazon.titan-text-express-v1
//! - amazon.titan-text-lite-v1
//! - mistral.mistral-7b-instruct-v0:2
//! - mistral.mixtral-8x7b-instruct-v0:1
//! - cohere.command-r-plus-v1:0
//! - cohere.command-r-v1:0
//!
//! Environment Variables:
//! - AWS_ACCESS_KEY_ID: AWS Access Key ID
//! - AWS_SECRET_ACCESS_KEY: AWS Secret Access Key
//! - AWS_SESSION_TOKEN: (Optional) AWS Session Token for temporary credentials
//! - AWS_REGION: AWS Region (default: us-east-1)

mod adapter_impl;
mod aws_auth;
mod streamer;

pub use adapter_impl::*;
