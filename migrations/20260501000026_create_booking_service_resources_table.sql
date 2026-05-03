-- Booking: M2M between services and resources eligible to perform them.
--
-- A service can list multiple resources (any stylist can do a haircut), and
-- a resource can perform multiple services (one stylist, several services).
-- Empty set = the service cannot be booked yet.

CREATE TABLE IF NOT EXISTS booking_service_resources (
    service_id  UUID NOT NULL REFERENCES booking_services(id)  ON DELETE CASCADE,
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE CASCADE,
    PRIMARY KEY (service_id, resource_id)
);

CREATE INDEX IF NOT EXISTS idx_booking_service_resources_resource
    ON booking_service_resources (resource_id);
