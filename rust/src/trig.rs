//! Trigonometric and coordinate transformation module
//! 
//! This module contains functions that find the target in space.
//! Assumes: 640x480 resolution
//! Units: radians, meters
//! 
//! # Coordinate System
//! 
//! - Camera center: (320, 240) => (0, 0) radians
//! - Top-left: (0, 0) => (-24°, 18°)
//! - Bottom-right: (640, 480) => (24°, -18°)
//! 
//! # Field of View
//! 
//! - FOV: 60° with 4:3 aspect ratio
//! - X-FOV: 48°
//! - Y-FOV: 36°

use std::f64::consts::PI;

/// Camera field of view constants
pub const CAMERA_FOV_X: f64 = 48.0;  // degrees
pub const CAMERA_FOV_Y: f64 = 36.0;  // degrees
pub const CAMERA_RESOLUTION_X: f64 = 640.0;
pub const CAMERA_RESOLUTION_Y: f64 = 480.0;

/// Estimated default range (meters)
pub const DEFAULT_RANGE_Z: f64 = 5.0;

/// Known object height for height-based calculations (meters)
pub const OBJECT_HEIGHT: f64 = 0.54;

/// Camera to gun offset constants
pub const GUN_OFFSET_X: f64 = -0.11;  // meters
pub const GUN_OFFSET_Y: f64 = -0.26;  // meters
pub const GUN_OFFSET_Z: f64 = 0.15;   // meters

/// Servo signal constants
pub const SERVO_DEGREE_FACTOR: f64 = 0.325;
pub const PAN_CENTER: i32 = 538;
pub const TILT_CENTER: i32 = 488;
pub const PAN_MIN: i32 = 251;
pub const PAN_MAX: i32 = 805;
pub const TILT_MIN: i32 = 211;
pub const TILT_MAX: i32 = 765;

/// Converts degrees to radians
#[inline]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// Converts radians to degrees
#[inline]
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

/// Transforms points on screen to angles in camera
/// 
/// # Arguments
/// 
/// * `x` - X pixel coordinate (0-640)
/// * `y` - Y pixel coordinate (0-480)
/// 
/// # Returns
/// 
/// Tuple of (theta, phi) angles in radians
/// 
/// # Examples
/// 
/// ```
/// use paintball_sentry_turret::trig::px_to_angle;
/// 
/// // Center of screen => (0, 0)
/// let (theta, phi) = px_to_angle(320.0, 240.0);
/// assert!((theta.abs() < 0.01) && (phi.abs() < 0.01));
/// ```
pub fn px_to_angle(x: f64, y: f64) -> (f64, f64) {
    // (320,240) => (0,0); (0,0) => (-24,18); (640,480) => (24,-18)
    // The FOV is 60deg with an aspect ratio of 4:3
    // x has 48deg, y has 36deg
    // Assume: Distance is linear to angle (might be off on the sides)
    let theta = deg_to_rad(0.075 * x - 24.0);
    let phi = deg_to_rad(-0.075 * y + 18.0);
    (theta, phi)
}

/// Finds the cartesian location from 2 angles and estimating a range
/// 
/// # Arguments
/// 
/// * `theta` - Horizontal angle in radians
/// * `phi` - Vertical angle in radians
/// 
/// # Returns
/// 
/// Tuple of (x, y, z) coordinates in meters
pub fn cam_polar_cart_z(theta: f64, phi: f64) -> (f64, f64, f64) {
    let z = DEFAULT_RANGE_Z;
    let x = z * theta.tan();
    let y = z * phi.tan();
    (x, y, z)
}

/// Finds the cartesian location from 2 angles and knowing the height of the object
/// 
/// # Arguments
/// 
/// * `theta` - Horizontal angle in radians
/// * `phi` - Vertical angle in radians
/// * `h` - Height of object in pixels
/// 
/// # Returns
/// 
/// Tuple of (x, y, z) coordinates in meters
pub fn cam_polar_cart_h(theta: f64, phi: f64, h: f64) -> (f64, f64, f64) {
    let z = OBJECT_HEIGHT / deg_to_rad(0.075 * h).tan();
    let x = z * theta.tan();
    let y = z * phi.tan();
    (x, y, z)
}

/// Shifts cartesian coordinates from camera to gun reference frame
/// 
/// # Arguments
/// 
/// * `x` - X coordinate in camera frame
/// * `y` - Y coordinate in camera frame
/// * `z` - Z coordinate in camera frame
/// 
/// # Returns
/// 
/// Tuple of (x, y, z) in gun reference frame
pub fn shift_cart(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // a, b, c is gun to cam, on relative gun cart
    (x + GUN_OFFSET_X, y + GUN_OFFSET_Y, z + GUN_OFFSET_Z)
}

