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
            "tenant-secret-at-least-32-bytes-long-xx".to_string(),
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

    /// Issues a valid backoffice-audience token carrying `permissions`.
    fn backoffice_token(permissions: &[&str]) -> String {
        use backoffice_identity::{
            BackofficeEmail, BackofficeTokenService, BackofficeUser, BackofficeUserId,
            JwtBackofficeTokenService,
        };
        use chrono::Utc;

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
        let perms: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
        svc.issue_backoffice_token(&user, &perms).unwrap()
    }

    /// GET /backoffice/orgs with a token that HAS `platform:org.list` passes
    /// both auth and the permission gate, reaching the use case. Against the
    /// lazy (unconnected) pool the query fails, so we get 500 — proving the
    /// handler ran past authorization rather than short-circuiting at 403/401.
    #[tokio::test]
    async fn orgs_route_accepts_valid_backoffice_token() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/orgs")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "with org.list the handler must reach the DB layer (500 on no DB), not 403/401"
        );
    }

    /// GET /backoffice/orgs with a valid token that LACKS `platform:org.list`
    /// is rejected by the permission gate with 403 — before any DB access.
    #[tokio::test]
    async fn orgs_route_denies_token_without_permission() {
        let token = backoffice_token(&["platform:audit.read"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/orgs")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "missing platform:org.list must be rejected with 403"
        );
    }

    // -------------------------------------------------------------------------
    // Phase 6 — Slice A: plan catalog routes
    // -------------------------------------------------------------------------

    /// GET /backoffice/plans without a token returns 401 (auth middleware).
    #[tokio::test]
    async fn plans_route_requires_auth() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/plans")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// GET /backoffice/plans with a token lacking `platform:plan.read` → 403,
    /// before any DB access.
    #[tokio::test]
    async fn plans_list_denied_without_permission() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/plans")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "missing platform:plan.read must be rejected with 403"
        );
    }

    /// GET /backoffice/plans with `platform:plan.read` passes the gate and
    /// reaches the use case; the lazy pool then fails → 500 (proves the handler
    /// ran past authorization).
    #[tokio::test]
    async fn plans_list_with_permission_reaches_db() {
        let token = backoffice_token(&["platform:plan.read"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/plans")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "with plan.read the handler must reach the DB layer (500 on no DB)"
        );
    }
}
