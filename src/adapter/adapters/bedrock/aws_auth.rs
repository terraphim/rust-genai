//! AWS Signature Version 4 authentication for Bedrock API
//!
//! This implements the AWS SigV4 signing process for HTTP requests.
//! See: https://docs.aws.amazon.com/general/latest/gr/sigv4_signing.html

use crate::{Error, Result};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// AWS Credentials loaded from environment
#[derive(Debug, Clone)]
pub struct AwsCredentials {
	pub access_key_id: String,
	pub secret_access_key: String,
	pub session_token: Option<String>,
	pub region: String,
}

impl AwsCredentials {
	/// Load credentials from environment variables
	pub fn from_env() -> Result<Self> {
		let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
			.map_err(|_| Error::Internal("AWS_ACCESS_KEY_ID environment variable not set".to_string()))?;

		let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
			.map_err(|_| Error::Internal("AWS_SECRET_ACCESS_KEY environment variable not set".to_string()))?;

		let session_token = std::env::var("AWS_SESSION_TOKEN").ok();

		let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

		Ok(Self {
			access_key_id,
			secret_access_key,
			session_token,
			region,
		})
	}
}

/// AWS SigV4 signer for Bedrock requests
pub struct AwsSigV4Signer {
	credentials: AwsCredentials,
	service: String,
}

impl AwsSigV4Signer {
	/// Create a new signer for Bedrock
	pub fn new(credentials: AwsCredentials) -> Self {
		Self {
			credentials,
			service: "bedrock".to_string(),
		}
	}

	/// Sign a request and return the required headers
	pub fn sign_request(
		&self,
		method: &str,
		url: &str,
		headers: &BTreeMap<String, String>,
		payload: &[u8],
	) -> Result<BTreeMap<String, String>> {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_err(|e| Error::Internal(format!("System time error: {}", e)))?;

		let timestamp = format_timestamp(now.as_secs());
		let date = &timestamp[..8];

		// Parse URL
		let parsed_url = parse_url(url)?;

		// Create canonical headers
		let mut signed_headers = headers.clone();
		signed_headers.insert("host".to_string(), parsed_url.host.clone());
		signed_headers.insert("x-amz-date".to_string(), timestamp.clone());

		if let Some(ref token) = self.credentials.session_token {
			signed_headers.insert("x-amz-security-token".to_string(), token.clone());
		}

		// Calculate payload hash
		let payload_hash = sha256_hex(payload);
		signed_headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());

		// Create canonical request
		let canonical_request = self.create_canonical_request(method, &parsed_url, &signed_headers, &payload_hash);

		// Create string to sign
		let credential_scope = format!("{}/{}/{}/aws4_request", date, self.credentials.region, self.service);
		let string_to_sign = format!(
			"AWS4-HMAC-SHA256\n{}\n{}\n{}",
			timestamp,
			credential_scope,
			sha256_hex(canonical_request.as_bytes())
		);

		// Calculate signature
		let signing_key = self.derive_signing_key(date);
		let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes());

		// Build Authorization header
		let signed_header_names: Vec<&str> = signed_headers.keys().map(|s| s.as_str()).collect();
		let authorization = format!(
			"AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
			self.credentials.access_key_id,
			credential_scope,
			signed_header_names.join(";"),
			signature
		);

		// Return all headers needed for the request
		let mut result_headers = signed_headers;
		result_headers.insert("authorization".to_string(), authorization);

		Ok(result_headers)
	}

	fn create_canonical_request(
		&self,
		method: &str,
		parsed_url: &ParsedUrl,
		headers: &BTreeMap<String, String>,
		payload_hash: &str,
	) -> String {
		// Canonical URI
		let canonical_uri = if parsed_url.path.is_empty() {
			"/".to_string()
		} else {
			uri_encode(&parsed_url.path, false)
		};

		// Canonical query string (empty for POST requests typically)
		let canonical_query = parsed_url
			.query
			.as_ref()
			.map(|q| {
				let mut params: Vec<(String, String)> = q
					.split('&')
					.filter_map(|p| {
						let mut parts = p.splitn(2, '=');
						let key = parts.next()?;
						let value = parts.next().unwrap_or("");
						Some((uri_encode(key, true), uri_encode(value, true)))
					})
					.collect();
				params.sort();
				params
					.into_iter()
					.map(|(k, v)| format!("{}={}", k, v))
					.collect::<Vec<_>>()
					.join("&")
			})
			.unwrap_or_default();

		// Canonical headers
		let canonical_headers: String = headers
			.iter()
			.map(|(k, v)| format!("{}:{}\n", k.to_lowercase(), v.trim()))
			.collect();

		// Signed headers
		let signed_headers: String = headers.keys().map(|k| k.to_lowercase()).collect::<Vec<_>>().join(";");

		format!(
			"{}\n{}\n{}\n{}\n{}\n{}",
			method, canonical_uri, canonical_query, canonical_headers, signed_headers, payload_hash
		)
	}

	fn derive_signing_key(&self, date: &str) -> Vec<u8> {
		let k_secret = format!("AWS4{}", self.credentials.secret_access_key);
		let k_date = hmac_sha256(k_secret.as_bytes(), date.as_bytes());
		let k_region = hmac_sha256(&k_date, self.credentials.region.as_bytes());
		let k_service = hmac_sha256(&k_region, self.service.as_bytes());
		hmac_sha256(&k_service, b"aws4_request")
	}
}

