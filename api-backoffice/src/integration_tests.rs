// Integration tests for the backoffice API router.
//
// P3-T07 scenario: server routes are wired correctly.
// These tests use Axum's oneshot testing approach — no real server or DB.

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::{
        Extension, Router,
        body::Body,
        extract::ConnectInfo,
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

    /// Builds the router the way `main` serves it.
    ///
    /// Stands in for `into_make_service_with_connect_info`: the rate limiters
    /// key on the peer IP, and without a `ConnectInfo` in the extensions every
    /// request is rejected with 500 (`GovernorError::UnableToExtractKey`)
    /// before it ever reaches the route.
    ///
    /// NOTE: this deliberately does NOT use `axum::extract::MockConnectInfo`.
    /// That layer inserts a `MockConnectInfo<SocketAddr>` extension, which only
    /// axum's own `ConnectInfo` *extractor* knows to fall back to. Middleware
    /// that reads the extension directly — `tower_governor` does
    /// `extensions().get::<ConnectInfo<SocketAddr>>()` — finds nothing and
    /// fails. Inserting the real `ConnectInfo` is what mirrors production.
    ///
    /// Each call builds a fresh router, so each test gets its own quota state.
    fn make_app() -> Router {
        build_router(make_state()).layer(Extension(ConnectInfo(SocketAddr::from((
            [127, 0, 0, 1],
            9999,
        )))))
    }

    /// P3-T07: GET /health returns 200.
    #[tokio::test]
    async fn health_returns_200() {
        let app = make_app();
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
        let app = make_app();
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
        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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
        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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
        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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
        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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
        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
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

        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/analytics/kpis/sales.revenue_total?window=nope")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // -------------------------------------------------------------------------
    // Audit log read route
    // -------------------------------------------------------------------------

    /// GET /backoffice/audit without a token returns 401 (auth middleware).
    #[tokio::test]
    async fn audit_log_route_requires_auth() {
        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/audit")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// A valid token WITHOUT `platform:audit.read` is rejected with 403 before
    /// any DB access. The audit trail records who suspended orgs and who
    /// impersonated whom, so the gate matters more here than anywhere else.
    #[tokio::test]
    async fn audit_log_denied_without_permission() {
        let token = backoffice_token(&["platform:org.list"]);

        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/audit")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "missing platform:audit.read must be rejected with 403"
        );
    }

    /// With `platform:audit.read` the handler passes the gate and reaches the
    /// DB layer — 500 against the lazy (unconnected) pool, not 403/401.
    #[tokio::test]
    async fn audit_log_with_permission_reaches_db() {
        let token = backoffice_token(&["platform:audit.read"]);

        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/audit")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// Filters and pagination are accepted and still reach the DB layer.
    #[tokio::test]
    async fn audit_log_accepts_filters_and_pagination() {
        let token = backoffice_token(&["platform:audit.read"]);

        let app = make_app();
        let request = Request::builder()
            .uri(
                "/backoffice/audit?action=org.suspend\
                 &actor_id=00000000-0000-7000-8000-000000000001\
                 &target_org_id=00000000-0000-7000-8000-000000000002\
                 &page=2&page_size=10",
            )
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// A malformed UUID filter is rejected with 400 by query deserialization,
    /// before the permission gate or any DB access.
    #[tokio::test]
    async fn audit_log_malformed_uuid_filter_is_400() {
        let token = backoffice_token(&["platform:audit.read"]);

        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/audit?actor_id=not-a-uuid")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// An out-of-range page_size must be clamped, not rejected and not passed
    /// through — the handler still reaches the DB layer.
    #[tokio::test]
    async fn audit_log_oversized_page_size_is_clamped_not_rejected() {
        let token = backoffice_token(&["platform:audit.read"]);

        let app = make_app();
        let request = Request::builder()
            .uri("/backoffice/audit?page=4294967295&page_size=4294967295")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "extreme pagination must be clamped and reach the DB, not panic or 400"
        );
    }

    // -------------------------------------------------------------------------
    // Rate limiting
    // -------------------------------------------------------------------------

    fn login_request() -> Request<Body> {
        let body = json!({ "email": "admin@platform.com", "password": "wrong-password" });
        Request::builder()
            .uri("/backoffice/auth/login")
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Repeated login attempts from one peer IP must eventually be throttled.
    /// This is the control that makes an online brute force against the
    /// platform-owner credential impractical — if it silently stopped working,
    /// nothing else in the system would notice.
    #[tokio::test]
    async fn login_is_rate_limited_after_a_burst() {
        let app = make_app();

        let mut statuses = Vec::new();
        for _ in 0..12 {
            let response = app.clone().oneshot(login_request()).await.unwrap();
            statuses.push(response.status());
        }

        assert!(
            statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
            "sustained login attempts must be throttled with 429, got {statuses:?}"
        );
    }

    /// The first attempts must NOT be throttled — a limiter that rejects an
    /// operator's first password entry is a broken limiter, not a strict one.
    #[tokio::test]
    async fn first_login_attempt_is_not_throttled() {
        let app = make_app();

        let response = app.oneshot(login_request()).await.unwrap();

        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "the first login attempt must never be rate limited"
        );
    }

    /// The authenticated surface is throttled too, but generously enough that a
    /// normal burst of operator activity passes untouched.
    #[tokio::test]
    async fn authenticated_route_tolerates_a_normal_burst() {
        let token = backoffice_token(&["platform:audit.read"]);
        let app = make_app();

        for i in 0..20 {
            let request = Request::builder()
                .uri("/backoffice/audit")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap();

            let status = app.clone().oneshot(request).await.unwrap().status();
            assert_ne!(
                status,
                StatusCode::TOO_MANY_REQUESTS,
                "request {i} of a normal operator burst was throttled"
            );
        }
    }

    /// A forged `X-Forwarded-For` must not buy a fresh quota bucket. This is
    /// the whole reason the limiter keys on the peer IP rather than on
    /// `SmartIpKeyExtractor`, which reads that header first.
    #[tokio::test]
    async fn spoofed_forwarded_for_does_not_reset_the_login_quota() {
        let app = make_app();

        let mut statuses = Vec::new();
        for i in 0..12 {
            let body = json!({ "email": "admin@platform.com", "password": "wrong-password" });
            let request = Request::builder()
                .uri("/backoffice/auth/login")
                .method("POST")
                .header(CONTENT_TYPE, "application/json")
                // A different forged origin on every attempt.
                .header("X-Forwarded-For", format!("203.0.113.{i}"))
                .header("X-Real-IP", format!("198.51.100.{i}"))
                .body(Body::from(body.to_string()))
                .unwrap();

            statuses.push(app.clone().oneshot(request).await.unwrap().status());
        }

        assert!(
            statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
            "rotating X-Forwarded-For must NOT bypass the login limiter, got {statuses:?}"
        );
    }

    /// Health must stay reachable — it is public and outside the authenticated
    /// router, so the API limiter must not cover it.
    #[tokio::test]
    async fn health_is_not_rate_limited() {
        let app = make_app();

        for _ in 0..80 {
            let request = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            let status = app.clone().oneshot(request).await.unwrap().status();
            assert_eq!(status, StatusCode::OK, "health must never be throttled");
        }
    }
}
