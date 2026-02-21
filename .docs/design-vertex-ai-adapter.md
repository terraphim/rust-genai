# Design: Vertex AI Adapter for rust-genai

## Overview

### Summary
Implement a new Vertex AI adapter for rust-genai that enables enterprise Google Cloud users to access Gemini models through Vertex AI, with IAM-based authentication, regional endpoints, and full compatibility with existing rust-genai APIs.

### Approach
Create new `vertex` adapter that follows the same patterns as existing adapters (gemini, anthropic, etc.) but with Vertex AI-specific authentication and URL structure.

### Scope

**IN Scope:**
- Vertex AI chat completion
- Vertex AI streaming chat
- Vertex AI embeddings
- IAM/OAuth2 authentication
- Regional endpoint support
- Project/location configuration

**OUT of Scope:**
- Vertex AI Model Garden custom models
- Vertex AI Vector Search
- Vertex AI Pipelines
- Fine-tuning operations

**Avoid At All Cost:**
- Breaking changes to existing adapters
- Code duplication with Gemini adapter
- Blocking authentication flows

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    rust-genai Client                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │   Client     │───▶│   Adapter    │───▶│   Vertex     │     │
│  │   Config     │    │   Resolver   │    │   Adapter    │     │
│  │              │    │              │    │              │     │
│  └──────────────┘    └──────────────┘    └──────────────┘     │
│         │                   │                   │               │
│         │                   │                   │               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │   Auth       │◀───│   Service    │◀───│   GCP        │     │
│  │   Resolver   │    │   Target     │    │   OAuth2     │     │
│  │              │    │              │    │              │     │
│  └──────────────┘    └──────────────┘    └──────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Vertex AI API   │
                    │  (Google Cloud)  │
                    └──────────────────┘
```

### Data Flow

```
1. User creates client with Vertex AI config
   ClientConfig::new(AdapterKind::Vertex)
       .with_auth(AuthData::from_env("GOOGLE_APPLICATION_CREDENTIALS"))
       .with_options(|opts| {
           opts.vertex_project_id = "my-project";
           opts.vertex_location = "us-central1";
       })

2. Client resolves adapter
   AdapterKind::Vertex → VertexAdapter

3. Adapter builds request
   - Get OAuth2 token from GCP auth
   - Construct Vertex AI URL:
     https://us-central1-aiplatform.googleapis.com/v1/
     projects/my-project/locations/us-central1/
     publishers/google/models/gemini-1.5-flash-001:generateContent
   - Build request payload (same as Gemini)

4. Send request with OAuth2 bearer token

5. Parse response (same format as Gemini)
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| New `vertex` adapter vs extending `gemini` | Clear separation, different auth, different URLs | Single adapter with mode switch (complex) |
| OAuth2 token caching | Performance, avoid token refresh on every call | Request new token each time (slow) |
| Environment variable for credentials | Standard GCP pattern | Config file only (less flexible) |
| Project/location in options | Required for all Vertex AI calls | Hardcode (not flexible) |

---

## Simplicity Check

**"What if this could be easy?"**

- Start with **chat only** - add streaming/embeddings later
- Use **existing GCP auth patterns** - don't reinvent
- **Copy Gemini adapter** as starting point - modify URLs and auth
- **Single region first** - add multi-region later

**Simplified MVP:**
1. Copy gemini adapter structure
2. Change URL construction
3. Add OAuth2 auth
4. Test with us-central1

---

## File Changes

### New Files
```
src/
└── adapter/
    └── adapters/
        └── vertex/
            ├── mod.rs
            ├── adapter_impl.rs
            ├── auth.rs          # OAuth2 / Service account
            └── streamer.rs      # Streaming support
```

### Modified Files
```
src/
├── adapter/
│   ├── adapter_kind.rs      # Add Vertex variant
│   ├── adapters/
│   │   └── mod.rs           # Register vertex module
│   └── mod.rs               # Export Vertex types
├── client/
│   └── config.rs            # Add vertex_project_id, vertex_location options
└── resolver/
    └── auth_resolver.rs     # Handle GCP auth data
```

