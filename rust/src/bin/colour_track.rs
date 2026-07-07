//! Colour Track - Main color tracking program
//! 
//! This is the main program for the paintball sentry turret.
//! It captures video from a camera, detects colored objects using HSV color space,
//! calculates the target position, and optionally sends commands to an Arduino
//! for servo control.
//! 
//! # Usage
//! 
//! ```bash
//! cargo run --bin colour_track
//! ```
//! 
//! # Controls
//! 
//! - ESC: Exit the program

use anyhow::{Context, Result};
use opencv::{
    core::{Mat, Point, Scalar, Size, Vector, CV_8UC3},
    imgproc::{self, COLOR_BGR2HSV, RETR_LIST, CHAIN_APPROX_SIMPLE},
    highgui,
    videoio,
    prelude::*,
};
use std::io::Write;

use paintball_sentry_turret::{
    common::{
        clock, draw_str, find_largest_contour, ArduinoConfig, HsvRange, Preferences,
    },
    trig::cam_to_gun,
};

/// Serial port handler for Arduino communication
struct ArduinoSerial {
    port: Option<Box<dyn serialport::SerialPort>>,
}

impl ArduinoSerial {
    fn new(config: &ArduinoConfig) -> Self {
        let port = serialport::new(&config.port, config.baud_rate)
            .timeout(std::time::Duration::from_millis(100))
            .open()
            .ok();
        
        if port.is_some() {
            println!("Arduino connected on {}", config.port);
        } else {
            eprintln!("Warning: Could not open serial port {}", config.port);
        }
        
        Self { port }
    }

    fn send_pan(&mut self, value: i32) {
        if let Some(ref mut port) = self.port {
            let cmd = format!("p{}\n", value);
            let _ = port.write_all(cmd.as_bytes());
        }
    }

    fn send_tilt(&mut self, value: i32) {
        if let Some(ref mut port) = self.port {
            let cmd = format!("t{}\n", value);
            let _ = port.write_all(cmd.as_bytes());
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("Paintball Sentry Turret - Colour Track");
    println!("=======================================");
    println!("Press ESC to exit");
    
    // Configuration
    let prefs = Preferences::new();
    let arduino_config = ArduinoConfig::new();
    let hsv_range = HsvRange::default_orange();
    
    // Initialize video capture (device 1 as in Python version)
    let mut cap = videoio::VideoCapture::new(1, videoio::CAP_ANY)
        .context("Failed to open video capture device 1")?;
    
    if !cap.is_opened()? {
        // Try device 0 as fallback
        cap = videoio::VideoCapture::new(0, videoio::CAP_ANY)
            .context("Failed to open video capture device 0")?;
    }
    
    if !cap.is_opened()? {
        anyhow::bail!("Could not open any video capture device");
    }
    
    println!("Video capture initialized");
    
    // Initialize Arduino serial connection if enabled
    let mut arduino: Option<ArduinoSerial> = if prefs.arduino_enabled {
        Some(ArduinoSerial::new(&arduino_config))
    } else {
        None
    };
    
    // State variables for target prediction
    let mut last_cx: Option<f64> = None;
    let mut last_cy: Option<f64> = None;
    let mut last_time: Option<f64> = None;
    let mut current_pan: i32 = 0;
    let mut current_tilt: i32 = 0;
    let mut last_dt: f64 = 0.0;
    let mut max_area: f64 = 0.0;
    
    // Main loop
    loop {
        // Capture frame
        let mut frame = Mat::default();
        cap.read(&mut frame)?;
        
        if frame.empty() {
            continue;
        }
        
        // Apply Gaussian blur (3x3 kernel as in Python version)
        let mut blurred = Mat::default();
        imgproc::blur(&frame, &mut blurred, Size::new(3, 3), Point::new(-1, -1))?;
        
        // Convert to HSV color space
        let mut hsv = Mat::default();
        imgproc::cvt_color(&blurred, &mut hsv, COLOR_BGR2HSV, 0)?;
        
        // Create mask for target color
        let mut mask = Mat::default();
        opencv::core::in_range(&hsv, &hsv_range.lower_scalar(), &hsv_range.upper_scalar(), &mut mask)?;
        
        // Find contours
        let mut contours: Vector<Vector<Point>> = Vector::default();
        let mut hierarchy = Mat::default();
        imgproc::find_contours(
            &mask,
            &mut contours,
            RETR_LIST,
            CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;
        
        // Find largest contour and its centroid
        if let Some(contour_result) = find_largest_contour(&contours) {
            let cx = contour_result.center_x;
            let cy = contour_result.center_y;
            max_area = contour_result.area;
            
            // Draw center point
            imgproc::circle(
                &mut frame,
                Point::new(cx, cy),
                5,
                Scalar::new(255.0, 255.0, 255.0, 0.0),
                -1,
                imgproc::LINE_8,
                0,
            )?;
            
            // Calculate pan/tilt using trig module
            let current_time = clock();
            let (pan, tilt) = if prefs.target_lead_enabled && last_time.is_some() {
                let dt = current_time - last_time.unwrap();
                last_dt = dt;
                cam_to_gun(
                    cx as f64,
                    cy as f64,
                    last_cx,
                    last_cy,
                    Some(dt),
                )
            } else {
                cam_to_gun(cx as f64, cy as f64, None, None, None)
            };
            
            // Update state for next iteration
            last_time = Some(current_time);
            last_cx = Some(cx as f64);
            last_cy = Some(cy as f64);
            current_pan = pan;
            current_tilt = tilt;
            
            // Send commands to Arduino if enabled
            if let Some(ref mut arduino_serial) = arduino {
                arduino_serial.send_pan(pan);
                arduino_serial.send_tilt(tilt);
            }
        }
        
        // Display if enabled
        if prefs.screen_enabled {
            // Draw contours outline
            for cnt in contours.iter() {
                imgproc::draw_contours(
                    &mut frame,
                    &Vector::from_slice(&[cnt.clone()]),
                    0,
                    Scalar::new(0.0, 255.0, 0.0, 0.0),
                    3,
                    imgproc::LINE_8,
                    &Mat::default(),
                    i32::MAX,
                    Point::new(0, 0),
                )?;
            }
            
            // Draw information overlay
            if last_dt > 0.0 {
                let time_str = format!("time: {:.1} ms", last_dt * 1000.0);
                draw_str(&mut frame, 20, 20, &time_str)?;
            }
            
            let area_str = format!("area: {:.1} px", max_area);
            draw_str(&mut frame, 20, 50, &area_str)?;
            
            if current_pan > 0 || current_tilt > 0 {
                let pan_str = format!("pan: {}", current_pan);
                let tilt_str = format!("tilt: {}", current_tilt);
                draw_str(&mut frame, 20, 80, &pan_str)?;
                draw_str(&mut frame, 20, 110, &tilt_str)?;
            }
            
            // Show frame
            highgui::imshow("frame", &frame)?;
        }
        
        // Check for ESC key (33ms delay for ~30fps)
        let key = highgui::wait_key(33)?;
        if key == 27 {
            // ESC pressed
            break;
        }
    }
    
    // Cleanup
    println!("Shutting down...");
    highgui::destroy_all_windows()?;
    cap.release()?;
    
    println!("Goodbye!");
    Ok(())
}