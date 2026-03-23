CREATE TABLE code_runs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    language    session_language NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX code_runs_session_created_idx ON code_runs(session_id, created_at);
CREATE INDEX code_runs_created_idx ON code_runs(created_at);
CREATE INDEX code_runs_user_created_idx ON code_runs(user_id, created_at);