---

## API Design

### Vertex Adapter

```rust
// src/adapter/adapters/vertex/adapter_impl.rs

pub struct VertexAdapter;

impl VertexAdapter {
    pub const AUTH_DEFAULT_ENV_NAME: &str = "GOOGLE_APPLICATION_CREDENTIALS";
    
    /// Default endpoint for Vertex AI (region-specific)
    fn default_endpoint(location: &str) -> Endpoint {
        format!("https://{}-aiplatform.googleapis.com/v1/", location)
    }
}

impl Adapter for VertexAdapter {
    fn default_endpoint() -> Endpoint {
        // Note: This is a placeholder, actual endpoint needs location
        Endpoint::from_static("https://us-central1-aiplatform.googleapis.com/v1/")
    }

    fn default_auth() -> AuthData {
        AuthData::from_env(Self::AUTH_DEFAULT_ENV_NAME)
    }

    fn get_service_url(
        model: &ModelIden, 
        service_type: ServiceType, 
        endpoint: Endpoint,
        options: &AdapterOptions,  // Contains project_id, location
    ) -> Result<String> {
        let base_url = endpoint.base_url();
        let (_, model_name) = model.model_name.namespace_and_name();
        let project_id = options.vertex_project_id.as_ref()
            .ok_or_else(|| Error::MissingVertexProjectId)?;
        let location = options.vertex_location.as_ref()
            .unwrap_or(&"us-central1".to_string());
        
        let url = match service_type {
            ServiceType::Chat => format!(
                "{base_url}projects/{project_id}/locations/{location}/\
                 publishers/google/models/{model_name}:generateContent"
            ),
            ServiceType::ChatStream => format!(
                "{base_url}projects/{project_id}/locations/{location}/\
                 publishers/google/models/{model_name}:streamGenerateContent"
            ),
            ServiceType::Embed => format!(
                "{base_url}projects/{project_id}/locations/{location}/\
                 publishers/google/models/{model_name}:embedContent"
            ),
        };
        Ok(url)
    }

    fn to_web_request_data(
        target: ServiceTarget,
        service_type: ServiceType,
        chat_req: ChatRequest,
        options_set: ChatOptionsSet<'_, '_>,
    ) -> Result<WebRequestData> {
        // Similar to Gemini, but with OAuth2 token
        // ...
    }
}
```

### Configuration Options

```rust
// src/client/config.rs

pub struct ClientConfig {
    // ... existing fields ...
    
    /// Google Cloud Project ID for Vertex AI
    pub vertex_project_id: Option<String>,
    
    /// Vertex AI location (e.g., "us-central1", "europe-west4")
    pub vertex_location: Option<String>,
}

impl ClientConfig {
    pub fn with_vertex_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.vertex_project_id = Some(project_id.into());
        self
    }
    
    pub fn with_vertex_location(mut self, location: impl Into<String>) -> Self {
        self.vertex_location = Some(location.into());
        self
    }
}
```

### Usage Example

```rust
use genai::Client;
use genai::adapter::AdapterKind;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with Vertex AI
    let client = Client::builder()
        .with_adapter_kind(AdapterKind::Vertex)
        .with_auth_from_env("GOOGLE_APPLICATION_CREDENTIALS")
        .with_vertex_project_id("my-gcp-project")
        .with_vertex_location("us-central1")
        .build();

    // Use like any other adapter
    let chat_res = client
        .chat("gemini-1.5-flash-001")
        .system("You are a helpful assistant")
        .user("Explain how AI works")
        .send()
        .await?;

    println!("{}", chat_res.content.text_as_str().unwrap_or("N/A"));
    
    Ok(())
}
```

---

## Authentication

### OAuth2 Flow

