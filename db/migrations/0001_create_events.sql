-- Migration: create events table for persisted envelopes
CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    payload TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
