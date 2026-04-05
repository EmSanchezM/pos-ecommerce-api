-- Setup readonly PostgreSQL user for DBHub MCP server
-- Run this as a superuser/admin on the posecommerce database

CREATE USER readonly WITH PASSWORD 'password';
GRANT CONNECT ON DATABASE posecommerce TO readonly;
GRANT USAGE ON SCHEMA public TO readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO readonly;
