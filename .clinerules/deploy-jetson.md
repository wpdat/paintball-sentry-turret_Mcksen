# Deployment Guide for NVIDIA Jetson
- Verwende den `deploy/jetson.service`-Datei für die systemd-Unit-Datei.
- Stelle sicher, dass die Binärdatei auf dem Jetson-Gerät kopiert wird.
- Starte den Dienst mit `sudo systemctl start paintball-sentry-turret`.
- Verwende `scp` für die Dateikopierung.
- Verwende `ssh` für die Fernsteuerung des Jetson-Geräts.