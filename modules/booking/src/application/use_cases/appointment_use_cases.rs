//! Appointment lifecycle use cases (auth/admin side).
//!
//! `book_appointment_public` lives in `public_booking.rs` because it has its
//! own external-facing contract.

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::BookingError;
use crate::application::dtos::{
    CancelAppointmentCommand, CompleteAppointmentCommand, CreateAppointmentAdminCommand,
};
use crate::domain::entities::Appointment;
use crate::domain::repositories::{
    AppointmentRepository, BookingPolicyRepository, ListAppointmentsFilters, ResourceRepository,
    ServiceRepository,
};
use crate::domain::value_objects::{AppointmentId, ResourceId, ServiceId};

pub struct CreateAppointmentAdminUseCase {
    services: Arc<dyn ServiceRepository>,
    resources: Arc<dyn ResourceRepository>,
    appointments: Arc<dyn AppointmentRepository>,
}

impl CreateAppointmentAdminUseCase {
    pub fn new(
        services: Arc<dyn ServiceRepository>,
        resources: Arc<dyn ResourceRepository>,
        appointments: Arc<dyn AppointmentRepository>,
    ) -> Self {
        Self {
            services,
            resources,
            appointments,
        }
    }

    pub async fn execute(
        &self,
        cmd: CreateAppointmentAdminCommand,
        actor_id: uuid::Uuid,
    ) -> Result<Appointment, BookingError> {
        let service_id = ServiceId::from_uuid(cmd.service_id);
        let resource_id = ResourceId::from_uuid(cmd.resource_id);

        let service = self
            .services
            .find_by_id(service_id)
            .await?
            .ok_or_else(|| BookingError::ServiceNotFound(cmd.service_id))?;

        if self.resources.find_by_id(resource_id).await?.is_none() {
            return Err(BookingError::ResourceNotFound(cmd.resource_id));
        }

        // Confirm the resource is eligible to perform the service.
        let eligible = self.services.find_eligible_resources(service_id).await?;
        if !eligible.contains(&resource_id) {
            return Err(BookingError::ResourceNotEligibleForService {
                service_id: cmd.service_id,
                resource_id: cmd.resource_id,
            });
        }

        let ends_at = cmd.starts_at + Duration::minutes(service.duration_minutes() as i64);
        let appointment = Appointment::schedule(
            cmd.store_id,
            service_id,
            resource_id,
            cmd.customer_id,
            cmd.customer_name,
            cmd.customer_email,
            cmd.customer_phone,
            cmd.starts_at,
            ends_at,
            cmd.notes,
            Some(actor_id),
        )?;
        self.appointments.save_with_slot_check(&appointment).await?;
        Ok(appointment)
    }
}

pub struct ListAppointmentsUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl ListAppointmentsUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(
        &self,
        filters: ListAppointmentsFilters,
    ) -> Result<Vec<Appointment>, BookingError> {
        self.appointments.list(filters).await
    }
}

pub struct GetAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl GetAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(&self, id: AppointmentId) -> Result<Appointment, BookingError> {
        self.appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))
    }
}

pub struct ConfirmAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl ConfirmAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(&self, id: AppointmentId) -> Result<Appointment, BookingError> {
        let mut appt = self
            .appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))?;
        appt.confirm()?;
        self.appointments.update(&appt).await?;
        Ok(appt)
    }
}

pub struct StartAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl StartAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(&self, id: AppointmentId) -> Result<Appointment, BookingError> {
        let mut appt = self
            .appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))?;
        appt.start()?;
        self.appointments.update(&appt).await?;
        Ok(appt)
    }
}

pub struct CompleteAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl CompleteAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(
        &self,
        id: AppointmentId,
        cmd: CompleteAppointmentCommand,
    ) -> Result<Appointment, BookingError> {
        let mut appt = self
            .appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))?;
        appt.complete(cmd.generated_sale_id)?;
        self.appointments.update(&appt).await?;
        Ok(appt)
    }
}

pub struct CancelAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
    policies: Arc<dyn BookingPolicyRepository>,
}

impl CancelAppointmentUseCase {
    pub fn new(
        appointments: Arc<dyn AppointmentRepository>,
        policies: Arc<dyn BookingPolicyRepository>,
    ) -> Self {
        Self {
            appointments,
            policies,
        }
    }

    pub async fn execute(
        &self,
        id: AppointmentId,
        cmd: CancelAppointmentCommand,
    ) -> Result<Appointment, BookingError> {
        let mut appt = self
            .appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))?;

        // Enforce cancellation window if a policy exists for the store.
        if let Some(policy) = self.policies.find_by_store(appt.store_id()).await? {
            let now = Utc::now();
            let lead = appt.starts_at() - now;
            let required = Duration::hours(policy.cancellation_window_hours() as i64);
            if lead < required {
                return Err(BookingError::OutsideCancellationWindow {
                    window_hours: policy.cancellation_window_hours(),
                });
            }
        }

        appt.cancel(cmd.reason)?;
        self.appointments.update(&appt).await?;
        Ok(appt)
    }
}

pub struct NoShowAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl NoShowAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(&self, id: AppointmentId) -> Result<Appointment, BookingError> {
        let mut appt = self
            .appointments
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::AppointmentNotFound(id.into_uuid()))?;
        appt.mark_no_show()?;
        self.appointments.update(&appt).await?;
        Ok(appt)
    }
}
