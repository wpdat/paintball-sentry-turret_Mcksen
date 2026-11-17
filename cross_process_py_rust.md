# Cross-Prozess: Portierung Python → Rust

Prozedur zur Portierung des Python-Codes des Paintball Sentry Turret Projekts nach Rust.
Erstellt am 2026-11-16. Quelle der fachlichen Anforderungen: `FunktionalDokument.md`.

---

## Schritt 1: Ausgangsbasis, Zielsetzung und Methode

### 1.1 Ausgangslage (Python-Seite)

Laut `FunktionalDokument.md` (Abschnitte 2, 3, 5) besteht das Projekt aus vier Python-2.x-Modulen:

| Python-Datei | Rolle | Zeilen | Kerninhalt |
|---|---|---|---|
| `trig.py` | Koordinatentransformationen | 157 | Pixel → Winkel → kartesisch → Geschütz-Polar → Servo-Signal; optional Geschwindigkeitsprädiktion |
| `common.py` | Hilfsfunktionen/OpenCV-Utilities | 217 | Klassen `Bunch`, `Sketcher`, `StatValue`, `RectSelector` sowie Zeichen-/Timer-/Mathe-Helfer |
| `colourTrack.py` | Hauptprogramm | 95 | Webcam-Capture → HSV-Maske → Konturen → Schwerpunkt → `trig.camToGun()` → seriell an Arduino (`p<pan>\n`, `t<tilt>\n`) |
| `calibrateRange.py` | Kalibrierungs-GUI | 65 | Trackbars für HSV-Bereich, Live-Vorschau der Maske |

Abhängigkeiten (Abschnitt 3): `cv2` (OpenCV), `numpy`, `serial` (pyserial), `math`, `time`, `os`, `contextlib`, `itertools`.
Hardware-Schnittstellen (Abschnitt 4): Webcam 640x480, Arduino seriell `/dev/ttyACM0` @ 115200 Baud, Pan/Tilt-Servos.

### 1.2 Ausgangslage (Rust-Seite, bereits vorhanden)

Unter `rust/` existiert bereits eine Rust-Implementierung (Commit `5361e6f`, "Rust implementaion complete v01"):

```
rust/src/lib.rs                  Deklariert Module `trig` und `common`
rust/src/trig.rs                 Port von trig.py (alle 9 Funktionen + Tests)
rust/src/common.rs               Port der tatsächlich genutzten common.py-Teile (Bunch, StatValue, Timer, clock, draw_str, HsvRange, Preferences, ArduinoConfig, find_largest_contour)
rust/src/bin/colour_track.rs     Port von colourTrack.py (Hauptschleife, Capture, Maske, Konturen, Arduino-Serial)
rust/src/bin/calibrate_range.rs  Port von calibrateRange.py (Trackbars, HSV-Maske, Live-Anzeige)
```

### 1.3 Zielsetzung

Eine nachvollziehbare, schrittweise Prozedur, mit der der Python-Code vollständig und funktional äquivalent nach Rust portiert wird — inklusive Abgleich, was bereits portiert ist, was fehlt, und in welcher Reihenfolge vorgegangen wird.

### 1.4 Methode

- **Bottom-up**: zuerst die reinen Mathe-Module ohne GUI-Abhängigkeit (`trig`), dann die Hilfsfunktionen (`common`), dann die beiden Anwendungen (`colourTrack`, `calibrateRange`) — entsprechend der Aufrufhierarchie in Abschnitt 6 des FunktionalDokuments.
- **Funktionale Äquivalenz**: gleiche Eingaben müssen gleiche Ausgaben liefern (gleiche Pixelkoordinaten → gleiche Servo-Werte). Interne Struktur (Ownership, `Option`, `Result`) folgt Rust-Konventionen.
- **Konstanten-Treue**: alle im FunktionalDokument Abschnitt 11 definierten Parameter (FOV, Offsets, HSV-Default, Servo-Grenzen, serielles Protokoll) werden 1:1 übernommen.
- **Nachweis**: pro Modul Unit-Tests gegen Referenzwerte aus der Python-Logik; am Ende `cargo test` und `cargo build --release`.


