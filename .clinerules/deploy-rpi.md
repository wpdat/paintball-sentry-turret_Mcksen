# Deployment Guide for Raspberry Pi
- Verwende den `deploy/rpi.service`-Datei für die systemd-Unit-Datei.
- Stelle sicher, dass die Binärdatei auf dem Raspberry Pi kopiert wird.
- Starte den Dienst mit `sudo systemctl start paintball-sentry-turret`.
- Verwende `scp` für die Dateikopierung.
- Verwende `ssh` für die Fernsteuerung des Raspberry Pi.