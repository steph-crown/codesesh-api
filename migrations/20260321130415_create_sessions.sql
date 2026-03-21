-- Add migration script here
CREATE TYPE session_visibility AS ENUM ('private', 'view_only', 'edit');
CREATE TYPE session_status AS ENUM ('active', 'ended');

CREATE TABLE sessions (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  short_id         VARCHAR(10) NOT NULL,
  host_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name             VARCHAR(255) NOT NULL,
  language         VARCHAR(50) NOT NULL DEFAULT 'typescript',
  visibility       session_visibility NOT NULL DEFAULT 'edit',
  status           session_status NOT NULL DEFAULT 'active',
  content          TEXT NOT NULL DEFAULT '',
  last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  event_count      INTEGER NOT NULL DEFAULT 0,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT sessions_short_id_key UNIQUE (short_id)
);

CREATE INDEX sessions_host_id_idx ON sessions(host_id);
CREATE INDEX sessions_status_idx ON sessions(status);
CREATE INDEX sessions_host_status_idx ON sessions(host_id, status);
