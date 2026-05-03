//! Public, unauthenticated booking flow.
//!
//! `CheckAvailabilityUseCase` and `BookAppointmentPublicUseCase` are the only
//! two use cases exposed via `/api/v1/public/booking/...`. Both run without a
//! `CurrentUser` — callers come straight off the storefront/booking widget.
//!
//! The booking step relies on `AppointmentRepository::save_with_slot_check`
//! to detect double-booking under concurrency. The repository implementation
//! does the lock + overlap check inside a single transaction; if a conflict
//! exists it returns `BookingError::SlotConflict` and the caller surfaces a
//! 409 to the client.

use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use uuid::Uuid;

use crate::BookingError;
use crate::application::availability::{WorkingWindow, generate_slots, subtract_booked};
use crate::application::dtos::{
    AppointmentResponse, CheckAvailabilityQuery, PublicBookCommand, PublicBookingResponse,
    ResourceAvailabilityResponse, ServiceResponse,
};
use crate::domain::entities::Appointment;
use crate::domain::repositories::{
    AppointmentRepository, ResourceCalendarRepository, ResourceRepository, ServiceRepository,
};
use crate::domain::value_objects::{ResourceId, ServiceId, TimeSlot};

const DEFAULT_GRANULARITY_MINUTES: u32 = 15;

pub struct ListPublicServicesUseCase {
    services: Arc<dyn ServiceRepository>,
}

impl ListPublicServicesUseCase {
    pub fn new(services: Arc<dyn ServiceRepository>) -> Self {
        Self { services }
    }

    pub async fn execute(&self, store_id: Uuid) -> Result<Vec<ServiceResponse>, BookingError> {
        let services = self.services.list_by_store(store_id, true).await?;
        Ok(services.iter().map(ServiceResponse::from).collect())
    }
}

pub struct CheckAvailabilityUseCase {
    services: Arc<dyn ServiceRepository>,
    resources: Arc<dyn ResourceRepository>,
    calendars: Arc<dyn ResourceCalendarRepository>,
    appointments: Arc<dyn AppointmentRepository>,
}

impl CheckAvailabilityUseCase {
    pub fn new(
        services: Arc<dyn ServiceRepository>,
        resources: Arc<dyn ResourceRepository>,
        calendars: Arc<dyn ResourceCalendarRepository>,
        appointments: Arc<dyn AppointmentRepository>,
    ) -> Self {
        Self {
            services,
            resources,
            calendars,
            appointments,
        }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        query: CheckAvailabilityQuery,
    ) -> Result<Vec<ResourceAvailabilityResponse>, BookingError> {
        let service_id = ServiceId::from_uuid(query.service_id);
        let service = self
            .services
            .find_by_id(service_id)
            .await?
            .ok_or_else(|| BookingError::ServiceNotFound(query.service_id))?;
        if service.store_id() != store_id {
            return Err(BookingError::ServiceNotFound(query.service_id));
        }

        let mut eligible = self.services.find_eligible_resources(service_id).await?;
        if let Some(rid) = query.resource_id {
            let target = ResourceId::from_uuid(rid);
            eligible.retain(|r| *r == target);
        }

        let day_start: DateTime<Utc> = Utc.from_utc_datetime(
            &query
                .date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| BookingError::InvalidTimeRange)?,
        );
        let day_end = day_start + Duration::days(1);
        let weekday_index = weekday_to_index(query.date);

