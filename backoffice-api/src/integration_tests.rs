// Integration tests for the backoffice API router.
//
// P3-T07 scenario: server routes are wired correctly.
// These tests use Axum's oneshot testing approach — no real server or DB.

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::CONTENT_TYPE},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::router::build_router;
    use crate::state::BackofficeAppState;

    fn make_state() -> BackofficeAppState {
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");
        BackofficeAppState::from_pool(
            pool,
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        )
    }

    /// P3-T07: GET /health returns 200.
    #[tokio::test]
    async fn health_returns_200() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// P3-T07: POST /backoffice/auth/login with no DB returns 500 (pool not connected),
    /// which means the route IS registered and auth logic was reached.
    /// We verify the route exists by asserting we do NOT get 404.
    #[tokio::test]
    async fn login_route_is_registered() {
        let app = build_router(make_state());
        let body = json!({
            "email": "admin@platform.com",
            "password": "wrong-password"
        });
        let request = Request::builder()
            .uri("/backoffice/auth/login")
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // We get either 401 (wrong creds) or 500 (pool not connected) —
        // not 404. The important check is that the route is mounted.
        assert_ne!(
            response.status(),
            StatusCode::NOT_FOUND,
            "login route must be registered"
        );
    }

    /// P3-T07: GET /backoffice/orgs without auth returns 401 (middleware applied).
    #[tokio::test]
    async fn orgs_route_requires_auth() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/orgs")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "GET /backoffice/orgs without token must return 401"
        );
    }

    /// P3-T07: GET /backoffice/orgs with a valid Backoffice token reaches the stub (501).
    #[tokio::test]
    async fn orgs_route_accepts_valid_backoffice_token() {
        use backoffice_identity::{
            BackofficeEmail, BackofficeUser, BackofficeUserId, JwtBackofficeTokenService,
            BackofficeTokenService,
        };
        use chrono::Utc;

        // Issue a valid backoffice-audience token
        let svc = JwtBackofficeTokenService::with_issuer(
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        );
        let user = BackofficeUser::new(
            BackofficeUserId::new(),
            BackofficeEmail::new("admin@platform.com").unwrap(),
            "hashed".to_string(),
            None,
            true,
            None,
            Utc::now(),
            Utc::now(),
        );
        let token = svc
            .issue_backoffice_token(&user, &["platform:org.list".to_string()])
            .unwrap();

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/orgs")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // With a valid Backoffice token, middleware passes and we hit the stub
        // which returns 501.
        assert_eq!(
            response.status(),
            StatusCode::NOT_IMPLEMENTED,
            "GET /backoffice/orgs with valid Backoffice token must reach stub handler"
        );
    }
}
