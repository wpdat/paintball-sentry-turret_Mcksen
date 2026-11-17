# Mackensen Rust Stand — Dokumentation der Portierung Python → Rust

Dokumentiert am 2026-11-16. Prozedur siehe `../cross_process_py_rust.md` (Schritte 1–8).
Fachliche Referenz: `../FunktionalDokument.md` (Abschnitte 2–12).

## 1. Ausgangsstand (Commit 5361e6f, "20260707 Rust implementaion complete v01")

```
rust/src/lib.rs                  Moduldeklaration (trig, common)
rust/src/trig.rs                 trig.py-Port: 9 Funktionen + 9 Unit-Tests
rust/src/common.rs               common.py-Port (nur genutzte Helfer + Konfig-Structs)
rust/src/bin/colour_track.rs     colourTrack.py-Port (Hauptschleife, Arduino-Serial)
rust/src/bin/calibrate_range.rs  calibrateRange.py-Port (6 HSV-Trackbars)
rust/Cargo.toml                  opencv 0.92, serialport 4.3, nalgebra 0.32, chrono 0.4, anyhow, thiserror, log, env_logger
```

## 2. Im Rahmen dieser Portierung angepasste Punkte (aus cross_process_py_rust.md Schritt 6.3)

| Änderung | Datei | Begründung |
|---|---|---|
| VideoWriter implementiert (`prefWriteVideo`): `output/<chrono-timestamp>.avi`, MJPG-fourcc, 20 fps, 640x480, `write()` pro Frame, `release()` beim Cleanup | `src/bin/colour_track.rs` | Doku 9.1/11.1 — war die einzige fehlende Hauptfunktion |
| Trackbar-Initialwerte korrekt: `create_trackbar(..., None, 255, ...)` + `set_trackbar_pos("H+"/"S+"/"V+", 255)` statt temporärer `&mut`-Werte | `src/bin/calibrate_range.rs` | Python: `cv2.setTrackbarPos(...,255)`; temporäre `Some(&mut 255)` wären dangling |
| Toten Code entfernt (`HSV_VALUES`-Static, `get_hsv_values`, `HsvValues`-Struct, ungenutzte `Arc/Mutex`-Imports) | `src/bin/calibrate_range.rs` | erzeugte `dead_code`-Warnungen |
| Unbenutzte Imports entfernt (`CV_8UC3` in colour_track, `Size`/`highgui` in common) | `src/bin/colour_track.rs`, `src/common.rs` | `unused_imports`-Warnungen |
| `est_next_pos(..., _dt)` — Parameter explizit als ignoriert markiert (Python ignoriert `dt` ebenso, fix 0.1 s) | `src/trig.rs` | `unused_variables`-Warnung |
| Zeilenenden CRLF → LF in allen `src/*.rs`, anschließend `cargo fmt` | `src/**/*.rs` | einheitliches Format, `cargo fmt --check` sauber |
| `.gitignore` neu (Repo-Root): `/rust/target/`, `/output/`, `__pycache__/`, `*.pyc` | `../.gitignore` | `rust/target/` war in Commit 5361e6f eingecheckt (Commit-Aufblähung) |

## 3. Modul-Mapping (Ist-Stand nach Portierung)

| Python | Rust | Status |
|---|---|---|
| `trig.py` (9 Funktionen) | `src/trig.rs` | vollständig, 9 Unit-Tests |
| `common.py` (genutzte Teile) | `src/common.rs` | vollständig (`Bunch`, `StatValue`, `Timer`, `clock`, `draw_str`, `getsize`, `find_largest_contour`, `HsvRange`, `Preferences`, `ArduinoConfig`) |
| `common.py` (ungenutzte Helfer: `Sketcher`, `RectSelector`, `mosaic`, `make_cmap`, `lookat`, …) | — | bewusst nicht portiert (YAGNI, keine Aufrufer) |
| `colourTrack.py` | `src/bin/colour_track.rs` | vollständig inkl. VideoWriter, Arduino `p<pan>\n`/`t<tilt>\n` auf `/dev/ttyACM0` @ 115200 |
| `calibrateRange.py` | `src/bin/calibrate_range.rs` | vollständig, 6 Trackbars, ESC-Exit, HSV-Werte-Ausgabe |

