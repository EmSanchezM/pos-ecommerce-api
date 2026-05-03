//! # Booking Module
//!
//! Vertical for the **Servicios con cita** vertical (salons, spas, repair
//! shops). Provides:
//!
//! - **Domain**: `Resource` (person/equipment/room), `ResourceCalendar`
//!   (recurring weekly availability), `Service` (bookable, with duration +
//!   buffers + optional deposit), `Appointment` (the aggregate root with the
//!   `Scheduled → Confirmed → InProgress → Completed | Canceled | NoShow`
//!   workflow), `BookingPolicy` (per-store cancellation/deposit rules).
//!
//! - **Application**: Pure availability math (`availability::generate_slots`,
//!   `subtract_booked`), CRUD use cases for resources/services/policies, the
//!   appointment lifecycle (`Confirm`/`Start`/`Complete`/`Cancel`/`NoShow`),
//!   and the public flow (`ListPublicServices`, `CheckAvailability`,
//!   `BookAppointmentPublic`, `GetPublicAppointment`). The public flow does
//!   **not** go through `cart`; the `Appointment` is itself the transactional
//!   aggregate (see roadmap section "Decisión arquitectónica clave").
//!
//! - **Infrastructure**: `Pg*Repository` SQLx implementations.
//!   `PgAppointmentRepository::save_with_slot_check` runs a `FOR UPDATE`
//!   overlap check inside a transaction so concurrent public bookings cannot
//!   double-book the same resource window.
//!
//! See `docs/roadmap-modulos.md` (Fase 2.2 + "Plan detallado — Módulo
//! booking") for the full contract.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::BookingError;

// Domain
pub use domain::entities::{Appointment, BookingPolicy, Resource, ResourceCalendar, Service};
pub use domain::repositories::{
    AppointmentRepository, BookingPolicyRepository, ListAppointmentsFilters,
    ResourceCalendarRepository, ResourceRepository, ServiceRepository,
};
pub use domain::value_objects::{
    AppointmentId, AppointmentStatus, BookingPolicyId, ResourceCalendarId, ResourceId,
    ResourceType, ServiceId, TimeSlot,
};

// Application
pub use application::availability::{WorkingWindow, generate_slots, subtract_booked};
pub use application::dtos::{
    AppointmentResponse, AssignServiceResourcesCommand, BookingPolicyResponse, CalendarWindowDto,
    CancelAppointmentCommand, CheckAvailabilityQuery, CompleteAppointmentCommand,
    CreateAppointmentAdminCommand, CreateResourceCommand, CreateServiceCommand, PublicBookCommand,
    PublicBookingResponse, ResourceAvailabilityResponse, ResourceCalendarEntryResponse,
    ResourceResponse, ServiceResponse, SetResourceCalendarCommand, UpdateResourceCommand,
    UpdateServiceCommand, UpsertBookingPolicyCommand,
};
pub use application::subscriber::BookingEventSubscriber;
pub use application::use_cases::{
    AssignServiceResourcesUseCase, BookAppointmentPublicUseCase, CancelAppointmentUseCase,
    CheckAvailabilityUseCase, CompleteAppointmentUseCase, ConfirmAppointmentUseCase,
    CreateAppointmentAdminUseCase, CreateResourceUseCase, CreateServiceUseCase,
    DeactivateResourceUseCase, DeactivateServiceUseCase, GetAppointmentUseCase,
    GetBookingPolicyUseCase, GetPublicAppointmentUseCase, GetResourceCalendarUseCase,
    ListAppointmentsUseCase, ListPublicServicesUseCase, ListResourcesUseCase, ListServicesUseCase,
    NoShowAppointmentUseCase, SetResourceCalendarUseCase, StartAppointmentUseCase,
    UpdateResourceUseCase, UpdateServiceUseCase, UpsertBookingPolicyUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgAppointmentRepository, PgBookingPolicyRepository, PgResourceCalendarRepository,
    PgResourceRepository, PgServiceRepository,
};