// region:    --- URL Parsing

struct ParsedUrl {
	host: String,
	path: String,
	query: Option<String>,
}

fn parse_url(url: &str) -> Result<ParsedUrl> {
	// Simple URL parsing
	let url = url
		.strip_prefix("https://")
		.or_else(|| url.strip_prefix("http://"))
		.unwrap_or(url);

	let (host_and_path, query) = if let Some(idx) = url.find('?') {
		(&url[..idx], Some(url[idx + 1..].to_string()))
	} else {
		(url, None)
	};

	let (host, path) = if let Some(idx) = host_and_path.find('/') {
		(&host_and_path[..idx], host_and_path[idx..].to_string())
	} else {
		(host_and_path, "/".to_string())
	};

	Ok(ParsedUrl {
		host: host.to_string(),
		path,
		query,
	})
}

// endregion: --- URL Parsing

// region:    --- Crypto Helpers (using pure Rust implementations)

/// SHA-256 hash implementation
fn sha256(data: &[u8]) -> [u8; 32] {
	// SHA-256 constants
	const K: [u32; 64] = [
		0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
		0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
		0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
		0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
		0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
		0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
		0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
		0xc67178f2,
	];

	// Initial hash values
	let mut h: [u32; 8] = [
		0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
	];

	// Pre-processing: adding padding bits
	let ml = (data.len() as u64) * 8;
	let mut padded = data.to_vec();
	padded.push(0x80);
	while (padded.len() % 64) != 56 {
		padded.push(0);
	}
	padded.extend_from_slice(&ml.to_be_bytes());

	// Process each 512-bit chunk
	for chunk in padded.chunks(64) {
		let mut w = [0u32; 64];

		// Break chunk into sixteen 32-bit big-endian words
		for (i, word) in chunk.chunks(4).enumerate().take(16) {
			w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
		}

		// Extend the sixteen 32-bit words into sixty-four 32-bit words
		for i in 16..64 {
			let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
			let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
			w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
		}

		// Initialize working variables
		let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;

		// Compression function main loop
		for i in 0..64 {
			let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
			let ch = (e & f) ^ ((!e) & g);
			let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
			let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
			let maj = (a & b) ^ (a & c) ^ (b & c);
			let temp2 = s0.wrapping_add(maj);

			hh = g;
			g = f;
			f = e;
			e = d.wrapping_add(temp1);
			d = c;
			c = b;
			b = a;
			a = temp1.wrapping_add(temp2);
		}

		// Add the compressed chunk to the current hash value
		h[0] = h[0].wrapping_add(a);
		h[1] = h[1].wrapping_add(b);
		h[2] = h[2].wrapping_add(c);
		h[3] = h[3].wrapping_add(d);
		h[4] = h[4].wrapping_add(e);
		h[5] = h[5].wrapping_add(f);
		h[6] = h[6].wrapping_add(g);
		h[7] = h[7].wrapping_add(hh);
	}

	// Produce the final hash value (big-endian)
	let mut result = [0u8; 32];
	for (i, &val) in h.iter().enumerate() {
		result[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
	}
	result
}

fn sha256_hex(data: &[u8]) -> String {
	let hash = sha256(data);
	hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// HMAC-SHA256 implementation
fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
	const BLOCK_SIZE: usize = 64;

	// If key is longer than block size, hash it
	let key = if key.len() > BLOCK_SIZE {
		sha256(key).to_vec()
	} else {
		key.to_vec()
	};

	// Pad key to block size
	let mut padded_key = key.clone();
	padded_key.resize(BLOCK_SIZE, 0);

	// Create inner and outer padded keys
	let mut i_key_pad = vec![0x36u8; BLOCK_SIZE];
	let mut o_key_pad = vec![0x5cu8; BLOCK_SIZE];

	for i in 0..BLOCK_SIZE {
		i_key_pad[i] ^= padded_key[i];
		o_key_pad[i] ^= padded_key[i];
	}

	// Inner hash
	let mut inner = i_key_pad;
	inner.extend_from_slice(data);
	let inner_hash = sha256(&inner);

	// Outer hash
	let mut outer = o_key_pad;
	outer.extend_from_slice(&inner_hash);
	sha256(&outer).to_vec()
}

fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
	let hash = hmac_sha256(key, data);
	hash.iter().map(|b| format!("{:02x}", b)).collect()
}

