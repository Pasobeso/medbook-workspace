-- Your SQL goes here

CREATE TABLE "carts"(
  id SERIAL PRIMARY KEY,
  patient_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_carts_timestamp
BEFORE UPDATE ON carts
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

CREATE TABLE "cart_items" (
    cart_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (cart_id) REFERENCES carts(id) ON DELETE CASCADE,
    PRIMARY KEY ("cart_id", "product_id")
);

CREATE TRIGGER update_cart_items_timestamp
BEFORE UPDATE ON cart_items
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

CREATE TABLE "orders" (
  "id" serial PRIMARY KEY,
  "cart_id" integer NOT NULL,
  "patient_id" integer NOT NULL,
  "status" text NOT NULL DEFAULT 'PENDING',
  "order_type" text NOT NULL DEFAULT 'PICKUP', -- PICKUP, DELIVERY
  "delivery_address" JSONB,
  "payment_id" UUID,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "deleted_at" TIMESTAMPTZ
);

CREATE TRIGGER update_orders_timestamp
BEFORE UPDATE ON orders
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();
