-- Your SQL goes here
CREATE TABLE "outbox" (
  "id" serial PRIMARY KEY,
  "event_type" text NOT NULL,
  "payload" text NOT NULL,
  "status" text NOT NULL DEFAULT 'PENDING'
);
