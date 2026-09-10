-- MasterNote schema v3: add phone + mobile to contacts

ALTER TABLE contacts ADD COLUMN phone TEXT;
ALTER TABLE contacts ADD COLUMN mobile TEXT;