---

## Schritt 2: Modul-Mapping Python → Rust

Jedes Python-Modul wird einem Rust-Gegenstück zugeordnet. Status: `vollständig` / `teilweise` / `offen`.

| Python-Modul | Rust-Gegenstück | Status | Anmerkung |
|---|---|---|---|
| `trig.py` (9 Funktionen) | `rust/src/trig.rs` | vollständig | Alle Funktionen portiert: `px_to_angle`, `cam_polar_cart_z`, `cam_polar_cart_h`, `shift_cart`, `gun_cart_to_polar`, `signal_from_polar`, `compute_velocity`, `est_next_pos`, `cam_to_gun` + 9 Unit-Tests |
| `common.py` (4 Klassen, 17 Funktionen) | `rust/src/common.rs` | teilweise | Portiert und von den Anwendungen tatsächlich genutzt: `Bunch`, `StatValue`, `Timer`, `clock()`, `draw_str()`, `getsize()`, `find_largest_contour()` sowie Konfig-Structs `HsvRange`, `Preferences`, `ArduinoConfig`. Offen: `Sketcher`, `RectSelector`, `make_cmap`, `mosaic`, `grouper`, `mdot`, `lookat`, `homotrans`, `rect2rect_mtx`, `mtx2rvec`, `anorm/anorm2`, `to_rect`, `draw_keypoints`, `nothing`, `splitfn` (siehe Schritt 6) |
| `colourTrack.py` (Hauptprogramm) | `rust/src/bin/colour_track.rs` | teilweise | Hauptschleife vollständig portiert (Capture → blur → HSV → inRange → findContours → größte Kontur → Moments → `cam_to_gun` → Arduino `p…\n`/`t…\n` → Overlay → ESC). Offen: `prefWriteVideo` (VideoWriter/Aufzeichnung) nicht umgesetzt (siehe Schritt 6) |
| `calibrateRange.py` (Kalibrier-GUI) | `rust/src/bin/calibrate_range.rs` | vollständig | 6 Trackbars (H-/H+/S-/S+/V-/V+), Live-Maske, Konturen, Flächenanzeige, Ausgabe der finalen HSV-Werte; ESC-Exit |

### 2.1 Sprachkonstrukt-Mapping (Python 2 → Rust)

| Python-Konstrukt | Rust-Äquivalent | Wo verwendet |
|---|---|---|
| `print "..."` (Statement) | `println!` / `eprintln!` | Warnungen in `signal_from_polar` |
| `tuple` Rückgabe | `(f64, f64)` / `(i32, i32)` Tupel oder `struct Position`/`Velocity` | `trig.rs` |
| `None` / optional args | `Option<f64>` | `cam_to_gun(cx, cy, last_cx: Option<f64>, …)` |
| `'best_cnt' in locals()` | `Option<ContourResult>` (`if let Some(...)`) | `colour_track.rs` |
| `round(x, 3)` | `(x * 1000.0).round() / 1000.0` | `compute_velocity` |
| `with Timer(msg):` (contextmanager) | `struct Timer` mit `impl Drop` | `common.rs` |
| Klassen-Attribute (`self.__dict__.update`) | `struct` mit pub-Feldern | `Bunch`, `StatValue`, `Preferences` |
| `while(1):` + `break` | `loop { … break; }` | Hauptschleifen |
| `cv2.waitKey(33)==27` | `highgui::wait_key(33)? == 27` | ESC-Erkennung |
| `ser.write("p"+str(pan)+"\n")` | `port.write_all(format!("p{}\n", pan).as_bytes())` | Arduino-Protokoll (Abschnitt 10.1) |


---

## Schritt 3: Dependency-Mapping (Python-Bibliotheken → Rust-Crates)

Abgleich zwischen Abschnitt 3 des FunktionalDokuments und den in `rust/Cargo.toml` deklarierten Abhängigkeiten.

