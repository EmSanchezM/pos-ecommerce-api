//! HTTP client implementing the `ImpersonationTokenIssuer` port (impersonation
//! v2). Instead of signing tenant tokens locally, the backoffice calls
//! api-gateway's internal mint endpoint, authenticated with the shared
//! `INTERNAL_SERVICE_SECRET`. The tenant signing key (JWT_SECRET) therefore
//! never lives in this process.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use backoffice_identity::{BackofficeIdentityError, ImpersonationTokenIssuer};

const INTERNAL_TOKEN_HEADER: &str = "X-Internal-Service-Token";

pub struct HttpImpersonationTokenIssuer {
    client: reqwest::Client,
    /// Full URL of the gateway internal mint endpoint.
    endpoint: String,
    internal_secret: String,
}

impl HttpImpersonationTokenIssuer {
    /// `gateway_base_url` is the api-gateway base (e.g. `http://app:8000`).
    pub fn new(gateway_base_url: &str, internal_secret: String) -> Self {
        let endpoint = format!(
            "{}/internal/issue-impersonation-token",
            gateway_base_url.trim_end_matches('/')
        );
        Self {
            client: reqwest::Client::new(),
            endpoint,
            internal_secret,
        }
    }
}

#[derive(Serialize)]
struct IssueRequest<'a> {
    tenant_user_id: Uuid,
    operator_id: Uuid,
    operator_email: &'a str,
}

#[derive(Deserialize)]
struct IssueResponse {
    access_token: String,
}

#[async_trait]
impl ImpersonationTokenIssuer for HttpImpersonationTokenIssuer {
    async fn issue_impersonation_token(
        &self,
        tenant_user_id: Uuid,
        operator_id: Uuid,
        operator_email: &str,
    ) -> Result<String, BackofficeIdentityError> {
        let response = self
            .client
            .post(&self.endpoint)
            .header(INTERNAL_TOKEN_HEADER, &self.internal_secret)
            .json(&IssueRequest {
                tenant_user_id,
                operator_id,
                operator_email,
            })
            .send()
            .await
            .map_err(|e| {
                BackofficeIdentityError::JwtError(format!("impersonation mint request failed: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(BackofficeIdentityError::JwtError(format!(
                "impersonation mint endpoint returned {}",
                response.status()
            )));
        }

        let body: IssueResponse = response.json().await.map_err(|e| {
            BackofficeIdentityError::JwtError(format!(
                "impersonation mint response parse failed: {e}"
            ))
        })?;

        Ok(body.access_token)
    }
}