```rust
// src/adapter/adapters/vertex/auth.rs

use gcp_auth::TokenProvider;

pub struct VertexAuth {
    token_provider: Box<dyn TokenProvider>,
    cached_token: Option<(String, Instant)>,
}

impl VertexAuth {
    pub async fn new(credentials_path: &str) -> Result<Self> {
        let token_provider = gcp_auth::from_file(credentials_path).await?;
        Ok(Self {
            token_provider: Box::new(token_provider),
            cached_token: None,
        })
    }
    
    pub async fn get_token(&mut self) -> Result<String> {
        // Check if cached token is still valid (with 5 min buffer)
        if let Some((token, expiry)) = &self.cached_token {
            if expiry.duration_since(Instant::now()) > Duration::from_secs(300) {
                return Ok(token.clone());
            }
        }
        
        // Fetch new token
        let token = self.token_provider.token().await?;
        let expiry = Instant::now() + Duration::from_secs(token.expires_in() as u64);
        let token_string = token.as_str().to_string();
        
        self.cached_token = Some((token_string.clone(), expiry));
        Ok(token_string)
    }
}
```

---

## Test Strategy

### Unit Tests
- URL construction for all service types
- OAuth2 token caching
- Error handling for missing project/location

### Integration Tests
- Real Vertex AI project (test account)
- All service types: chat, streaming, embeddings
- Multiple regions: us-central1, europe-west4, asia-northeast1

### Manual Validation
- Service account authentication
- Application Default Credentials (ADC)
- VPC-SC compliance (if test environment available)

---

## Implementation Steps

### Phase 1: Foundation (Week 1)
1. **Create vertex adapter module**
   - File: `src/adapter/adapters/vertex/mod.rs`
   - Export public types

2. **Add Vertex to AdapterKind enum**
   - File: `src/adapter/adapter_kind.rs`
   - Add `Vertex` variant

3. **Implement basic VertexAdapter**
   - File: `src/adapter/adapters/vertex/adapter_impl.rs`
   - Copy from Gemini, modify URLs
   - Hardcode project/location for now

4. **Add OAuth2 authentication**
   - File: `src/adapter/adapters/vertex/auth.rs`
   - Use `gcp-auth` crate
   - Token caching

### Phase 2: Configuration (Week 1-2)
5. **Add Vertex options to ClientConfig**
   - File: `src/client/config.rs`
   - `vertex_project_id`, `vertex_location`

6. **Update URL construction**
   - Use options in `get_service_url()`
   - Error if project_id missing

7. **Register adapter**
   - File: `src/adapter/adapters/mod.rs`
   - Add vertex module

### Phase 3: Testing (Week 2)
8. **Unit tests**
   - URL construction
   - Auth token caching
   - Error handling

9. **Integration tests**
   - Real Vertex AI calls
   - All service types

10. **Documentation**
    - Usage examples
    - GCP setup instructions

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Chat completion | Works | Integration test |
| Streaming | Works | Integration test |
| Embeddings | Works | Integration test |
| Auth token refresh | Automatic | Unit test |
| Latency | <2s | Benchmark |
| Test coverage | >80% | cargo tarpaulin |

---

## Rollback Plan

If implementation fails:
1. **Feature flag** - Disable vertex adapter via compile flag
2. **Version pinning** - Users can pin to previous version
3. **Documentation** - Clear migration guide

---

## Open Items

| Item | Decision Needed | By When |
|------|-----------------|---------|
| gcp-auth crate | Use `gcp-auth` vs custom implementation | Week 1 |
| Default region | us-central1 vs require explicit | Week 1 |
| Model names | Same as Gemini or Vertex-specific | Week 1 |

---

## References

- [Vertex AI API Reference](https://cloud.google.com/vertex-ai/docs/reference/rest)
- [Gemini on Vertex AI](https://cloud.google.com/vertex-ai/generative-ai/docs/model-reference/gemini)
- [GCP Authentication](https://cloud.google.com/docs/authentication)
- [gcp-auth crate](https://docs.rs/gcp-auth)