| Python-Bibliothek | Zweck (laut Doku) | Rust-Crate | In Cargo.toml | Status |
|---|---|---|---|---|
| `cv2` (OpenCV) | Bildverarbeitung, Video-Capture, GUI | `opencv = "0.92"` (Features: `clang-runtime`, `videoio`, `imgproc`, `highgui`) | ja | portiert (`cvt_color`, `in_range`, `find_contours`, `moments`, `contour_area`, `blur`, `VideoCapture`, `imshow`, `wait_key`, `create_trackbar`, `put_text`, `circle`, `draw_contours`) |
| `numpy` | Arrays, Matrix-Operationen | `nalgebra = "0.32"` | ja | nur für optionale Mathe-Helfer (voir Schritt 6); Kernlogik nutzt `f64`/Tupel |
| `serial` (pyserial) | Arduino-Serial `/dev/ttyACM0` @ 115200 | `serialport = "4.3"` (default-features = false) | ja | portiert in `ArduinoSerial` (`colour_track.rs`) |
| `math` | `tan`, `atan2`, `radians`, `degrees` | `std::f64` (inherent methods) + `deg_to_rad`/`rad_to_deg` in `trig.rs` | — (stdlib) | portiert |
| `time` (`strftime`, `sleep`) | Zeitstempel, Video-Dateiname | `chrono = "0.4"`; Zeitmessung via `Instant` in `common::clock()` | ja | `clock()` portiert; `strftime` für Video-Dateinamen offen (nur bei VideoWriter nötig) |
| `os` | `os.path.split/splitext` | `std::path::Path` bzw. `splitfn()` in `common.rs` | — (stdlib) | rudimentär portiert |
| `contextlib` (`@contextmanager`) | `Timer` | `impl Drop for Timer` | — (stdlib) | portiert |
| `itertools` (`izip_longest`, `chain`) | `grouper`, `mosaic` | `itertools`-Crate oder std-Iteratoren | nein | offen, nur für `mosaic`/`grouper` nötig (nicht vom Hauptprogramm verwendet) |
| — | Fehlerbehandlung | `anyhow = "1.0"`, `thiserror = "1.0"` | ja | `Result<()>` in `main()`, `?` statt Exceptions |
| — | Logging | `log = "0.4"`, `env_logger = "0.11"` | ja | `env_logger::init()` in `colour_track.rs` |

### 3.1 Systemabhängigkeiten (nicht pip/cargo)

Laut `rust/README.md` / `rust/scripts/install_deps.sh` werden benötigt: `libopencv-dev`, `clang`/`libclang-dev` (für opencv-Crate-Bindings), `pkg-config`, `cmake`, `build-essential`, `libv4l-dev`, `libgtk-3-dev` (highgui), `libudev-dev` (serialport). Rust-Toolchain via rustup.


---

## Schritt 4: Portierungsreihenfolge und Detailabgleich `trig.py` → `trig.rs`

### 4.1 Reihenfolge (begründet über die Aufrufhierarchie, Doku Abschnitt 6)

```
Schritt 4a: trig.py    → trig.rs    (keine CV-Abhängigkeit, reine Mathematik — zuerst testbar)
Schritt 4b: common.py  → common.rs  (OpenCV-Helfer, die die Anwendungen importieren)
Schritt 5:  colourTrack.py → bin/colour_track.rs  (Hauptprogramm, nutzt trig + common)
Schritt 5:  calibrateRange.py → bin/calibrate_range.rs (GUI-Tool, nutzt common)
Schritt 6:  Lücken schließen + Verifikation
```

### 4.2 Funktion-für-Funktion-Abgleich `trig.py` (Doku Abschnitt 5.1) vs. `rust/src/trig.rs`

