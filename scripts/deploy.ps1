# Deploy script
# Copy the binary to the target directory
scp target/release/paintball-sentry-turret pi@raspberrypi:/home/pi/paintball-sentry-turret/
scp target/release/paintball-sentry-turret jetson@jetson:/home/jetson/paintball-sentry-turret/
# Start the service on both devices
ssh pi@raspberrypi 'sudo systemctl start paintball-sentry-turret'
ssh jetson@jetson 'sudo systemctl start paintball-sentry-turret'