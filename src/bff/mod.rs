//! Local Backend-For-Frontend (BFF) proxy module.
//!
//! Provides a secure boundary for browser-internal local BFF functionality,
//! ensuring that provider credentials are never returned or leaked to the web page/caller.

use std::collections::HashMap;

/// Opaque page-supplied payload representing an AI request.
/// Contains no credentials or sensitive material.
#[derive(Debug, Clone, PartialEq)]
pub struct AiRequest {
    /// The target AI provider (e.g., "anthropic").
    pub provider: String,
    /// The API path of the provider's endpoint.
    pub path: String,
    /// The raw request body payload supplied by the page.
    pub body: Vec<u8>,
}

/// External AI response returned to the caller.
/// Contains no sensitive credentials or material.
#[derive(Debug, Clone, PartialEq)]
pub struct AiResponse {
    /// The HTTP status code returned by the external service.
    pub status: u16,
    /// The raw response body payload returned by the external service.
    pub body: Vec<u8>,
}

/// Errors returned by the local BFF.
///
/// Guaranteed never to place or leak credential strings in any of its variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BffError {
    /// The requested provider is unknown or not supported by this proxy.
    UnknownProvider(String),
    /// The requested provider is known, but no credential exists in the store.
    MissingCredential(String),
    /// The provider and credential are valid, but egress is not implemented in this scaffold.
    NotImplemented,
}

/// Abstracts the storage where credentials live.
pub trait SecretStore {
    /// Retrieves the credential for a given provider.
    ///
    /// // TODO(spec): real secure-store backend (OS keychain) — not in this scaffold
    fn credential(&self, provider: &str) -> Option<String>;
}

/// An in-memory implementation of [`SecretStore`] holding a map of provider to credential.
///
/// # Security Warning
///
/// **LOUD WARNING**: This struct is for tests and scaffolding ONLY and must NOT
/// be used to hold production secrets in memory permanently or in production builds.
#[derive(Debug, Default, Clone)]
pub struct InMemorySecretStore {
    credentials: HashMap<String, String>,
}

impl InMemorySecretStore {
    /// Creates a new, empty `InMemorySecretStore`.
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }

    /// Inserts a credential for a provider into the in-memory store.
    ///
    /// # Security Warning
    ///
    /// For test/scaffolding usage only.
    pub fn insert(&mut self, provider: String, credential: String) {
        self.credentials.insert(provider, credential);
    }
}

impl SecretStore for InMemorySecretStore {
    fn credential(&self, provider: &str) -> Option<String> {
        self.credentials.get(provider).cloned()
    }
}

/// Allowed provider IDs list.
const ALLOWED_PROVIDERS: &[&str] = &["anthropic", "openai", "gemini"];

/// Local Backend-For-Frontend (BFF) proxy.
///
/// Mediates requests to external AI services, injecting credentials securely
/// at the process boundary so the calling web page never receives secret material.
#[derive(Debug, Clone)]
pub struct LocalBff<S: SecretStore> {
    store: S,
}

impl<S: SecretStore> LocalBff<S> {
    /// Creates a new `LocalBff` with the provided secret store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Forwards an AI request to the external service after validating the provider
    /// and injecting the appropriate credential.
    ///
    /// # Errors
    ///
    /// - [`BffError::UnknownProvider`] if the provider is not in the allowed list.
    /// - [`BffError::MissingCredential`] if the provider is allowed but no credential is found.
    /// - [`BffError::NotImplemented`] if the credential is found (real egress is currently deferred).
    pub fn forward(&self, req: &AiRequest) -> Result<AiResponse, BffError> {
        // 1. Determine whether req.provider is a known/allowed provider.
        if !ALLOWED_PROVIDERS.iter().any(|&p| p == req.provider) {
            return Err(BffError::UnknownProvider(req.provider.clone()));
        }

        // 2. Look up the credential via self.store.credential(&req.provider).
        let _credential = match self.store.credential(&req.provider) {
            Some(cred) => cred,
            None => return Err(BffError::MissingCredential(req.provider.clone())),
        };

        // 3. Egress is not wired in this scaffold.
        // TODO(spec): inject credential into outbound request and perform egress to the external AI service
        Err(BffError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_provider() {
        let store = InMemorySecretStore::new();
        let bff = LocalBff::new(store);
        let req = AiRequest {
            provider: "invalid-provider".to_string(),
            path: "/v1/chat/completions".to_string(),
            body: buff_body(),
        };

        let res = bff.forward(&req);
        assert_eq!(
            res,
            Err(BffError::UnknownProvider("invalid-provider".to_string()))
        );
    }

    #[test]
    fn test_missing_credential() {
        let store = InMemorySecretStore::new();
        let bff = LocalBff::new(store);
        let req = AiRequest {
            provider: "anthropic".to_string(),
            path: "/v1/messages".to_string(),
            body: buff_body(),
        };

        let res = bff.forward(&req);
        assert_eq!(
            res,
            Err(BffError::MissingCredential("anthropic".to_string()))
        );
    }

    #[test]
    fn test_known_provider_with_credential() {
        let mut store = InMemorySecretStore::new();
        store.insert("anthropic".to_string(), "SECRET-XYZ".to_string());
        let bff = LocalBff::new(store);
        let req = AiRequest {
            provider: "anthropic".to_string(),
            path: "/v1/messages".to_string(),
            body: buff_body(),
        };

        let res = bff.forward(&req);
        assert_eq!(res, Err(BffError::NotImplemented));
    }

    #[test]
    fn test_security_boundary_no_leak() {
        let mut store = InMemorySecretStore::new();
        let secret = "SUPER-SECRET-XYZ-KEY-DO-NOT-LEAK";
        store.insert("anthropic".to_string(), secret.to_string());
        let bff = LocalBff::new(store);
        let req = AiRequest {
            provider: "anthropic".to_string(),
            path: "/v1/messages".to_string(),
            body: buff_body(),
        };

        let res = bff.forward(&req);
        // Ensure result is Err(BffError::NotImplemented)
        assert_eq!(res, Err(BffError::NotImplemented));

        // Format the error/result as Debug/Display and verify it doesn't leak the secret
        let debug_str = format!("{:?}", res);
        assert!(
            !debug_str.contains(secret),
            "Security Boundary Violation: Secret leaked in Debug representation of Result/Error!"
        );
    }

    fn buff_body() -> Vec<u8> {
        b"{\"prompt\": \"hello\"}".to_vec()
    }
}
