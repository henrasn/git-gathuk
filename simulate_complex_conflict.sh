#!/usr/bin/env bash
# ==============================================================================
# simulate_complex_conflict.sh
# Generates a realistic Git merge conflict across multiple files with:
#  - Multiple conflict hunks per file
#  - Long horizontal lines (>120 chars) to test horizontal panning (h / l / ← / →)
#  - Vertical alignment between Local, Merged, and Remote panes
#  - Continuous Unicode borders (┌───┬───, ├───┼───, └───┴───)
# ==============================================================================

set -euo pipefail

TARGET_DIR="${1:-/tmp/git_gathuk_simulation}"
echo "==> Setting up complex Git merge simulation at: $TARGET_DIR"

rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"
cd "$TARGET_DIR"

git init -b main
git config user.name "Merge Tester"
git config user.email "tester@example.com"

# ------------------------------------------------------------------------------
# 1. Base Commit (common ancestor)
# ------------------------------------------------------------------------------
mkdir -p src

cat << 'EOF' > src/session_manager.rs
// Base Session Manager - Common Ancestor
pub struct UserSession {
    pub session_id: String,
    pub user_uuid: String,
    pub created_at_timestamp: u64,
}

impl UserSession {
    pub fn new(user_uuid: &str) -> Self {
        Self {
            session_id: format!("sess_{}", user_uuid),
            user_uuid: user_uuid.to_string(),
            created_at_timestamp: 1700000000,
        }
    }

    // Unchanged shared helper function
    pub fn is_active(&self) -> bool {
        self.created_at_timestamp > 0
    }

    pub fn validate_and_extract_claims(&self) -> bool {
        !self.session_id.is_empty()
    }
}
EOF

cat << 'EOF' > src/routes.rs
// Base HTTP Service Routes
pub fn configure_app_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::resource("/health").to(|| async { "OK" }));
    cfg.service(actix_web::web::resource("/metrics").to(|| async { "METRICS" }));
}
EOF

cat << 'EOF' > Cargo.toml
[package]
name = "demo-backend-service"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
EOF

git add .
git commit -m "Initial base commit: session manager, routes, and Cargo.toml"

# ------------------------------------------------------------------------------
# 2. Branch: feature/oauth-v2 (Incoming Remote)
# ------------------------------------------------------------------------------
git checkout -b feature/oauth-v2

cat << 'EOF' > src/session_manager.rs
// Base Session Manager - Common Ancestor
pub struct UserSession {
    pub session_id: String,
    pub user_uuid: String,
    pub created_at_timestamp: u64,
    // [Remote Incoming Hunk 1] Long field definition with extended OAuth2 metadata annotations
    pub oauth2_provider_issuer_metadata_url: Option<String>,
    pub remote_idp_jwks_keyset_cache_ttl_seconds: u32,
}

impl UserSession {
    // [Remote Incoming Hunk 1 Method] Constructor taking OAuth2 credentials with extended signature
    pub fn new(user_uuid: &str, provider_url: Option<&str>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self {
            session_id: format!("oauth2_sess_{}_{}", user_uuid, "incoming_token_with_lengthy_hex_digest_9876543210abcdef"),
            user_uuid: user_uuid.to_string(),
            created_at_timestamp: 1700000000,
            oauth2_provider_issuer_metadata_url: provider_url.map(|u| u.to_string()),
            remote_idp_jwks_keyset_cache_ttl_seconds: 3600,
        }
    }

    // Unchanged shared helper function
    pub fn is_active(&self) -> bool {
        self.created_at_timestamp > 0
    }

    // [Remote Incoming Hunk 2] Extended claim validation with comprehensive remote endpoint URL
    pub fn validate_and_extract_claims(&self) -> Result<serde_json::Value, String> {
        let claims_endpoint_uri = "https://identity.enterprise-service.internal.corp:8443/oauth/v2/jwks/verify?strict=true&audit_log=enabled&region=us-east-2";
        if self.session_id.starts_with("oauth2_sess_") {
            Ok(serde_json::json!({ "status": "authenticated", "provider": self.oauth2_provider_issuer_metadata_url, "endpoint": claims_endpoint_uri }))
        } else {
            Err("Invalid provider claims detected during OAuth2 validation sequence".to_string())
        }
    }
}
EOF

cat << 'EOF' > src/routes.rs
// Base HTTP Service Routes
pub fn configure_app_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::resource("/health").to(|| async { "OK" }));
    // [Remote Incoming Hunk 1] New OAuth2 callback endpoint with query parser middleware
    cfg.service(actix_web::web::resource("/api/v2/auth/callback/enterprise-sso").to(|| async { "OAUTH2_ENTERPRISE_SSO_HANDLER_V2_ACTIVE" }));
    cfg.service(actix_web::web::resource("/metrics").to(|| async { "METRICS" }));
}
EOF