| Python-Funktion | Rust-Funktion | Äquivalenz | Abweichung / Hinweis |
|---|---|---|---|
| `pxToAngle(x, y)` | `px_to_angle(x: f64, y: f64) -> (f64, f64)` | ja | Formel identisch: `radians(0.075*x - 24)`, `radians(-0.075*y + 18)`; Konstanten als `pub const` (Doku 11.2: FOV 48°/36°) |
| `camPolarCartZ(theta, phi)` | `cam_polar_cart_z` | ja | `z = 5.0` als `DEFAULT_RANGE_Z` (Doku 11.4) |
| `camPolarCartH(theta, phi, h)` | `cam_polar_cart_h` | ja | `objectHeight = 0.54` als `OBJECT_HEIGHT` (Doku 11.4) |
| `shiftCart(x, y, z)` | `shift_cart` | ja | Offsets a=-0.11, b=-0.26, c=0.15 als `GUN_OFFSET_*` (Doku 11.4) |
| `gunCartToPolar(x, y, z)` | `gun_cart_to_polar` | ja | `atan2(x, z)`, `atan2(y, z)` — Rust `f64::atan2(y, x)` mit vertauschten Args |
| `signalFromPolar(theta, phi)` | `signal_from_polar -> (i32, i32)` | ja | `-degrees(θ)/0.325 + 538` bzw. `+ 488`; Bereichsprüfung Pan 251–805, Tilt 211–765 (Doku 10.2, 12.1) → bei Überschreitung `eprintln!` + Rückgabe `(538, 488)`; Rundung via `.round()` statt `int(round())` |
| `computeVelocity(gunCart, lastGunCart, dt)` | `compute_velocity(Position, Position, f64) -> Velocity` | ja | `round(Δ/dt, 3)` → `(Δ/dt * 1000).round()/1000`; zusätzlich Guard `dt <= 0` → Null-Vektor (Python hätte DivisionByZero) |
| `estNextPos(gunCart, v, dt)` | `est_next_pos(Position, Velocity, f64) -> Position` | ja | Python ignoriert sein `dt`-Argument und nutzt fix `dt = 0.1` — im Rust-Port ebenso festkodiert (`prediction_dt = 0.1`), Verhalten identisch |
| `camToGun(cx, cy, lastCx, lastCy, dt)` | `cam_to_gun(cx, cy, last_cx: Option<f64>, last_cy: Option<f64>, dt: Option<f64>)` | ja | Kette pxToAngle → camPolarCartZ → shiftCart → [optional computeVelocity → estNextPos] → gunCartToPolar → signalFromPolar (Doku 9.2). Toter Python-Code (`before = …`, auskommentierter Block) entfällt |

### 4.3 Konventionen im trig-Modul

- Rückgabe von 3D-Punkten als Tupel `(f64, f64, f64)`; für Velocity/Position zusätzlich typisierte Structs `Position`/`Velocity` (bessere Lesbarkeit, `#[derive(Debug, Clone, Copy, Default)]`).
- Alle Konstanten aus Doku Abschnitt 11.2–11.4 als `pub const` oben im Modul — zentrale, dokumentierte Parameterquelle.
- 9 Unit-Tests vorhanden (Center/Corners, Bereichsprüfung, Prädiktion) — siehe Schritt 7.


---

## Schritt 5: Detailabgleich der Anwendungen (`common.py`, `colourTrack.py`, `calibrateRange.py`)

### 5.1 `common.py` → `rust/src/common.rs` (Doku Abschnitt 5.2)

