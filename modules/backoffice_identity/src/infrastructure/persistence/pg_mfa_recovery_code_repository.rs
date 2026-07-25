//! PostgreSQL implementation of [`MfaRecoveryCodeRepository`].

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::MfaRecoveryCode;
use crate::domain::repositories::MfaRecoveryCodeRepository;
use crate::domain::value_objects::BackofficeUserId;
use crate::error::BackofficeIdentityError;

#[derive(Clone)]
pub struct PgMfaRecoveryCodeRepository {
    pool: PgPool,
}

impl PgMfaRecoveryCodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MfaRecoveryCodeRepository for PgMfaRecoveryCodeRepository {
    async fn replace_all_for_user(
        &self,
        user_id: BackofficeUserId,
        codes: &[MfaRecoveryCode],
    ) -> Result<(), BackofficeIdentityError> {
        // One transaction: a partial write would leave the operator holding a
        // printed set that does not match what is stored.
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM backoffice_mfa_recovery_codes WHERE backoffice_user_id = $1")
            .bind(user_id.as_uuid())
            .execute(&mut *tx)
            .await?;

        for code in codes {
            sqlx::query(
                r#"
                INSERT INTO backoffice_mfa_recovery_codes
                    (id, backoffice_user_id, code_hash, used_at, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(code.id())
            .bind(code.backoffice_user_id().as_uuid())
            .bind(code.code_hash())
            .bind(code.used_at())
            .bind(code.created_at())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn list_available_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<MfaRecoveryCode>, BackofficeIdentityError> {
        let rows = sqlx::query_as::<_, RecoveryCodeRow>(
            r#"
            SELECT id, backoffice_user_id, code_hash, used_at, created_at
            FROM backoffice_mfa_recovery_codes
            WHERE backoffice_user_id = $1 AND used_at IS NULL
            ORDER BY created_at
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn consume(&self, code_id: Uuid) -> Result<bool, BackofficeIdentityError> {
        // The `used_at IS NULL` guard is what makes single-use a property of
        // the database. A check-then-write in the use case could be raced by
        // two concurrent logins redeeming the same code.
        let result = sqlx::query(
            r#"
            UPDATE backoffice_mfa_recovery_codes
            SET used_at = NOW()
            WHERE id = $1 AND used_at IS NULL
            "#,
        )
        .bind(code_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    async fn delete_all_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<(), BackofficeIdentityError> {
        sqlx::query("DELETE FROM backoffice_mfa_recovery_codes WHERE backoffice_user_id = $1")
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RecoveryCodeRow {
    id: Uuid,
    backoffice_user_id: Uuid,
    code_hash: String,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<RecoveryCodeRow> for MfaRecoveryCode {
    fn from(row: RecoveryCodeRow) -> Self {
        MfaRecoveryCode::new(
            row.id,
            BackofficeUserId::from_uuid(row.backoffice_user_id),
            row.code_hash,
            row.used_at,
            row.created_at,
        )
    }
}

// =============================================================================
// DB-backed tests
//
// Single-use is enforced by the UPDATE guard, which no mock can exercise.
// These need a live database: `docker compose -f compose.dev.yml up -d db`.
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    use crate::domain::entities::BackofficeUser;
    use crate::domain::repositories::BackofficeUserRepository;
    use crate::domain::value_objects::BackofficeEmail;
    use crate::infrastructure::persistence::PgBackofficeUserRepository;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    /// Recovery codes are FK-bound to a user, so every test needs one.
    async fn seed_user(pool: &PgPool) -> BackofficeUserId {
        let user = BackofficeUser::create(
            BackofficeEmail::new(&format!("mfa-{}@platform.com", new_uuid().simple())).unwrap(),
            "hash".to_string(),
        );
        let id = *user.id();
        PgBackofficeUserRepository::new(pool.clone())
            .save(&user)
            .await
            .expect("seeding the user must succeed");
        id
    }

    fn codes(user_id: BackofficeUserId, count: usize) -> Vec<MfaRecoveryCode> {
        (0..count)
            .map(|i| MfaRecoveryCode::issue(user_id, format!("$argon2id$hash{i}")))
            .collect()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn replace_then_list_returns_the_stored_codes(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);

        repo.replace_all_for_user(user_id, &codes(user_id, 3))
            .await
            .unwrap();

        let stored = repo.list_available_for_user(user_id).await.unwrap();
        assert_eq!(stored.len(), 3);
        assert!(stored.iter().all(|c| c.is_available()));
    }

    /// Issuing a new set must invalidate the old one, or a code printed a year
    /// ago still works.
    #[sqlx::test(migrations = "../../migrations")]
    async fn replacing_discards_the_previous_set(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);

        let first = codes(user_id, 3);
        repo.replace_all_for_user(user_id, &first).await.unwrap();
        repo.replace_all_for_user(user_id, &codes(user_id, 2))
            .await
            .unwrap();

        let stored = repo.list_available_for_user(user_id).await.unwrap();
        assert_eq!(stored.len(), 2);
        let stored_ids: Vec<Uuid> = stored.iter().map(|c| c.id()).collect();
        for old in &first {
            assert!(
                !stored_ids.contains(&old.id()),
                "an old code survived a regeneration"
            );
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn consuming_marks_the_code_used(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);
        let set = codes(user_id, 2);
        repo.replace_all_for_user(user_id, &set).await.unwrap();

        assert!(repo.consume(set[0].id()).await.unwrap());

        let remaining = repo.list_available_for_user(user_id).await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id(), set[1].id());
    }

    /// THE single-use guarantee. The second redemption must fail at the DB, not
    /// depend on the caller having checked first.
    #[sqlx::test(migrations = "../../migrations")]
    async fn a_code_cannot_be_consumed_twice(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);
        let set = codes(user_id, 1);
        repo.replace_all_for_user(user_id, &set).await.unwrap();

        assert!(repo.consume(set[0].id()).await.unwrap());
        assert!(
            !repo.consume(set[0].id()).await.unwrap(),
            "a recovery code must buy exactly one login"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn consuming_an_unknown_code_reports_false(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);
        repo.replace_all_for_user(user_id, &codes(user_id, 1))
            .await
            .unwrap();

        assert!(!repo.consume(new_uuid()).await.unwrap());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn deleting_clears_every_code(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);
        repo.replace_all_for_user(user_id, &codes(user_id, 4))
            .await
            .unwrap();

        repo.delete_all_for_user(user_id).await.unwrap();
        assert!(
            repo.list_available_for_user(user_id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    /// One operator's codes must never appear in another's set.
    #[sqlx::test(migrations = "../../migrations")]
    async fn codes_are_scoped_to_their_user(pool: PgPool) {
        let alice = seed_user(&pool).await;
        let bob = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);

        repo.replace_all_for_user(alice, &codes(alice, 3))
            .await
            .unwrap();
        repo.replace_all_for_user(bob, &codes(bob, 1))
            .await
            .unwrap();

        assert_eq!(repo.list_available_for_user(alice).await.unwrap().len(), 3);
        assert_eq!(repo.list_available_for_user(bob).await.unwrap().len(), 1);
    }

    /// Replacing one user's set must not touch another's.
    #[sqlx::test(migrations = "../../migrations")]
    async fn replacing_one_user_does_not_affect_another(pool: PgPool) {
        let alice = seed_user(&pool).await;
        let bob = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);

        repo.replace_all_for_user(alice, &codes(alice, 3))
            .await
            .unwrap();
        repo.replace_all_for_user(bob, &codes(bob, 2))
            .await
            .unwrap();
        repo.replace_all_for_user(alice, &codes(alice, 1))
            .await
            .unwrap();

        assert_eq!(repo.list_available_for_user(bob).await.unwrap().len(), 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn listing_a_user_with_no_codes_is_empty(pool: PgPool) {
        let user_id = seed_user(&pool).await;
        let repo = PgMfaRecoveryCodeRepository::new(pool);
        assert!(
            repo.list_available_for_user(user_id)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
