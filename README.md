# SuiNS Indexer

Sui Indexer using the [sui-indexer-alt](https://github.com/MystenLabs/sui/tree/main/crates/sui-indexer-alt) framework. Also based on [amnn/sui-sender-indexer](https://github.com/amnn/sui-sender-indexer).

Set up database:

```sh
# Make sure you have diesel cli installed:
cargo install diesel_cli --no-default-features --features postgres

diesel setup --migration-dir migrations
diesel migration run --migration-dir migrations
```

Run the indexer (testnet):

```sh
RUST_LOG=info cargo run -- \
  --remote-store-url https://checkpoints.testnet.sui.io \
  --first-checkpoint 207052780 # first time run with `first-checkpoint`
```

Run the indexer (mainnet):

```sh
RUST_LOG=info cargo run -- \
  --remote-store-url https://checkpoints.mainnet.sui.io

# For release
RUST_LOG=info cargo run --release -- \
  --remote-store-url https://checkpoints.mainnet.sui.io
```

Update `watermarks` tables so indexer keeps track of where it left off:
```sql
INSERT INTO watermarks (pipeline, epoch_hi_inclusive, checkpoint_hi_inclusive, tx_hi, timestamp_ms_hi_inclusive, reader_lo, pruner_timestamp, pruner_hi) VALUES ('offer_events', 783, 207052780, 0, 1749029074338, 0, '1970-01-01 00:00:00.000000', 0);
INSERT INTO watermarks (pipeline, epoch_hi_inclusive, checkpoint_hi_inclusive, tx_hi, timestamp_ms_hi_inclusive, reader_lo, pruner_timestamp, pruner_hi) VALUES ('offers', 783, 207052780, 0, 1749029074338, 0, '1970-01-01 00:00:00.000000', 0);
INSERT INTO watermarks (pipeline, epoch_hi_inclusive, checkpoint_hi_inclusive, tx_hi, timestamp_ms_hi_inclusive, reader_lo, pruner_timestamp, pruner_hi) VALUES ('auctions', 783, 207052780, 0, 1749029074338, 0, '1970-01-01 00:00:00.000000', 0);
```
