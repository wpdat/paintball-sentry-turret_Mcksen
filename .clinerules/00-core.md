# KIHR5 Kernregeln
- Rust-Projekt; Zielplattformen: Raspberry Pi 5 (aarch64) und NVIDIA Jetson (aarch64).
- Erst lesen (Cargo.toml, bestehende Module), dann ändern. Keine neuen Abhängigkeiten ohne Not; bevorzuge std + etablierte Crates (serde, tokio, clap).
- Fehler nie umschreiben/unterdrücken – Ursache fixen.
- Kurze, gezielte Aufgaben; keine Riesen-Refactorings in einem Task.
- Commits: Conventional Commits, eine logische Änderung pro Commit.
- start a new task when the context window is 50% full
- Nach abgeschlossener Teilaufgabe vorschlagen, den Task abzuschließen.