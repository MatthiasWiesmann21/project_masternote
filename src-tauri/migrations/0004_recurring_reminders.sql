-- Migration v4: Recurring reminders
ALTER TABLE reminders ADD COLUMN recur_interval INTEGER;
ALTER TABLE reminders ADD COLUMN recur_unit TEXT;  -- 'minutes', 'hours', 'days', 'weeks', 'months'
