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
            "http://localhost:8000".to_string(),
            "internal-secret-test".to_string(),
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

    // -------------------------------------------------------------------------
    // Phase 6 — Slice B: subscription admin routes
    // -------------------------------------------------------------------------

    fn org_path(suffix: &str) -> String {
        format!("/backoffice/subscriptions/00000000-0000-0000-0000-000000000001{suffix}")
    }

    /// POST force-cancel without a token returns 401 (auth middleware).
    #[tokio::test]
    async fn subs_force_cancel_requires_auth() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri(org_path("/force-cancel"))
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"reason": "fraud"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// POST force-cancel with a token lacking `platform:subscription.force_cancel`
    /// → 403, before any DB access.
    #[tokio::test]
    async fn subs_force_cancel_denied_without_permission() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri(org_path("/force-cancel"))
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"reason": "fraud"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// POST change-plan with a token lacking `platform:subscription.override_billing`
    /// → 403.
    #[tokio::test]
    async fn subs_change_plan_denied_without_permission() {
        let token = backoffice_token(&["platform:subscription.force_cancel"]);

        let app = build_router(make_state());
        let body = json!({
            "reason": "promo migration",
            "new_plan_id": "00000000-0000-0000-0000-000000000002"
        });
        let request = Request::builder()
            .uri(org_path("/change-plan"))
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// GET subscription with `platform:org.list` passes the gate and reaches the
    /// use case; the lazy pool then fails → 500.
    #[tokio::test]
    async fn subs_get_with_permission_reaches_db() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri(org_path(""))
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // -------------------------------------------------------------------------
    // Phase 6 — Slice C: manual dunning trigger
    // -------------------------------------------------------------------------

    const DUNNING_PATH: &str = "/backoffice/dunning/019ee5dd-0000-7000-8000-000000000abc/trigger";

    /// POST dunning trigger without a token returns 401.
    #[tokio::test]
    async fn dunning_trigger_requires_auth() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri(DUNNING_PATH)
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"reason": "manual retry"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// POST dunning trigger with a token lacking `platform:dunning.trigger`
    /// → 403, before any DB access.
    #[tokio::test]
    async fn dunning_trigger_denied_without_permission() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri(DUNNING_PATH)
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"reason": "manual retry"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// POST dunning trigger with `platform:dunning.trigger` passes the gate and
    /// reaches the org-resolution lookup; the lazy pool then fails → 500.
    #[tokio::test]
    async fn dunning_trigger_with_permission_reaches_db() {
        let token = backoffice_token(&["platform:dunning.trigger"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri(DUNNING_PATH)
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"reason": "manual retry"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // -------------------------------------------------------------------------
    // Phase 6 — Slice D: cross-org analytics
    // -------------------------------------------------------------------------

    /// GET analytics overview without a token returns 401.
    #[tokio::test]
    async fn analytics_overview_requires_auth() {
        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/analytics/overview")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// GET analytics overview with a token lacking `platform:analytics.read`
    /// → 403, before any DB access.
    #[tokio::test]
    async fn analytics_overview_denied_without_permission() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/analytics/overview")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// GET a single KPI with `platform:analytics.read` passes the gate and
    /// reaches the repository; the lazy pool then fails → 500.
    #[tokio::test]
    async fn analytics_kpi_with_permission_reaches_db() {
        let token = backoffice_token(&["platform:analytics.read"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/analytics/kpis/sales.revenue_total?window=this_month")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// An invalid `window` is rejected with 400 before touching the DB.
    #[tokio::test]
    async fn analytics_kpi_invalid_window_is_400() {
        let token = backoffice_token(&["platform:analytics.read"]);

        let app = build_router(make_state());
        let request = Request::builder()
            .uri("/backoffice/analytics/kpis/sales.revenue_total?window=nope")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
