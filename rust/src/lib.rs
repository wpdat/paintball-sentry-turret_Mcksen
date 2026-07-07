//! Paintball Sentry Turret - Rust Implementation
//! 
//! This is a Rust port of the Python-based paintball sentry turret system
//! designed for Raspberry Pi platform.
//! 
//! # Modules
//! 
//! - `trig`: Coordinate transformations (camera to gun)
//! - `common`: Helper functions and OpenCV utilities
//! 
//! # Binaries
//! 
//! - `colour_track`: Main color tracking program
//! - `calibrate_range`: HSV calibration tool

pub mod trig;
pub mod common;