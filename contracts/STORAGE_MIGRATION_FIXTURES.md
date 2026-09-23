# Registry V1 storage migration fixtures

The tests in `test_storage_migration.rs` seed the canonical `DataKey` and
`ProofRecord`/issuer/root types of an existing V1 instance. They cover the two
supported deployed states: a legacy instance with no `SchemaVersion` key and
an instance initialized with V1. The future-version fixture guards against
accidental downgrade. All digests and addresses are synthetic; no user media,
proof witness, credentials, or production key is captured.

Current `upgrade_storage` targets V1. A missing version reads as V1, so the
upgrade is intentionally a no-op: it preserves proof, video index, consumed
nullifier, issuer, and revoked root, and emits no migration event. Only the
stored admin may invoke it. There is no new schema or public interface and no
change to protocol or artifact version. An older Wasm can read the same V1
records if rollback is required. This fixture does not claim to be an export
of live contract storage or an end-to-end Wasm upgrade; obtain a sanitized,
consented snapshot and pin the old Wasm before asserting that level of coverage.

Run `cargo test -p harpocrates-registry test_storage_migration` from `contracts/`
and build the Wasm with `stellar contract build` when the Rust/Stellar toolchain
is available.