cat << 'EOF' > Cargo.toml
[package]
name = "demo-backend-service"
version = "0.2.0"
edition = "2021"

[dependencies]
# [Remote Incoming Dependencies with long feature lists]
serde = { version = "1.0.197", features = ["derive", "alloc", "rc", "serde_derive"] }
tokio = { version = "1.35", features = ["full", "tracing", "signal", "parking_lot"] }
uuid = { version = "1.6", features = ["v4", "fast-rng", "serde"] }
chrono = { version = "0.4.31", features = ["serde", "clock", "std"] }
EOF

git commit -am "Feature: implement oauth2 enterprise authentication with long endpoint URLs"

# ------------------------------------------------------------------------------
# 3. Branch: main (Local HEAD)
# ------------------------------------------------------------------------------
git checkout main

cat << 'EOF' > src/session_manager.rs
// Base Session Manager - Common Ancestor
pub struct UserSession {
    pub session_id: String,
    pub user_uuid: String,
    pub created_at_timestamp: u64,
    // [Local HEAD Hunk 1] Local HMAC hardware security module keystore buffer
    pub local_jwt_hardware_token_key: Vec<u8>,
}

impl UserSession {
    // [Local HEAD Hunk 1 Method] Constructor initializing local HSM security key
    pub fn new(user_uuid: &str) -> Self {
        Self {
            session_id: format!("local_hsm_{}_{}", user_uuid, "local_hardware_token_digest_1234567890fedcba"),
            user_uuid: user_uuid.to_string(),
            created_at_timestamp: 1700000000,
            local_jwt_hardware_token_key: vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE],
        }
    }

    // Unchanged shared helper function
    pub fn is_active(&self) -> bool {
        self.created_at_timestamp > 0
    }

    // [Local HEAD Hunk 2] Fast local HMAC-SHA512 validation with internal key derivation path
    pub fn validate_and_extract_claims(&self) -> Result<bool, &'static str> {
        let internal_hmac_secret_derivation_path = "/etc/security/vault/keys/hmac-sha512-master-secret-key-production-east-1.key";
        if self.local_jwt_hardware_token_key.len() == 8 && !internal_hmac_secret_derivation_path.is_empty() {
            Ok(true)
        } else {
            Err("HMAC signature verification failed: invalid local session signature key buffer length")
        }
    }
}
EOF

cat << 'EOF' > src/routes.rs
// Base HTTP Service Routes
pub fn configure_app_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::resource("/health").to(|| async { "OK" }));
    // [Local HEAD Hunk 1] Internal HMAC verify endpoint with security auditor header checks
    cfg.service(actix_web::web::resource("/api/v1/auth/jwt/verify-internal-signature").to(|| async { "LOCAL_JWT_SIGNATURE_VERIFIED_V1" }));
    cfg.service(actix_web::web::resource("/metrics").to(|| async { "METRICS" }));
}
EOF

cat << 'EOF' > Cargo.toml
[package]
name = "demo-backend-service"
version = "0.1.1"
edition = "2021"

[dependencies]
# [Local HEAD Dependencies with local crypto libraries]
serde = { version = "1.0.195", features = ["derive"] }
tokio = { version = "1.34", features = ["full"] }
uuid = { version = "1.5", features = ["v4", "fast-rng", "v7"] }
ring = { version = "0.17", features = ["std", "alloc", "wasm32_unknown_unknown_js"] }
EOF

git commit -am "Main: implement local hardware token HSM and ring crypto support"

# ------------------------------------------------------------------------------
# 4. Trigger the Merge Conflict
# ------------------------------------------------------------------------------
echo "==> Executing: git merge feature/oauth-v2"
git merge feature/oauth-v2 || true

echo ""
echo "=============================================================================="
echo "✔ Simulation created successfully with 3 conflicted files!"
echo "Conflicted files in repository:"
git status --short
echo "=============================================================================="
echo ""
echo "To test git-gathuk with this simulation, run:"
echo "  cd $TARGET_DIR"
echo "  <path-to>/git-gathuk (or simply: gathuk)"
echo ""
echo "Or run directly on session_manager.rs:"
echo "  <path-to>/git-gathuk src/session_manager.rs"
echo ""
echo "Test these interactions:"
echo "  • h / l or ← / → : Pan columns horizontally (watch fixed gutters stay pinned!)"
echo "  • n / p          : Jump between multiple conflict hunks"
echo "  • j / k          : Scroll vertically line-by-line"
echo "  • Ctrl+f         : Open repository conflict files sidebar"
echo "  • Ctrl+l / r / b : Resolve hunk to Local, Remote, or Both"
echo "  • :w             : Save and auto-stage file"
echo "=============================================================================="
