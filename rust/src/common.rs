//! Common utilities and helper functions
//! 
//! This module contains common routines used by other modules,
//! including OpenCV utilities, drawing functions, and helper classes.

use opencv::{
    core::{Mat, Point, Scalar, Size, Vector},
    imgproc::{self, FONT_HERSHEY_PLAIN, LINE_8},
    highgui,
    prelude::*,
    Result as OpenCvResult,
};
use std::time::Instant;

/// Image file extensions supported
pub const IMAGE_EXTENSIONS: &[&str] = &[
    ".bmp", ".jpg", ".jpeg", ".png", ".tif", ".tiff", ".pbm", ".pgm", ".ppm",
];

/// Simple data container (similar to Python's Bunch class)
#[derive(Debug, Clone)]
pub struct Bunch {
    pub data: std::collections::HashMap<String, String>,
}

impl Bunch {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

impl Default for Bunch {
    fn default() -> Self {
        Self::new()
    }
}

/// Splits a filename into path, name, and extension
pub fn splitfn(fn_path: &str) -> (&str, &str, &str) {
    let path_end = fn_path.rfind('/').unwrap_or(0);
    let path = &fn_path[..path_end];
    let filename = &fn_path[path_end..];
    
    let ext_start = filename.rfind('.').unwrap_or(filename.len());
    let name = &filename[..ext_start];
    let ext = &filename[ext_start..];
    
    (path, name, ext)
}

/// Returns the current time in seconds (similar to cv2.getTickCount / cv2.getTickFrequency)
pub fn clock() -> f64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_secs_f64()
}

/// Timer context for measuring execution time
pub struct Timer {
    msg: String,
    start: f64,
}

