-- Add migration script here
CREATE TABLE session_participants (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  joined_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  left_at    TIMESTAMPTZ,

  CONSTRAINT session_participants_unique UNIQUE (session_id, user_id)
);

CREATE INDEX session_participants_session_id_idx ON session_participants(session_id);
CREATE INDEX session_participants_user_id_idx ON session_participants(user_id);
