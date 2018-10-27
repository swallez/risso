
-- Add a flag to notify posters of replies to their comment
ALTER TABLE comments ADD COLUMN notification INTEGER NOT NULL DEFAULT 0;
