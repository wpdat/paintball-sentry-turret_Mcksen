
---
paths:
  - "rust/src/**/*.rs"
  - "rust/Cargo.toml"
---
# Rust-Style Guide
- Verwende snake\_case für Variablennamen und Funktionen.
- Verwende CamelCase für Typen.
- Verwende die Standardbibliothek, wenn möglich.
- Verwende etablierte Crates wie serde, tokio, clap.
- Vermeide unnotwendige Abhängigkeiten.
- Schreibe klare und dokumentierte Code.
- Edition 2021, `rustfmt`-Konventionen, `#![deny(warnings)]` in [lib.rs](http://lib.rs) vermeiden (CI entscheidet).
- Öffentliche APIs dokumentieren (`///`), Beispiele sparsam.
- Fehlerbehandlung: `thiserror` für Libs, `anyhow` nur in Binaries.
- Kein `unsafe`, außer mit SAFETY-Kommentar und Begründung.
- Tests als `#[cfg(test)]`-Modul je Datei; `cargo test` muss vor jedem Commit grün sein.
- Ressourcenschonend: keine unnötigen Klone, prefer Borrowing; memory-mapped I/O nur wo nötig (Ziel: kleine ARM-Boards).