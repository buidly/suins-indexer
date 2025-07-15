CREATE TYPE AuctionStatus AS ENUM (
    'created',
    'cancelled',
    'finalized'
);

CREATE TABLE IF NOT EXISTS auctions (
    auction_id VARCHAR NOT NULL PRIMARY KEY,
    domain_name VARCHAR NOT NULL,
    owner VARCHAR NOT NULL,
    start_time BIGINT NOT NULL,
    end_time BIGINT NOT NULL,
    min_bid VARCHAR NOT NULL,
    winner VARCHAR,
    amount VARCHAR,
    status AuctionStatus NOT NULL DEFAULT 'created',
    updated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    last_tx_digest VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS bids (
    id SERIAL PRIMARY KEY,
    auction_id VARCHAR NOT NULL REFERENCES auctions(auction_id),
    domain_name VARCHAR NOT NULL,
    bidder VARCHAR NOT NULL,
    amount VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    tx_digest VARCHAR NOT NULL
);