| Python | Rust | Status | Anmerkung |
|---|---|---|---|
| `class Bunch` | `struct Bunch { data: HashMap<String,String> }` | portiert | `set`/`get` statt `__dict__` |
| `class StatValue(smooth_coef=0.5)` + `update(v)` | `struct StatValue { value: Option<f64>, smooth_coef }` | portiert | Glättung `c*alt + (1-c)*neu` identisch |
| `Timer(msg)` context manager | `struct Timer` + `impl Drop` (druckt ms beim Drop) | portiert | `clock()` via `std::time::Instant` (OnceLock-Start) statt `cv2.getTickCount` |
| `draw_str(dst, (x,y), s)` | `draw_str(&mut Mat, x, y, s) -> OpenCvResult<()>` | portiert | Schwarzer Schatten + weißer Text via `imgproc::put_text` (FONT_HERSHEY_PLAIN) |
| `getsize(img)` | `getsize(&Mat) -> (i32, i32)` | portiert | |
| — (neu, aus Doku 11.1/11.3/10.1) | `struct Preferences`, `struct HsvRange` (Default 10–40 / 70–255 / 150–255), `struct ArduinoConfig` (`/dev/ttyACM0`, 115200) | portiert | Ersetzt die globalen `prefArduino/prefScreen/prefTargetLead/prefWriteVideo`-Variablen typisiert |
| — (aus colourTrack/calibrateRange extrahiert) | `find_largest_contour(&Vector<Vector<Point>>) -> Option<ContourResult>` | portiert | kapselt max-area-Suche + `moments` + Schwerpunkt; ersetzt `'best_cnt' in locals()` |
| `Sketcher`, `RectSelector` | — | offen | Maus-Interaktions-Tools; werden von keinem der beiden Programme benutzt → nur bei Bedarf portieren (Schritt 6) |
| `splitfn`, `anorm2/anorm`, `homotrans`, `to_rect`, `rect2rect_mtx`, `lookat`, `mtx2rvec`, `make_cmap`, `nothing`, `grouper`, `mosaic`, `mdot`, `draw_keypoints` | — (`splitfn` rudimentär vorhanden) | offen | reine Hilfsfunktionen ohne Aufrufer im Projekt (grep: nur Definitionen in common.py) → YAGNI, optional mit `nalgebra` umsetzbar |

### 5.2 `colourTrack.py` → `rust/src/bin/colour_track.rs` (Doku Abschnitte 5.3, 8, 9.1)

Hauptschleife 1:1 übertragen (Doku 9.1 Kontrollfluss):

```
cap.read() → blur(3x3) → cvtColor(BGR2HSV) → inRange(HSV-Default) → findContours(RETR_LIST, CHAIN_APPROX_SIMPLE)
→ find_largest_contour() → circle(center) → [prefTargetLead && last_time: dt = clock()-t → cam_to_gun(cx,cy,lastCx,lastCy,dt)
                                          sonst: cam_to_gun(cx,cy)]
→ state update (t, lastCx, lastCy, pan, tilt) → [prefArduino: "p{pan}\n", "t{tilt}\n"]
→ [prefScreen: drawContours grün (0,255,0) Dicke 3, Overlay time/area/pan/tilt via draw_str, imshow("frame")]
→ wait_key(33) == 27 → break → destroyAllWindows, cap.release
```

Abweichungen:
- `prefWriteVideo` / `cv2.VideoWriter('output/<timestamp>.avi', MJPG, 20, 640x480)` ist im Rust-Port **nicht umgesetzt** (Doku 11.1: `write_video_enabled: false` Default; VideoWriter würde `videoio::VideoWriter` + `chrono::Local` für den Dateinamen brauchen) → offen, siehe Schritt 6.
- Python-`locals()`-Tricks (`'t' in locals()`, `'pan' in locals()`) werden durch `Option<f64>`/`Option<i32>`-State sauber ersetzt.
- Kamera-Device: Python nutzt fest `VideoCapture(1)`; Rust versucht Device 1, fällt auf Device 0 zurück.
- Arduino-Serial: `serialport::new(port, 115200).timeout(100ms)`; Befehle als `p<value>\n` / `t<value>\n` (Doku 10.1) — identisch.

### 5.3 `calibrateRange.py` → `rust/src/bin/calibrate_range.rs` (Doku Abschnitt 5.4, 7.2)

- 6 Trackbars `H-/H+/S-/S+/V-/V+` (0–255, H+/S+/V+ initial 255) über `highgui::create_trackbar`; Werte werden im Loop per `get_trackbar_pos` gepollt (Rust-Callbacks sind eingeschränkt — dokumentierte Abweichung zum Python-Callback-Modell, funktional gleich).
- Schleife: read → blur → HSV → inRange(trackbar-Werte) → findContours → größte Kontur + Schwerpunkt-Kreis → drawContours + `area: … px` Overlay + HSV-Werte-Overlay → imshow → ESC.
- Beim Beenden werden die finalen HSV-Werte auf der Konsole ausgegeben (in Python manuell ablesbar) — praktisch für die Konfiguration von `HsvRange`.


