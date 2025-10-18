-- Your SQL goes here

CREATE TABLE "delivery_addresses" (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER NOT NULL,
    recipient_name VARCHAR(100),
    phone_number VARCHAR(20),
    street_address VARCHAR(255) NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100) DEFAULT 'Thailand',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_delivery_address_timestamp
BEFORE UPDATE ON delivery_addresses
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

CREATE TABLE "deliveries" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delivery_address JSONB NOT NULL,
    order_id INT NOT NULL,
    status VARCHAR(64) NOT NULL DEFAULT 'PREPARING', -- PREPARING, EN_ROUTE, DELIVERED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_delivery_timestamp
BEFORE UPDATE ON deliveries
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

CREATE TABLE "delivery_logs" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delivery_id UUID NOT NULL references deliveries(id) on delete cascade,
    description TEXT NOT NULL,
    status VARCHAR(64) NOT NULL DEFAULT 'PREPARING', -- PREPARING, EN_ROUTE, DELIVERED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_delivery_log_timestamp
BEFORE UPDATE ON delivery_logs
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();
