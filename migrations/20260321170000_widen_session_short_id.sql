-- Meet-style codes are `xxx-xxx-xxx` (11 chars); allow headroom for the column.
ALTER TABLE sessions
  ALTER COLUMN short_id TYPE VARCHAR(32);
