# Research: Vertex AI Support for rust-genai

## Executive Summary

### Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| **Energizing?** | ✅ YES | Vertex AI is Google's enterprise AI platform with strong enterprise adoption. Supporting it enables rust-genai in GCP environments. |
| **Leverages strengths?** | ✅ YES | rust-genai already has Gemini adapter. Vertex AI uses similar API structure but with enterprise features (IAM, VPC, audit logging). |
| **Meets real need?** | ✅ YES | Enterprise users need Vertex AI support for: IAM integration, VPC-SC, audit logging, committed use discounts. Current Gemini adapter only supports consumer API. |

**Verdict:** PROCEED with implementation.

---

## Problem Statement

### Description
rust-genai currently supports Google Gemini via the consumer API (`generativelanguage.googleapis.com`). However, enterprise users on Google Cloud Platform need Vertex AI support which provides:
- IAM-based authentication (not API keys)
- VPC Service Controls for network isolation
- Cloud Audit Logs for compliance
- Committed use discounts for cost optimization
- Enterprise support and SLAs

### Impact
- **Blocked enterprise adoption:** Companies using VPC-SC cannot use consumer API
- **Compliance gaps:** No audit logging for regulated industries
- **Authentication complexity:** Service accounts vs simple API keys
- **Missing features:** Vertex AI has different model availability and features

### Success Criteria
| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Authentication | API key only | IAM + API key | Test both auth methods |
| Endpoint coverage | Consumer API | Vertex AI | All service types work |
| Feature parity | Basic | Full | Embeddings, streaming, etc. |
| Documentation | None | Complete | Usage examples |

---

## Current State Analysis

### What Exists
1. **Gemini adapter** - Uses `generativelanguage.googleapis.com`
2. **API key auth** - Via `GEMINI_API_KEY` environment variable
3. **Model support** - Gemini 1.5 Flash, Pro, etc.
4. **Service types** - Chat, streaming, embeddings

### What's Missing
1. **Vertex AI endpoint** - `https://LOCATION-aiplatform.googleapis.com/v1/...`
2. **IAM authentication** - Google Cloud service accounts, OAuth2
3. **Project/location context** - Vertex AI requires project ID and region
4. **Different URL structure** - Resource-based vs model-based URLs
5. **Enterprise features** - VPC-SC, audit logs (implicit via endpoint)

---

## Constraints

### Technical (Max 3 Vital)
1. **Maintain backward compatibility** - Existing Gemini adapter must continue working
2. **Minimal code duplication** - Share code between Gemini and Vertex AI adapters
3. **Async OAuth2 flow** - Must handle token refresh without blocking

### Business
1. **Apache 2.0 license** - Compatible with rust-genai
2. **No breaking changes** - Semantic versioning compliance
3. **Google Cloud partnership** - Follow best practices for GCP integrations

### Non-Functional
1. **Latency** - Vertex AI typically has higher latency than consumer API (acceptable trade-off)
2. **Regional availability** - Must support all Vertex AI regions
3. **Documentation** - Must include GCP setup instructions

---

## Essentialism

### Vital Few (Max 3)
1. **Vertex AI chat completion** - Core functionality, highest demand
2. **IAM authentication** - Required for enterprise use
3. **Project/location configuration** - Required for all Vertex AI calls

### Eliminated from Scope
- ❌ Vertex AI Model Garden (custom models) - Complex, lower priority
- ❌ Vertex AI Vector Search - Out of scope for genai
- ❌ Vertex AI Pipelines - Too complex, different use case
- ❌ Fine-tuning via Vertex AI - Not core functionality

---

## Dependencies

### Internal
- Existing Gemini adapter (code reuse target)
- Adapter trait system (must implement)
- Auth resolver (needs extension for OAuth2)

### External
- `gcp-auth` or similar crate for OAuth2
- Google Cloud SDK (for local development)
- Service account credentials (user-provided)

---

## Risks and Unknowns

### Known Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| OAuth2 complexity | High | Use established crate, good documentation |
| Different API responses | Medium | Thorough testing, response mapping |
| Regional differences | Low | Test in multiple regions |

### Open Questions
1. Should we reuse Gemini adapter or create new VertexAdapter? (Design decision)
2. How to handle Application Default Credentials (ADC)? (Research needed)
3. What's the exact URL format for all service types? (Testing needed)

### Assumptions
- Vertex AI API is similar enough to Gemini API for code reuse
- OAuth2 tokens can be cached and refreshed
- Users have Google Cloud project set up

---

## Research Findings

### Vertex AI vs Gemini API Differences

| Aspect | Gemini API | Vertex AI |
|--------|-----------|-----------|
| **Endpoint** | `generativelanguage.googleapis.com` | `LOCATION-aiplatform.googleapis.com` |
| **Auth** | API key | OAuth2 / Service account |
| **URL format** | `/models/MODEL:method` | `/projects/PROJECT/locations/LOCATION/publishers/google/models/MODEL:method` |
| **Features** | Consumer features | Enterprise features (IAM, VPC, audit) |
| **Pricing** | Pay-as-you-go | Committed use discounts |

### Prior Art
- **Google's Rust SDK** - `google-cloud-rust` exists but is alpha
- **gcp-auth crate** - Handles OAuth2 for GCP
- **Vertex AI Python SDK** - Reference implementation

### Key Insights
1. **URL structure is the main difference** - Same request/response format
2. **Auth is more complex** - But well-documented GCP patterns
3. **Regional endpoints** - Must include location in URL

---

## Recommendations

### Proceed/No-Proceed
✅ **PROCEED** - Clear enterprise need, builds on existing Gemini support, manageable complexity.

### Scope
**Phase 1:** Vertex AI chat + streaming (MVP)
**Phase 2:** Embeddings + advanced features
**Phase 3:** Documentation + examples

### Risk Mitigation
1. Create new `vertex` adapter (don't modify Gemini)
2. Extensive testing with real Vertex AI project
3. Clear documentation on GCP setup

---

## Next Steps

1. Create Design Document (Phase 2)
2. Set up test Vertex AI project
3. Implement VertexAdapter
4. Test all service types
5. Documentation and examples