Konstanten (Doku Abschnitt 10–11) liegen zentral als `pub const` in `trig.rs` bzw. Struct-Defaults in `common.rs`.

## 4. Verifikation (durchgeführt am 2026-11-16 auf diesem Rechner)

| Test | Ergebnis |
|---|---|
| `cargo fetch` | OK — alle Crates aus Cargo.lock auflösbar/herunterladbar |
| `cargo fmt` | OK — alle Quellen formatiert |
| `rustc --edition 2021 --crate-type lib --emit=metadata src/trig.rs` | OK — fehlerfrei kompiliert (vollständiger Parse/Type-/Borrow-Check, ohne Linken) |
| `rustc --edition 2021 --test --emit=metadata src/trig.rs` | OK — lib + alle 9 Unit-Tests type-geprüft, keine Warnungen |
| Äquivalenztest der trig-Kette (1:1 aus `trig.rs` übertragen) gegen Referenzwerte aus FunktionalDokument 5.1/10.2/11.4/12.1 | 14/14 PASS (Zentrum→(0,0), Ecken→(±24°,∓18°), z=5 m, Offsets, Servo-Zentrum 538/488, Bereichsüberschreitung→(538,488), velocity/prädiktion, lead==no-lead bei Stillstand) |
| `cargo check` / `cargo build` (vollständig) | **in dieser Umgebung nicht ausführbar** — siehe Abschnitt 5 |

## 5. Build auf dem Zielsystem (Laptop/Raspberry Pi mit OpenCV)

Diese Maschine hat **kein Windows SDK** (keine `kernel32.lib`; `link.exe`/`rust-lld` können ohne SDK-Importlibs nicht linken) und **keine OpenCV-C++-Installation** (das `opencv`-Crate braucht System-OpenCV + libclang). Daher schlägt `cargo check` hier bereits beim Linken der Build-Scripts fehl (`link: extra operand … rcgu.o` → danach fehlende SDK-Libs) — ein Umgebungsproblem, kein Codefehler.

Auf einem System mit den Voraussetzungen (siehe `rust/README.md` bzw. `scripts/install_deps.sh`: `libopencv-dev`, `clang`/`libclang-dev`, `pkg-config`, `libudev-dev`, …) ist der Build:

```bash
cd rust
./scripts/install_deps.sh     # Systempakete
cargo fetch                   # Crates laden
cargo test                    # Unit-Tests (trig: 9, common: 6)
cargo build --release         # Binaries
cargo run --bin calibrate_range   # HSV kalibrieren
cargo run --bin colour_track      # Tracking (ESC beendet)
```

Cross-Compile Raspberry Pi: `rustup target add aarch64-unknown-linux-gnu && cargo build --release --target aarch64-unknown-linux-gnu`.

## 6. Bekannte, dokumentierte Abweichungen zum Python-Original

1. `est_next_pos` ignoriert `dt` und nutzt fix 0.1 s — wie das Python-Original (bewusst 1:1 übernommen, Parameter jetzt `_dt`).
2. `compute_velocity` mit `dt <= 0` liefert Null-Vektor statt DivisionByZero (robuster).
3. Trackbar-Werte werden in `calibrate_range` per `get_trackbar_pos` gepollt statt per Callback (Limitation der opencv-Rust-API, funktional gleich).
4. Kamera-Device: Versuch auf Device 1, Fallback auf Device 0 (Python: fest Device 1).
5. Python-`locals()`-Prüfungen durch `Option<>`-State ersetzt; `print`-Warnungen → `eprintln!`.