// endregion: --- Crypto Helpers

// region:    --- Encoding Helpers

fn format_timestamp(unix_secs: u64) -> String {
	// Convert Unix timestamp to ISO8601 format: YYYYMMDD'T'HHMMSS'Z'
	let secs_per_day = 86400;
	let secs_per_hour = 3600;
	let secs_per_minute = 60;

	// Days since Unix epoch
	let days = unix_secs / secs_per_day;
	let remaining_secs = unix_secs % secs_per_day;

	// Calculate year, month, day
	let (year, month, day) = days_to_ymd(days);

	// Calculate hours, minutes, seconds
	let hours = remaining_secs / secs_per_hour;
	let remaining = remaining_secs % secs_per_hour;
	let minutes = remaining / secs_per_minute;
	let seconds = remaining % secs_per_minute;

	format!(
		"{:04}{:02}{:02}T{:02}{:02}{:02}Z",
		year, month, day, hours, minutes, seconds
	)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
	// Simplified calculation from days since 1970-01-01
	let mut remaining_days = days as i64;
	let mut year = 1970i64;

	// Find year
	loop {
		let days_in_year = if is_leap_year(year) { 366 } else { 365 };
		if remaining_days < days_in_year {
			break;
		}
		remaining_days -= days_in_year;
		year += 1;
	}

	// Find month and day
	let days_in_months: [i64; 12] = if is_leap_year(year) {
		[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
	} else {
		[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
	};

	let mut month = 1;
	for &days in &days_in_months {
		if remaining_days < days {
			break;
		}
		remaining_days -= days;
		month += 1;
	}

	(year as u64, month, (remaining_days + 1) as u64)
}

fn is_leap_year(year: i64) -> bool {
	(year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn uri_encode(s: &str, encode_slash: bool) -> String {
	let mut result = String::new();
	for byte in s.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
				result.push(byte as char);
			}
			b'/' if !encode_slash => {
				result.push('/');
			}
			_ => {
				result.push_str(&format!("%{:02X}", byte));
			}
		}
	}
	result
}

// endregion: --- Encoding Helpers

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sha256_empty() {
		let hash = sha256_hex(b"");
		assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
	}

	#[test]
	fn test_sha256_hello() {
		let hash = sha256_hex(b"hello");
		assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
	}

	#[test]
	fn test_hmac_sha256() {
		let hash = hmac_sha256_hex(b"key", b"The quick brown fox jumps over the lazy dog");
		assert_eq!(hash, "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8");
	}

	#[test]
	fn test_format_timestamp() {
		// 2024-01-15 12:30:45 UTC
		let ts = format_timestamp(1705321845);
		assert_eq!(ts, "20240115T123045Z");
	}

	#[test]
	fn test_uri_encode() {
		assert_eq!(uri_encode("hello world", true), "hello%20world");
		assert_eq!(uri_encode("path/to/resource", false), "path/to/resource");
		assert_eq!(uri_encode("path/to/resource", true), "path%2Fto%2Fresource");
	}
}