---

## Schritt 6: Konstanten-, Protokoll- und Fehlerbehandlungs-Abgleich; offene Punkte

### 6.1 Konstanten (Doku Abschnitt 11) — alle in `trig.rs`/`common.rs` als `pub const` bzw. Struct-Defaults hinterlegt

| Parameter | Python-Quelle | Wert | Rust-Entsprechung |
|---|---|---|---|
| Auflösung | Doku 11.2 | 640 x 480 | `CAMERA_RESOLUTION_X/Y` |
| FOV | Doku 11.2 | 60° gesamt, X 48°, Y 36° | `CAMERA_FOV_X/Y` (Faktor 0.075°/px in `px_to_angle`) |
| Geschätzte Entfernung z | Doku 11.4 | 5 m | `DEFAULT_RANGE_Z` |
| Objekthöhe | Doku 11.4 | 0.54 m | `OBJECT_HEIGHT` |
| Kamera→Geschütz-Offset a,b,c | Doku 11.4 | -0.11, -0.26, 0.15 m | `GUN_OFFSET_X/Y/Z` |
| Servo-Faktor | Doku 10.2 | 0.325 °/Schritt | `SERVO_DEGREE_FACTOR` |
| Pan/Tilt Zentrum | Doku 10.2 | 538 / 488 | `PAN_CENTER` / `TILT_CENTER` |
| Pan/Tilt Grenzen | Doku 10.2 | 251–805 / 211–765 | `PAN_MIN/MAX`, `TILT_MIN/MAX` |
| HSV-Default | Doku 11.3 | H 10–40, S 70–255, V 150–255 | `HsvRange::default_orange()` |
| Preferences | Doku 11.1 | arduino=0, screen=1, lead=1, video=0 | `Preferences::new()` |
| Serial | Doku 10.1 | `/dev/ttyACM0`, 115200 Baud | `ArduinoConfig::new()` |
| Protokoll | Doku 10.1 | `p<pan>\n`, `t<tilt>\n` | `format!("p{}\n", pan)` in `ArduinoSerial::send_pan/tilt` |

### 6.2 Fehlerbehandlung (Doku Abschnitt 12)

- `signalFromPolar`: Bereichsverletzung → Warnung + Rückgabe Zentrum (538, 488) — in Rust identisch via `eprintln!` + `return (PAN_CENTER, TILT_CENTER)` (Tests: `test_signal_from_polar_bounds`).
- `compute_velocity` mit `dt <= 0`: Python käme DivisionByZero; Rust gibt Null-Vektor zurück (robuster, dokumentierte Abweichung).
- `find_largest_contour`: `m00 == 0` oder keine Kontur → `None` (Python: `'best_cnt' in locals()`-Check) — äquivalent.
- OpenCV-Aufrufe liefern `Result` und werden per `?` propagiert; `main() -> anyhow::Result<()>` ersetzt Pythons ungeprüfte Exceptions.

### 6.3 Offene Punkte (Lücken zwischen Python und vorhandenem Rust-Port)

