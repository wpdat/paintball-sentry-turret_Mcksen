# Build script for Raspberry Pi
# Build the project
./cargo build --release
# Copy the binary to the target directory
cp target/release/paintball-sentry-turret /home/pi/paintball-sentry-turret/
# Start the service
sudo systemctl start paintball-sentry-turret