impl Timer {
    pub fn new(msg: &str) -> Self {
        print!("{} ...", msg);
        Self {
            msg: msg.to_string(),
            start: clock(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        (clock() - self.start) * 1000.0
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        println!("{:.2} ms", self.elapsed());
    }
}

/// Statistical value with smoothing
#[derive(Debug, Clone)]
pub struct StatValue {
    pub value: Option<f64>,
    pub smooth_coef: f64,
}

impl StatValue {
    pub fn new(smooth_coef: f64) -> Self {
        Self {
            value: None,
            smooth_coef,
        }
    }

    pub fn update(&mut self, v: f64) {
        match self.value {
            None => self.value = Some(v),
            Some(current) => {
                let c = self.smooth_coef;
                self.value = Some(c * current + (1.0 - c) * v);
            }
        }
    }
}

impl Default for StatValue {
    fn default() -> Self {
        Self::new(0.5)
    }
}

/// Draws text on an image with shadow effect
/// 
/// # Arguments
/// 
/// * `dst` - Destination image
/// * `x` - X coordinate
/// * `y` - Y coordinate  
/// * `s` - Text string to draw
pub fn draw_str(dst: &mut Mat, x: i32, y: i32, s: &str) -> OpenCvResult<()> {
    let font_scale = 1.0;
    let thickness = 1;
    let line_type = LINE_8;
    
    // Draw shadow (black)
    imgproc::put_text(
        dst,
        s,
        Point::new(x + 1, y + 1),
        FONT_HERSHEY_PLAIN,
        font_scale,
        Scalar::new(0.0, 0.0, 0.0, 0.0),
        thickness,
        line_type,
        false,
    )?;
    
    // Draw text (white)
    imgproc::put_text(
        dst,
        s,
        Point::new(x, y),
        FONT_HERSHEY_PLAIN,
        font_scale,
        Scalar::new(255.0, 255.0, 255.0, 0.0),
        thickness,
        line_type,
        false,
    )?;
    
    Ok(())
}

/// Gets the size of an image (width, height)
pub fn getsize(img: &Mat) -> (i32, i32) {
    let size = img.size().unwrap();
    (size.width, size.height)
}

/// HSV color range for tracking
#[derive(Debug, Clone, Copy)]
pub struct HsvRange {
    pub h_min: u8,
    pub h_max: u8,
    pub s_min: u8,
    pub s_max: u8,
    pub v_min: u8,
    pub v_max: u8,
}

impl HsvRange {
    pub fn new(h_min: u8, h_max: u8, s_min: u8, s_max: u8, v_min: u8, v_max: u8) -> Self {
        Self {
            h_min,
            h_max,
            s_min,
            s_max,
            v_min,
            v_max,
        }
    }

    /// Default HSV range for orange paintball detection
    pub fn default_orange() -> Self {
        Self {
            h_min: 10,
            h_max: 40,
            s_min: 70,
            s_max: 255,
            v_min: 150,
            v_max: 255,
        }
    }

    pub fn lower_scalar(&self) -> Scalar {
        Scalar::new(self.h_min as f64, self.s_min as f64, self.v_min as f64, 0.0)
    }

    pub fn upper_scalar(&self) -> Scalar {
        Scalar::new(self.h_max as f64, self.s_max as f64, self.v_max as f64, 0.0)
    }
}

impl Default for HsvRange {
    fn default() -> Self {
        Self::default_orange()
    }
}

/// Result of contour analysis
#[derive(Debug, Clone)]
pub struct ContourResult {
    pub center_x: i32,
    pub center_y: i32,
    pub area: f64,
}

/// Finds the largest contour and calculates its centroid
/// 
/// # Arguments
/// 
/// * `contours` - Vector of contours
/// 
/// # Returns
/// 
/// Optional ContourResult with center coordinates and area
pub fn find_largest_contour(contours: &Vector<Vector<Point>>) -> Option<ContourResult> {
    let mut max_area: f64 = 0.0;
    let mut best_cnt: Option<&Vector<Point>> = None;

    for cnt in contours.iter() {
        let area = imgproc::contour_area(&cnt, false).unwrap_or(0.0);
        if area > max_area {
            max_area = area;
            best_cnt = Some(cnt);
        }
    }

    if let Some(cnt) = best_cnt {
        let moments = imgproc::moments(&cnt, false).ok()?;
        
        if moments.m00 > 0.0 {
            let cx = (moments.m10 / moments.m00) as i32;
            let cy = (moments.m01 / moments.m00) as i32;
            
            Some(ContourResult {
                center_x: cx,
                center_y: cy,
                area: max_area,
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// Application preferences/configuration
#[derive(Debug, Clone)]
pub struct Preferences {
    pub arduino_enabled: bool,
    pub screen_enabled: bool,
    pub target_lead_enabled: bool,
    pub write_video_enabled: bool,
}

impl Preferences {
    pub fn new() -> Self {
        Self {
            arduino_enabled: false,
            screen_enabled: true,
            target_lead_enabled: true,
            write_video_enabled: false,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new()
    }
}

/// Arduino serial communication settings
pub struct ArduinoConfig {
    pub port: String,
    pub baud_rate: u32,
}

impl ArduinoConfig {
    pub fn new() -> Self {
        Self {
            port: "/dev/ttyACM0".to_string(),
            baud_rate: 115200,
        }
    }
}

impl Default for ArduinoConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bunch() {
        let mut b = Bunch::new();
        b.set("key1", "value1");
        assert_eq!(b.get("key1"), Some(&"value1".to_string()));
        assert_eq!(b.get("key2"), None);
    }

    #[test]
    fn test_stat_value() {
        let mut sv = StatValue::new(0.5);
        assert_eq!(sv.value, None);
        
        sv.update(10.0);
        assert_eq!(sv.value, Some(10.0));
        
        sv.update(20.0);
        assert_eq!(sv.value, Some(15.0)); // 0.5 * 10 + 0.5 * 20
    }

    #[test]
    fn test_hsv_range() {
        let range = HsvRange::default_orange();
        assert_eq!(range.h_min, 10);
        assert_eq!(range.h_max, 40);
        assert_eq!(range.s_min, 70);
        assert_eq!(range.s_max, 255);
        assert_eq!(range.v_min, 150);
        assert_eq!(range.v_max, 255);
    }

    #[test]
    fn test_preferences() {
        let prefs = Preferences::new();
        assert!(!prefs.arduino_enabled);
        assert!(prefs.screen_enabled);
        assert!(prefs.target_lead_enabled);
        assert!(!prefs.write_video_enabled);
    }

    #[test]
    fn test_arduino_config() {
        let config = ArduinoConfig::new();
        assert_eq!(config.port, "/dev/ttyACM0");
        assert_eq!(config.baud_rate, 115200);
    }

    #[test]
    fn test_clock() {
        let t1 = clock();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = clock();
        assert!(t2 > t1);
    }
}