1. **VideoWriter (`prefWriteVideo`)** in `colour_track.rs` fehlt. Umsetzung: `videoio::VideoWriter::new("output/<chrono-timestamp>.avi", fourcc MJPG, 20.0, Size(640,480))`, `writer.write(&frame)` pro Frame, `writer.release()` beim Cleanup; `Preferences.write_video_enabled` auf `true` schalten. `chrono` ist bereits als Dependency vorhanden.
2. **Nicht portierte `common.py`-Helfer** (`Sketcher`, `RectSelector`, `make_cmap`, `mosaic`, `grouper`, `mdot`, `lookat`, `homotrans`, `rect2rect_mtx`, `mtx2rvec`, `anorm`, `to_rect`, `draw_keypoints`, `nothing`): keine Aufrufer in `colourTrack.py`/`calibrateRange.py` → nach YAGNI zurückgestellt; bei Bedarf mit `nalgebra` (bereits in Cargo.toml) umsetzen.
3. **Trackbar-Callbacks** in `calibrate_range.rs`: Python nutzt `nothing`-Callback + `setTrackbarPos`; Rust pollt via `get_trackbar_pos` — funktional gleich, initial werden H+/S+/V+ auf 255 gesetzt (im Rust-Code über `create_trackbar` mit initial value bzw. `set_trackbar_pos` sicherstellen).
4. **`rust/target/` ist eingecheckt** (Commit 5361e6f) — empf. `.gitignore` mit `/rust/target/` ergänzen, damit künftige Commits nicht aufgebläht werden (Änderung am Repo nur nach Freigabe, hier nur dokumentiert).


---

## Schritt 7: Test- und Verifikationsstrategie

Ziel: Nachweis der funktionalen Äquivalenz Python ↔ Rust (gleiche Eingabe → gleiche Ausgabe).

### 7.1 Unit-Tests (bereits vorhanden in `rust/src/trig.rs` und `rust/src/common.rs`)

| Test | prüft | Referenz |
|---|---|---|
| `test_px_to_angle_center` | `px_to_angle(320,240) ≈ (0,0)` | Doku 5.1: Zentrum → (0,0) |
| `test_px_to_angle_corners` | `(0,0) → (-24°,18°)`, `(640,480) → (24°,-18°)` | Doku 5.1 Kommentar |
| `test_cam_polar_cart_z` | `z = 5` bei θ=φ=0 | Doku 11.4 |
| `test_shift_cart` | Offsets a,b,c aufaddiert | Doku 11.4 |
| `test_gun_cart_to_polar` | x=y=0 → θ=φ=0 | Logik |
| `test_signal_from_polar_center` | θ=φ=0 → (538, 488) | Doku 10.2 Zentrum |
| `test_signal_from_polar_bounds` | θ=φ=10 rad → außerhalb → (538, 488) | Doku 12.1 |
| `test_cam_to_gun_center` | Zentrum → pan/tilt nahe 538/488 (±50) | Integration |
| `test_compute_velocity` / `test_est_next_pos` | v = Δpos/dt; pos + v·0.1 | Doku 5.1 |
| `test_stat_value`, `test_hsv_range`, `test_preferences`, `test_arduino_config`, `test_clock`, `test_bunch` | common.rs-Helfer | Doku 11 |

### 7.2 Erweiterte Äquivalenztests (empfohlen als Nächstes)

1. **Referenzvektor-Test**: Für ein Raster von Pixelkoordinaten (z. B. (0,0), (320,240), (640,480), (100,300), (500,50)) die Python-Funktion `trig.camToGun(cx, cy)` ausführen und die (pan, tilt)-Paare als JSON exportieren; im Rust-Test (`tests/equivalence.rs`) gegen `cam_to_gun` prüfen — Toleranz ±1 Servo-Schritt (Rundung).
2. **Prädiktionspfad**: Gleiches Raster mit `lastCx/lastCy/dt` (z. B. dt=0.033) vergleichen — prüft `compute_velocity` + `est_next_pos` Kette (Doku 9.2).
3. **Bereichsgrenzen**: Pixel, die pan/tilt außerhalb 251–805 / 211–765 treiben → Erwartung (538, 488) in beiden Implementierungen (Doku 12.1).

### 7.3 Build- und Laufzeit-Verifikation

