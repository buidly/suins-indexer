-- Your SQL goes here

-- Complete schema for the SuiNS indexer

-- Events cursor table to track processed events
CREATE TABLE IF NOT EXISTS events_cursor (
    id SERIAL PRIMARY KEY,
    checkpoint VARCHAR NOT NULL,
    tx_digest VARCHAR NOT NULL,
    CONSTRAINT events_cursor_checkpoint_unique UNIQUE (checkpoint)
);

-- Offer placed events table
CREATE TABLE IF NOT EXISTS offer_placed (
    id SERIAL PRIMARY KEY,
    domain_name VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tx_digest VARCHAR NOT NULL
);

-- Offer cancelled events table
CREATE TABLE IF NOT EXISTS offer_cancelled (
    id SERIAL PRIMARY KEY,
    domain_name VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tx_digest VARCHAR NOT NULL
);
