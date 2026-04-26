-- Per-store image storage configuration. Mirrors payment_gateways /
-- delivery_providers: super-admin manages CUD; credentials encrypted.
--
-- provider_type values:
--   local_server  - filesystem under IMAGE_STORAGE_ROOT/store_<id>/...
--   s3            - AWS S3 bucket
--   gcs           - Google Cloud Storage
--   cloudinary    - Cloudinary
--   azure_blob    - Azure Blob Storage

CREATE TABLE IF NOT EXISTS image_storage_providers (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    name VARCHAR(100) NOT NULL,
    provider_type VARCHAR(30) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    api_key_encrypted TEXT NOT NULL,
    secret_key_encrypted TEXT NOT NULL,
    -- Per-provider extras packaged as JSON (bucket, region, account, root_path,
    -- public_base_url, etc). Adapter-specific.
    config_json TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT image_storage_providers_store_name_unique UNIQUE (store_id, name)
);

CREATE UNIQUE INDEX IF NOT EXISTS image_storage_providers_one_default_per_store
    ON image_storage_providers(store_id)
    WHERE is_default = true;

CREATE INDEX IF NOT EXISTS idx_image_storage_providers_store
    ON image_storage_providers(store_id);
