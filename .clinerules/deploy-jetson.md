---
paths:
  - "deploy/**"
  - "scripts/deploy*"
  - "scripts/build-rpi*"
---
# Deployment Guide for NVIDIA Jetson
- Verwende den `deploy/jetson.service`-Datei für die systemd-Unit-Datei.
- Stelle sicher, dass die Binärdatei auf dem Jetson-Gerät kopiert wird.
- Starte den Dienst mit `sudo systemctl start paintball-sentry-turret`.
- Verwende `scp` für die Dateikopierung.
- Verwende `ssh` für die Fernsteuerung des Jetson-Geräts.
# RPi/Jetson-Deployment
- Deployment per scp + systemd (Units in deploy/ als Vorlage).
- Reihenfolge: build → strip → scp zum Ziel → `sudo systemctl restart <unit>`.
- Pfade am Ziel: /opt/kihr5/, Service-User ohne Root-Rechte.
- Logs am Ziel: `journalctl -u <unit> -f` – für Debug-Aufgaben an Cline weitergeben.
- Jetson: falls GPU/CUDA genutzt wird, Hinweis: Rust-Binding (z. B. cudarc) nur wenn explizit verlangt – Build-Komplexität hoch.