//! Calibrate Range - HSV color calibration tool
//! 
//! This tool provides an interactive GUI for calibrating HSV color ranges
//! for object detection. It displays the live camera feed with trackbars
//! to adjust the Hue, Saturation, and Value ranges in real-time.
//! 
//! # Usage
//! 
//! ```bash
//! cargo run --bin calibrate_range
//! ```
//! 
//! # Controls
//! 
//! - H- / H+: Adjust Hue range (0-255)
//! - S- / S+: Adjust Saturation range (0-255)
//! - V- / V+: Adjust Value range (0-255)
//! - ESC: Exit the program

use anyhow::{Context, Result};
use opencv::{
    core::{Mat, Point, Scalar, Size, Vector},
    imgproc::{self, COLOR_BGR2HSV, RETR_LIST, CHAIN_APPROX_SIMPLE},
    highgui,
    videoio,
    prelude::*,
};
use std::sync::{Arc, Mutex};

use paintball_sentry_turret::common::{draw_str, find_largest_contour};

/// HSV range values that can be modified via trackbars
#[derive(Debug, Clone)]
struct HsvValues {
    h_min: i32,
    h_max: i32,
    s_min: i32,
    s_max: i32,
    v_min: i32,
    v_max: i32,
}

impl Default for HsvValues {
    fn default() -> Self {
        Self {
            h_min: 0,
            h_max: 255,
            s_min: 0,
            s_max: 255,
            v_min: 0,
            v_max: 255,
        }
    }
}

// Global storage for HSV values (needed for trackbar callbacks)
static HSV_VALUES: std::sync::OnceLock<Arc<Mutex<HsvValues>>> = std::sync::OnceLock::new();

fn get_hsv_values() -> Arc<Mutex<HsvValues>> {
    HSV_VALUES
        .get_or_init(|| Arc::new(Mutex::new(HsvValues::default())))
        .clone()
}

fn main() -> Result<()> {
    println!("Paintball Sentry Turret - HSV Calibration Tool");
    println!("===============================================");
    println!("Adjust trackbars to set HSV color range");
    println!("Press ESC to exit");
    
    // Initialize video capture
    let mut cap = videoio::VideoCapture::new(1, videoio::CAP_ANY)
        .context("Failed to open video capture device 1")?;
    
    if !cap.is_opened()? {
        cap = videoio::VideoCapture::new(0, videoio::CAP_ANY)
            .context("Failed to open video capture device 0")?;
    }
    
    if !cap.is_opened()? {
        anyhow::bail!("Could not open any video capture device");
    }
    
    println!("Video capture initialized");
    
    // Create window
    highgui::named_window("frame", highgui::WINDOW_NORMAL)?;
    
    // Initialize HSV values
    let hsv_values = get_hsv_values();
    
    // Create trackbars for HSV range adjustment
    // Note: OpenCV Rust trackbar callbacks are limited, so we'll poll values in the loop
    highgui::create_trackbar("H-", "frame", Some(&mut 0), 255, None)?;
    highgui::create_trackbar("H+", "frame", Some(&mut 255), 255, None)?;
    highgui::create_trackbar("S-", "frame", Some(&mut 0), 255, None)?;
    highgui::create_trackbar("S+", "frame", Some(&mut 255), 255, None)?;
    highgui::create_trackbar("V-", "frame", Some(&mut 0), 255, None)?;
    highgui::create_trackbar("V+", "frame", Some(&mut 255), 255, None)?;
    
    let mut max_area: f64 = 0.0;
    
    // Main loop
    loop {
        // Capture frame
        let mut frame = Mat::default();
        cap.read(&mut frame)?;
        
        if frame.empty() {
            continue;
        }
        
        // Apply blur (3x3 kernel)
        let mut blurred = Mat::default();
        imgproc::blur(&frame, &mut blurred, Size::new(3, 3), Point::new(-1, -1))?;
        
        // Convert to HSV
        let mut hsv = Mat::default();
        imgproc::cvt_color(&blurred, &mut hsv, COLOR_BGR2HSV, 0)?;
        
        // Get trackbar positions
        let h_min = highgui::get_trackbar_pos("H-", "frame")?.unwrap_or(0);
        let h_max = highgui::get_trackbar_pos("H+", "frame")?.unwrap_or(255);
        let s_min = highgui::get_trackbar_pos("S-", "frame")?.unwrap_or(0);
        let s_max = highgui::get_trackbar_pos("S+", "frame")?.unwrap_or(255);
        let v_min = highgui::get_trackbar_pos("V-", "frame")?.unwrap_or(0);
        let v_max = highgui::get_trackbar_pos("V+", "frame")?.unwrap_or(255);
        
        // Create HSV range mask
        let lower = Scalar::new(h_min as f64, s_min as f64, v_min as f64, 0.0);
        let upper = Scalar::new(h_max as f64, s_max as f64, v_max as f64, 0.0);
        
        let mut mask = Mat::default();
        opencv::core::in_range(&hsv, &lower, &upper, &mut mask)?;
        
        // Find contours
        let mut contours: Vector<Vector<Point>> = Vector::default();
        imgproc::find_contours(
            &mask,
            &mut contours,
            RETR_LIST,
            CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;
        
        // Find largest contour
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
        }
        
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
        
        // Draw info overlay
        let area_str = format!("area: {:.1} px", max_area);
        draw_str(&mut frame, 20, 50, &area_str)?;
        
        // Draw current HSV range info
        let hsv_info = format!(
            "HSV: H[{}-{}] S[{}-{}] V[{}-{}]",
            h_min, h_max, s_min, s_max, v_min, v_max
        );
        draw_str(&mut frame, 20, 80, &hsv_info)?;
        
        // Show frame
        highgui::imshow("frame", &frame)?;
        
        // Check for ESC key
        let key = highgui::wait_key(33)?;
        if key == 27 {
            break;
        }
    }
    
    // Print final HSV values for reference
    let h_min = highgui::get_trackbar_pos("H-", "frame")?.unwrap_or(0);
    let h_max = highgui::get_trackbar_pos("H+", "frame")?.unwrap_or(255);
    let s_min = highgui::get_trackbar_pos("S-", "frame")?.unwrap_or(0);
    let s_max = highgui::get_trackbar_pos("S+", "frame")?.unwrap_or(255);
    let v_min = highgui::get_trackbar_pos("V-", "frame")?.unwrap_or(0);
    let v_max = highgui::get_trackbar_pos("V+", "frame")?.unwrap_or(255);
    
    println!("\nFinal HSV Range Values:");
    println!("======================");
    println!("Hue:        {} - {}", h_min, h_max);
    println!("Saturation: {} - {}", s_min, s_max);
    println!("Value:      {} - {}", v_min, v_max);
    println!("\nUse these values in colour_track.rs HsvRange configuration");
    
    // Cleanup
    highgui::destroy_all_windows()?;
    cap.release()?;
    
    Ok(())
}