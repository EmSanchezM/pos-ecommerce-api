//! Notification entity — a single attempt to deliver content over one channel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{NotificationChannel, NotificationId, NotificationStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    id: NotificationId,
    channel: NotificationChannel,
    recipient: String,
    subject: Option<String>,
    body: String,
    metadata: JsonValue,
    status: NotificationStatus,
    attempts: i32,
    last_error: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Notification {
    pub fn create(
        channel: NotificationChannel,
        recipient: impl Into<String>,
        subject: Option<String>,
        body: impl Into<String>,
        metadata: JsonValue,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: NotificationId::new(),
            channel,
            recipient: recipient.into(),
            subject,
            body: body.into(),
            metadata,
            status: NotificationStatus::Queued,
            attempts: 0,
            last_error: None,
            sent_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: NotificationId,
        channel: NotificationChannel,
        recipient: String,
        subject: Option<String>,
        body: String,
        metadata: JsonValue,
        status: NotificationStatus,
        attempts: i32,
        last_error: Option<String>,
        sent_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            channel,
            recipient,
            subject,
            body,
            metadata,
            status,
            attempts,
            last_error,
            sent_at,
            created_at,
            updated_at,
        }
    }

    pub fn mark_sent(&mut self) {
        self.attempts += 1;
        self.status = NotificationStatus::Sent;
        self.sent_at = Some(Utc::now());
        self.last_error = None;
        self.updated_at = Utc::now();
    }

    pub fn mark_delivered(&mut self) {
        self.status = NotificationStatus::Delivered;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.attempts += 1;
        self.status = NotificationStatus::Failed;
        self.last_error = Some(error.into());
        self.updated_at = Utc::now();
    }

    // Getters
    pub fn id(&self) -> NotificationId {
        self.id
    }
    pub fn channel(&self) -> NotificationChannel {
        self.channel
    }
    pub fn recipient(&self) -> &str {
        &self.recipient
    }
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }
    pub fn body(&self) -> &str {
        &self.body
    }
    pub fn metadata(&self) -> &JsonValue {
        &self.metadata
    }
    pub fn status(&self) -> NotificationStatus {
        self.status
    }
    pub fn attempts(&self) -> i32 {
        self.attempts
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
    pub fn sent_at(&self) -> Option<DateTime<Utc>> {
        self.sent_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_notification_is_queued() {
        let n = Notification::create(
            NotificationChannel::Email,
            "user@example.com",
            Some("Hi".into()),
            "body",
            json!({}),
        );
        assert_eq!(n.status(), NotificationStatus::Queued);
        assert_eq!(n.attempts(), 0);
    }

    #[test]
    fn mark_sent_increments_attempts_and_clears_error() {
        let mut n = Notification::create(
            NotificationChannel::Sms,
            "+50498765432",
            None,
            "code: 1234",
            json!({}),
        );
        n.mark_failed("rate limited");
        assert_eq!(n.attempts(), 1);
        n.mark_sent();
        assert_eq!(n.status(), NotificationStatus::Sent);
        assert_eq!(n.attempts(), 2);
        assert!(n.sent_at().is_some());
        assert!(n.last_error().is_none());
    }
}