        let mut out = Vec::new();
        for resource_id in eligible {
            let resource = match self.resources.find_by_id(resource_id).await? {
                Some(r) if r.is_active() => r,
                _ => continue,
            };

            let calendar = self.calendars.find_by_resource(resource_id).await?;
            let windows: Vec<WorkingWindow> = calendar
                .iter()
                .filter(|c| c.is_active() && c.day_of_week() == weekday_index)
                .map(|c| WorkingWindow {
                    start: c.start_time(),
                    end: c.end_time(),
                })
                .collect();
            if windows.is_empty() {
                continue;
            }

            let candidates = generate_slots(
                query.date,
                &windows,
                service.duration_minutes() as u32,
                service.buffer_minutes_before() as u32,
                service.buffer_minutes_after() as u32,
                DEFAULT_GRANULARITY_MINUTES,
            );

            let booked: Vec<TimeSlot> = self
                .appointments
                .list_occupying_slots(resource_id, day_start, day_end)
                .await?
                .iter()
                .map(|a| TimeSlot {
                    starts_at: a.starts_at(),
                    ends_at: a.ends_at(),
                })
                .collect();

            let free = subtract_booked(candidates, &booked);
            out.push(ResourceAvailabilityResponse {
                resource_id: resource_id.into_uuid(),
                resource_name: resource.name().to_string(),
                slots: free,
            });
        }

        Ok(out)
    }
}

pub struct BookAppointmentPublicUseCase {
    services: Arc<dyn ServiceRepository>,
    resources: Arc<dyn ResourceRepository>,
    appointments: Arc<dyn AppointmentRepository>,
}

impl BookAppointmentPublicUseCase {
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
        store_id: Uuid,
        cmd: PublicBookCommand,
    ) -> Result<PublicBookingResponse, BookingError> {
        let service_id = ServiceId::from_uuid(cmd.service_id);
        let service = self
            .services
            .find_by_id(service_id)
            .await?
            .ok_or_else(|| BookingError::ServiceNotFound(cmd.service_id))?;
        if service.store_id() != store_id || !service.is_active() {
            return Err(BookingError::ServiceNotFound(cmd.service_id));
        }

        let eligible = self.services.find_eligible_resources(service_id).await?;
        if eligible.is_empty() {
            return Err(BookingError::ResourceNotEligibleForService {
                service_id: cmd.service_id,
                resource_id: cmd.resource_id.unwrap_or_default(),
            });
        }

        let resource_id = match cmd.resource_id {
            Some(rid) => {
                let target = ResourceId::from_uuid(rid);
                if !eligible.contains(&target) {
                    return Err(BookingError::ResourceNotEligibleForService {
                        service_id: cmd.service_id,
                        resource_id: rid,
                    });
                }
                target
            }
            // Pick the first eligible resource. The slot-check below catches
            // conflicts; v1.1 can pick the resource with the lightest day.
            None => eligible[0],
        };

        if self
            .resources
            .find_by_id(resource_id)
            .await?
            .filter(|r| r.is_active())
            .is_none()
        {
            return Err(BookingError::ResourceNotFound(resource_id.into_uuid()));
        }

        let ends_at = cmd.starts_at + Duration::minutes(service.duration_minutes() as i64);
        let appointment = Appointment::schedule(
            store_id,
            service_id,
            resource_id,
            cmd.customer_id,
            cmd.customer_name,
            cmd.customer_email,
            cmd.customer_phone,
            cmd.starts_at,
            ends_at,
            cmd.notes,
            None, // public booking has no actor user
        )?;
        self.appointments.save_with_slot_check(&appointment).await?;

        let token = appointment.public_token().to_string();
        Ok(PublicBookingResponse {
            appointment: AppointmentResponse::from(&appointment),
            public_token: token,
        })
    }
}

pub struct GetPublicAppointmentUseCase {
    appointments: Arc<dyn AppointmentRepository>,
}

impl GetPublicAppointmentUseCase {
    pub fn new(appointments: Arc<dyn AppointmentRepository>) -> Self {
        Self { appointments }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        token: &str,
    ) -> Result<AppointmentResponse, BookingError> {
        let appt = self
            .appointments
            .find_by_public_token(token)
            .await?
            .ok_or(BookingError::InvalidPublicToken)?;
        if appt.id().into_uuid() != id {
            return Err(BookingError::InvalidPublicToken);
        }
        Ok(AppointmentResponse::from(&appt))
    }
}

fn weekday_to_index(date: NaiveDate) -> i16 {
    use chrono::Datelike;
    date.weekday().num_days_from_sunday() as i16
}
