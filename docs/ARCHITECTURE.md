
---

## Шаг 6 — Крейт `core`

Создаём `crates/core/`. Файлы:

### `crates/core/Cargo.toml`

```toml
[package]
name = "loreforge-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
thiserror.workspace = true
unicode-normalization.workspace = true
blake3.workspace = true