/// Takes a point in space and aims the barrel towards that point
/// 
/// Assumes that the center of rotation of the gun is at the origin.
/// 
/// # Arguments
/// 
/// * `x` - X coordinate in gun frame
/// * `y` - Y coordinate in gun frame
/// * `z` - Z coordinate in gun frame
/// 
/// # Returns
/// 
/// Tuple of (theta, phi) in radians
/// - theta: controls the pan (horizontal)
/// - phi: controls the tilt (vertical)
pub fn gun_cart_to_polar(x: f64, y: f64, z: f64) -> (f64, f64) {
    let theta = x.atan2(z);
    let phi = y.atan2(z);
    (theta, phi)
}

/// Converts the polar angles to actual degrees the servo will use
/// 
/// # Servo Calibration
/// 
/// - Degree = Position Raw Data × 0.325
/// - (21, 1002) max but probably never should reach that
/// - 512 - Center of rotation
/// 
/// # Arguments
/// 
/// * `theta` - Pan angle in radians
/// * `phi` - Tilt angle in radians
/// 
/// # Returns
/// 
/// Tuple of (pan, tilt) servo signal values
pub fn signal_from_polar(theta: f64, phi: f64) -> (i32, i32) {
    let a = -rad_to_deg(theta) / SERVO_DEGREE_FACTOR + PAN_CENTER as f64;
    let b = -rad_to_deg(phi) / SERVO_DEGREE_FACTOR + TILT_CENTER as f64;

    // Safety bounds check
    if a < PAN_MIN as f64 || a > PAN_MAX as f64 {
        eprintln!(
            "!!!!!!!!!!! a = {} went beyond 90Deg !!!!!!!!!!!!!",
            a
        );
        return (PAN_CENTER, TILT_CENTER);
    }
    if b < TILT_MIN as f64 || b > TILT_MAX as f64 {
        eprintln!(
            "!!!!!!!!!!! b = {} went beyond 90Deg !!!!!!!!!!!!!",
            b
        );
        return (PAN_CENTER, TILT_CENTER);
    }

    (a.round() as i32, b.round() as i32)
}

/// 3D velocity vector
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

/// 3D position
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

/// Finds the velocity of the marker
/// 
/// # Arguments
/// 
/// * `gun_cart` - Current position in gun frame
/// * `last_gun_cart` - Previous position in gun frame
/// * `dt` - Time delta in seconds
/// 
/// # Returns
/// 
/// Velocity vector
pub fn compute_velocity(gun_cart: Position, last_gun_cart: Position, dt: f64) -> Velocity {
    if dt <= 0.0 {
        return Velocity::default();
    }
    
    Velocity {
        vx: ((gun_cart.x - last_gun_cart.x) / dt * 1000.0).round() / 1000.0,
        vy: ((gun_cart.y - last_gun_cart.y) / dt * 1000.0).round() / 1000.0,
        vz: ((gun_cart.z - last_gun_cart.z) / dt * 1000.0).round() / 1000.0,
    }
}

/// Estimates the position of the marker with velocity
/// 
/// Uses last clock time to estimate next position.
/// 
/// # Arguments
/// 
/// * `gun_cart` - Current position
/// * `v` - Velocity vector
/// * `dt` - Time delta for prediction
/// 
/// # Returns
/// 
/// Estimated next position
pub fn est_next_pos(gun_cart: Position, v: Velocity, dt: f64) -> Position {
    let prediction_dt = 0.1; // Fixed prediction time
    Position {
        x: gun_cart.x + v.vx * prediction_dt,
        y: gun_cart.y + v.vy * prediction_dt,
        z: gun_cart.z + v.vz * prediction_dt,
    }
}

