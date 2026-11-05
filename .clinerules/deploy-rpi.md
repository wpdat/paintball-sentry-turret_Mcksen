---
paths:
  - "Cross.toml"
  - ".cargo/**"
  - "scripts/build*"
---
# Deployment Guide for Raspberry Pi
- Verwende den `deploy/rpi.service`-Datei für die systemd-Unit-Datei.
- Stelle sicher, dass die Binärdatei auf dem Raspberry Pi kopiert wird.
- Starte den Dienst mit `sudo systemctl start paintball-sentry-turret`.
- Verwende `scp` für die Dateikopierung.
- Verwende `ssh` für die Fernsteuerung des Raspberry Pi.
# Cross-Kompilierung
- Builds für ARM64 IMMER über `cross` (Docker-Images mit fertiger Toolchain), nie manuelles Linker-Setup.
- Ziele: aarch64-unknown-linux-gnu (RPi 5 & Jetson, beide ARM64 → ein Target!).
- Befehle: `cross build --release --target aarch64-unknown-linux-gnu`, `cross test --target ...`.
- Docker Desktop/Engine muss laufen, sonst cross-Fehler.
- Output liegt in target/aarch64-unknown-linux-gnu/release/.
- Binary nach dem Build mit `aarch64-linux-gnu-strip` (im cross-Image) strippen.