CREATE TABLE users (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  display_name VARCHAR(100) NOT NULL,
  color        VARCHAR(64) NOT NULL DEFAULT 'blue',
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