/// Main transformation function: takes the position from the camera and returns pan and tilt for the turret
/// 
/// This is the primary function that converts camera pixel coordinates to servo signals.
/// 
/// # Arguments
/// 
/// * `cx` - Current X pixel coordinate
/// * `cy` - Current Y pixel coordinate
/// * `last_cx` - Optional previous X pixel coordinate (for prediction)
/// * `last_cy` - Optional previous Y pixel coordinate (for prediction)
/// * `dt` - Optional time delta (for prediction)
/// 
/// # Returns
/// 
/// Tuple of (pan, tilt) servo signal values
pub fn cam_to_gun(
    cx: f64,
    cy: f64,
    last_cx: Option<f64>,
    last_cy: Option<f64>,
    dt: Option<f64>,
) -> (i32, i32) {
    // Step 1: Convert pixel coordinates to camera angles
    let cam_angles = px_to_angle(cx, cy);

    // Step 2: Convert angles to 3D cartesian coordinates
    let cam_cart = cam_polar_cart_z(cam_angles.0, cam_angles.1);

    // Step 3: Shift from camera to gun reference frame
    let gun_cart = shift_cart(cam_cart.0, cam_cart.1, cam_cart.2);

    // Step 4: Optional prediction with velocity
    let gun_polar = if let (Some(last_x), Some(last_y), Some(delta_t)) = (last_cx, last_cy, dt) {
        let last_cam_angles = px_to_angle(last_x, last_y);
        let last_cam_cart = cam_polar_cart_z(last_cam_angles.0, last_cam_angles.1);
        let last_gun_cart = shift_cart(last_cam_cart.0, last_cam_cart.1, last_cam_cart.2);

        let gun_cart_pos = Position::new(gun_cart.0, gun_cart.1, gun_cart.2);
        let last_gun_cart_pos = Position::new(last_gun_cart.0, last_gun_cart.1, last_gun_cart.2);

        let v = compute_velocity(gun_cart_pos, last_gun_cart_pos, delta_t);
        let pos = est_next_pos(gun_cart_pos, v, delta_t);
        gun_cart_to_polar(pos.x, pos.y, pos.z)
    } else {
        gun_cart_to_polar(gun_cart.0, gun_cart.1, gun_cart.2)
    };

    // Step 5: Convert to servo signals
    signal_from_polar(gun_polar.0, gun_polar.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_to_angle_center() {
        let (theta, phi) = px_to_angle(320.0, 240.0);
        assert!((theta.abs() < 0.01), "theta should be ~0 at center");
        assert!((phi.abs() < 0.01), "phi should be ~0 at center");
    }

    #[test]
    fn test_px_to_angle_corners() {
        // Top-left corner
        let (theta, phi) = px_to_angle(0.0, 0.0);
        assert!((theta - deg_to_rad(-24.0)).abs() < 0.01);
        assert!((phi - deg_to_rad(18.0)).abs() < 0.01);

        // Bottom-right corner
        let (theta, phi) = px_to_angle(640.0, 480.0);
        assert!((theta - deg_to_rad(24.0)).abs() < 0.01);
        assert!((phi - deg_to_rad(-18.0)).abs() < 0.01);
    }

    #[test]
    fn test_cam_polar_cart_z() {
        let (x, y, z) = cam_polar_cart_z(0.0, 0.0);
        assert!((x.abs() < 0.01), "x should be ~0 at center");
        assert!((y.abs() < 0.01), "y should be ~0 at center");
        assert!((z - DEFAULT_RANGE_Z).abs() < 0.01);
    }

    #[test]
    fn test_shift_cart() {
        let (x, y, z) = shift_cart(0.0, 0.0, 0.0);
        assert!((x - GUN_OFFSET_X).abs() < 0.01);
        assert!((y - GUN_OFFSET_Y).abs() < 0.01);
        assert!((z - GUN_OFFSET_Z).abs() < 0.01);
    }

    #[test]
    fn test_gun_cart_to_polar() {
        let (theta, phi) = gun_cart_to_polar(0.0, 0.0, 5.0);
        assert!((theta.abs() < 0.01), "theta should be ~0 when x=0");
        assert!((phi.abs() < 0.01), "phi should be ~0 when y=0");
    }

    #[test]
    fn test_signal_from_polar_center() {
        let (pan, tilt) = signal_from_polar(0.0, 0.0);
        assert_eq!(pan, PAN_CENTER);
        assert_eq!(tilt, TILT_CENTER);
    }

    #[test]
    fn test_signal_from_polar_bounds() {
        // Test that out-of-bounds values return center
        let (pan, tilt) = signal_from_polar(10.0, 10.0);
        assert_eq!(pan, PAN_CENTER);
        assert_eq!(tilt, TILT_CENTER);
    }

    #[test]
    fn test_cam_to_gun_center() {
        let (pan, tilt) = cam_to_gun(320.0, 240.0, None, None, None);
        // At center, should be close to center values
        assert!((pan - PAN_CENTER).abs() < 50);
        assert!((tilt - TILT_CENTER).abs() < 50);
    }

    #[test]
    fn test_compute_velocity() {
        let pos1 = Position::new(0.0, 0.0, 5.0);
        let pos2 = Position::new(1.0, 0.0, 5.0);
        let v = compute_velocity(pos2, pos1, 1.0);
        assert!((v.vx - 1.0).abs() < 0.01);
        assert!((v.vy.abs() < 0.01));
        assert!((v.vz.abs() < 0.01));
    }

    #[test]
    fn test_est_next_pos() {
        let pos = Position::new(0.0, 0.0, 5.0);
        let v = Velocity {
            vx: 1.0,
            vy: 0.5,
            vz: 0.0,
        };
        let next = est_next_pos(pos, v, 0.1);
        assert!((next.x - 0.1).abs() < 0.01);
        assert!((next.y - 0.05).abs() < 0.01);
        assert!((next.z - 5.0).abs() < 0.01);
    }
}