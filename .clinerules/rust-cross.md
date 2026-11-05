---
paths:
  - "Cross.toml"
  - ".cargo/**"
  - "scripts/build*"
---
# Rust Cross-Compilation Guide
- Verwende Cross.toml für die Konfiguration der Toolchain.
- Stelle sicher, dass die Zielplattformen (Raspberry Pi 5 und NVIDIA Jetson) korrekt konfiguriert sind.
- Verwende die `armv7-linux-gnueabihf`-Target für beide Plattformen.
- Teste die Binärdateien auf den Zielplattformen vor der Deployment.
- Verwende die `--target`-Flag in den Build-Commands, um die Zielplattform zu spezifizieren.
- Builds für ARM64 IMMER über `cross` (Docker-Images mit fertiger Toolchain), nie manuelles Linker-Setup.
- Ziele: aarch64-unknown-linux-gnu (RPi 5 & Jetson, beide ARM64 → ein Target!).
- Befehle: `cross build --release --target aarch64-unknown-linux-gnu`, `cross test --target ...`.
- Docker Desktop/Engine muss laufen, sonst cross-Fehler.
- Output liegt in target/aarch64-unknown-linux-gnu/release/.
- Binary nach dem Build mit `aarch64-linux-gnu-strip` (im cross-Image) strippen.