| Schritt | Kommando | Erwartung |
|---|---|---|
| Abhängigkeiten | `rust/scripts/install_deps.sh` | OpenCV/clang/pkg-config installiert |
| Unit-Tests | `cargo test` (bzw. `rust/scripts/test.sh`) | alle Tests grün |
| Debug-Build | `cargo build` (`rust/scripts/build.sh`) | kompiliert ohne Fehler |
| Release-Build | `cargo build --release` | kompiliert; `opt-level=3`, `lto=true` (Cargo.toml) |
| Lint/Format | `cargo clippy`, `cargo fmt --check` | keine Warnungen/Formatfehler |
| Kalibrier-Tool | `cargo run --bin calibrate_range` | Fenster mit 6 Trackbars, ESC beendet, HSV-Werte auf Konsole |
| Hauptprogramm (ohne Hardware) | `cargo run --bin colour_track` mit `screen_enabled=false` bzw. in Headless-Umgebung nur bis Capture-Fehler | bricht sauber mit Fehlermeldung ab, wenn keine Kamera (erwartetes Verhalten ohne Hardware) |
| Hauptprogramm (mit Hardware) | `cargo run --bin colour_track` an Webcam + optional Arduino | Live-Bild, Konturen grün, Overlay, ESC; Arduino empfängt `p…\n`/`t…\n` (mit `screen`/`minicom` auf `/dev/ttyACM0` prüfen) |

### 7.4 Cross-Compile (Zielplattform lt. rust/README.md: Raspberry Pi)

```bash
rustup target add aarch64-unknown-linux-gnu        # Pi 64-bit
cargo build --release --target aarch64-unknown-linux-gnu
# 32-bit: armv7-unknown-linux-gnueabihf
```


---

## Schritt 8: Kompakte Prozedur-Checkliste (zum Abarbeiten)

1. **Voraussetzungen**: `rust/scripts/install_deps.sh` ausführen (OpenCV, clang, pkg-config, udev), Rust via rustup, `cd rust/`.
2. **Modul `trig` portieren/prüfen**: `trig.py` → `src/trig.rs`; alle 9 Funktionen gegen Referenzwerte testen (`cargo test trig`).
3. **Modul `common` portieren/prüfen**: genutzte Helfer aus `common.py` → `src/common.rs` (`Bunch`, `StatValue`, `Timer`, `clock`, `draw_str`, `getsize`, `find_largest_contour`, `HsvRange`, `Preferences`, `ArduinoConfig`); ungenutzte Helfer (`Sketcher`, `RectSelector`, `mosaic` …) nur bei Bedarf.
4. **Anwendung `colour_track`**: `colourTrack.py` → `src/bin/colour_track.rs`; Hauptschleife gemäß Doku 9.1; Preferences als Struct; Arduino-Protokoll `p<pan>\n`/`t<tilt>\n` auf `/dev/ttyACM0` @ 115200.
5. **Anwendung `calibrate_range`**: `calibrateRange.py` → `src/bin/calibrate_range.rs`; 6 HSV-Trackbars, Live-Maske, ESC-Exit, finale HSV-Werte ausgeben.
6. **Konstanten zentral halten**: alle Werte aus Doku Abschnitt 11 als `pub const` in `trig.rs`/`common.rs` (FOV, Offsets, Servo-Grenzen, HSV-Default, Serial-Port).
7. **Lücken schließen (optional)**: VideoWriter (`prefWriteVideo`) in `colour_track.rs` um `videoio::VideoWriter` + `chrono`-Dateinamen ergänzen; `.gitignore` um `/rust/target/` ergänzen.
8. **Verifizieren**: `cargo test` (alle Unit-Tests + empfohlene Äquivalenztests gegen Python-Referenzwerte), `cargo clippy`, `cargo fmt --check`, `cargo build --release`.
9. **Ausführen**: `cargo run --bin calibrate_range` (HSV kalibrieren), dann `cargo run --bin colour_track` (Tracking; ESC beendet). Für Raspberry Pi: Cross-Compile mit `aarch64-unknown-linux-gnu` (Abschnitt 7.4).

---

*Ende der Prozedur. Erstellt aus `FunktionalDokument.md` (Abschnitte 2–12), den Python-Quellen (`trig.py`, `common.py`, `colourTrack.py`, `calibrateRange.py`) und dem bestehenden Rust-Stand unter `rust/` (Commit 5361e6f).*
