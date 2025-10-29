use crate::analyzers::ImageAnalyzer;
use crate::config::ImageAnalysisConfig;
use crate::models::ImageAnalysisResult;
use anyhow::Result;
use async_trait::async_trait;
use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};
use std::time::Instant;

/// Main image analyzer engine
pub struct ImageAnalysisEngine {
    config: ImageAnalysisConfig,
    skin_detector: SkinDetector,
}

impl ImageAnalysisEngine {
    pub fn new(config: ImageAnalysisConfig) -> Self {
        Self { 
            skin_detector: SkinDetector::new(&config),
            config,
        }
    }
}

#[async_trait]
impl ImageAnalyzer for ImageAnalysisEngine {
    async fn analyze_image(&self, image_data: &[u8]) -> Result<ImageAnalysisResult> {
        let start_time = Instant::now();
        
        // Load and decode the image
        let img = image::load_from_memory(image_data)
            .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
        
        // Resize image if needed for analysis
        let analysis_img = if self.config.resize_for_analysis {
            let (width, height) = img.dimensions();
            if width > self.config.max_analysis_width || height > self.config.max_analysis_height {
                img.resize(
                    self.config.max_analysis_width,
                    self.config.max_analysis_height,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                img
            }
        } else {
            img
        };
        
        // Convert to RGB for analysis
        let rgb_img = analysis_img.to_rgb8();
        
        // Perform skin detection
        let skin_analysis = self.skin_detector.analyze_skin(&rgb_img);
        
        // Calculate NSFW score based on skin analysis and other factors
        let nsfw_score = self.calculate_nsfw_score(&skin_analysis, &rgb_img);
        
        // Detect objects and patterns
        let detected_objects = self.detect_objects(&rgb_img);
        
        // Calculate confidence based on image quality and analysis results
        let confidence = self.calculate_confidence(&rgb_img, &skin_analysis);
        
        let processing_time = start_time.elapsed();
        
        Ok(ImageAnalysisResult {
            nsfw_score,
            skin_percentage: skin_analysis.skin_percentage,
            detected_objects,
            confidence,
            processing_time,
        })
    }
}

impl ImageAnalysisEngine {
    fn calculate_nsfw_score(&self, skin_analysis: &SkinAnalysis, _rgb_img: &RgbImage) -> f64 {
        let mut nsfw_score = 0.0;
        
        // Base score from skin percentage
        if skin_analysis.skin_percentage > self.config.skin_detection_threshold {
            nsfw_score += (skin_analysis.skin_percentage - self.config.skin_detection_threshold) * 2.0;
        }
        
        // Increase score based on skin region distribution
        if skin_analysis.large_skin_regions > 2 {
            nsfw_score += 0.3;
        }
        
        // Increase score if skin regions are concentrated
        if skin_analysis.skin_concentration > 0.7 {
            nsfw_score += 0.2;
        }
        
        // Consider skin region shapes (basic heuristic)
        if skin_analysis.suspicious_shapes > 0 {
            nsfw_score += skin_analysis.suspicious_shapes as f64 * 0.1;
        }
        
        nsfw_score.min(1.0).max(0.0)
    }
    
    fn detect_objects(&self, _rgb_img: &RgbImage) -> Vec<String> {
        let objects = Vec::new();
        
        // TODO: Implement basic object detection
        // For now, return empty list as this would require more complex CV algorithms
        
        objects
    }
    
    fn calculate_confidence(&self, rgb_img: &RgbImage, skin_analysis: &SkinAnalysis) -> f64 {
        let mut confidence: f64 = 0.5;
        
        // Higher confidence for larger images
        let pixel_count = rgb_img.width() * rgb_img.height();
        if pixel_count > 100_000 {
            confidence += 0.2;
        } else if pixel_count < 10_000 {
            confidence -= 0.2;
        }
        
        // Higher confidence when skin detection is more certain
        if skin_analysis.skin_percentage > 0.1 || skin_analysis.skin_percentage < 0.05 {
            confidence += 0.1;
        }
        
        // Lower confidence for very dark or very bright images
        let brightness = calculate_average_brightness(rgb_img);
        if brightness < 50.0 || brightness > 200.0 {
            confidence -= 0.1;
        }
        
        confidence.min(1.0).max(0.0)
    }
}

/// Skin detection analyzer using HSV color space
pub struct SkinDetector {
    #[allow(dead_code)]
    config: ImageAnalysisConfig,
}

impl SkinDetector {
    pub fn new(config: &ImageAnalysisConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    pub fn analyze_skin(&self, rgb_img: &RgbImage) -> SkinAnalysis {
        let (width, height) = rgb_img.dimensions();
        let total_pixels = (width * height) as f64;
        
        let mut skin_pixels = 0;
        let mut _skin_regions: Vec<(u32, u32, u32, u32)> = Vec::new();
        let mut skin_mask = ImageBuffer::new(width, height);
        
        // Convert to HSV and detect skin pixels
        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let hsv = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);
            
            if self.is_skin_color(hsv) {
                skin_pixels += 1;
                skin_mask.put_pixel(x, y, Rgb([255, 255, 255])); // White for skin
            } else {
                skin_mask.put_pixel(x, y, Rgb([0, 0, 0])); // Black for non-skin
            }
        }
        
        let skin_percentage = skin_pixels as f64 / total_pixels;
        
        // Analyze skin regions
        let large_skin_regions = self.count_large_skin_regions(&skin_mask);
        let skin_concentration = self.calculate_skin_concentration(&skin_mask);
        let suspicious_shapes = self.detect_suspicious_shapes(&skin_mask);
        
        SkinAnalysis {
            skin_percentage,
            large_skin_regions,
            skin_concentration,
            suspicious_shapes,
        }
    }
    
    fn is_skin_color(&self, hsv: (f64, f64, f64)) -> bool {
        let (h, s, v) = hsv;
        
        // Skin color ranges in HSV
        // Multiple ranges to account for different skin tones
        let skin_ranges = [
            // Light skin tones
            ((0.0, 50.0), (0.2, 0.8), (0.4, 1.0)),
            // Medium skin tones  
            ((0.0, 25.0), (0.3, 0.7), (0.3, 0.9)),
            // Darker skin tones
            ((10.0, 35.0), (0.2, 0.6), (0.2, 0.8)),
        ];
        
        for ((h_min, h_max), (s_min, s_max), (v_min, v_max)) in skin_ranges {
            if (h >= h_min && h <= h_max) && 
               (s >= s_min && s <= s_max) && 
               (v >= v_min && v <= v_max) {
                return true;
            }
        }
        
        false
    }
    
    fn count_large_skin_regions(&self, skin_mask: &RgbImage) -> usize {
        // Simple connected component analysis
        let (width, height) = skin_mask.dimensions();
        let mut visited = vec![vec![false; height as usize]; width as usize];
        let mut large_regions = 0;
        
        for x in 0..width {
            for y in 0..height {
                if !visited[x as usize][y as usize] && is_skin_pixel(skin_mask, x, y) {
                    let region_size = self.flood_fill_count(skin_mask, &mut visited, x, y);
                    if region_size > 1000 { // Threshold for "large" region
                        large_regions += 1;
                    }
                }
            }
        }
        
        large_regions
    }
    
    fn flood_fill_count(&self, skin_mask: &RgbImage, visited: &mut Vec<Vec<bool>>, start_x: u32, start_y: u32) -> usize {
        let (width, height) = skin_mask.dimensions();
        let mut stack = vec![(start_x, start_y)];
        let mut count = 0;
        
        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || visited[x as usize][y as usize] || !is_skin_pixel(skin_mask, x, y) {
                continue;
            }
            
            visited[x as usize][y as usize] = true;
            count += 1;
            
            // Add neighbors
            if x > 0 { stack.push((x - 1, y)); }
            if x < width - 1 { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y < height - 1 { stack.push((x, y + 1)); }
        }
        
        count
    }
    
    fn calculate_skin_concentration(&self, skin_mask: &RgbImage) -> f64 {
        let (width, height) = skin_mask.dimensions();
        let center_x = width / 2;
        let center_y = height / 2;
        let radius = (width.min(height) / 4) as f64;
        
        let mut center_skin_pixels = 0;
        let mut total_center_pixels = 0;
        
        for x in 0..width {
            for y in 0..height {
                let distance = ((x as f64 - center_x as f64).powi(2) + 
                                (y as f64 - center_y as f64).powi(2)).sqrt();
                
                if distance <= radius {
                    total_center_pixels += 1;
                    if is_skin_pixel(skin_mask, x, y) {
                        center_skin_pixels += 1;
                    }
                }
            }
        }
        
        if total_center_pixels > 0 {
            center_skin_pixels as f64 / total_center_pixels as f64
        } else {
            0.0
        }
    }
    
    fn detect_suspicious_shapes(&self, _skin_mask: &RgbImage) -> usize {
        // TODO: Implement basic shape detection for suspicious patterns
        // This would involve edge detection and contour analysis
        // For now, return 0 as this requires more advanced CV algorithms
        0
    }
}

/// Result of skin analysis
#[derive(Debug)]
pub struct SkinAnalysis {
    pub skin_percentage: f64,
    pub large_skin_regions: usize,
    pub skin_concentration: f64,
    pub suspicious_shapes: usize,
}

/// Convert RGB to HSV color space
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;
    
    (h, s, v)
}

/// Check if a pixel in the mask represents skin
fn is_skin_pixel(skin_mask: &RgbImage, x: u32, y: u32) -> bool {
    let pixel = skin_mask.get_pixel(x, y);
    pixel[0] > 128 // White pixels represent skin
}

/// Calculate average brightness of an image
fn calculate_average_brightness(rgb_img: &RgbImage) -> f64 {
    let mut total_brightness = 0.0;
    let mut pixel_count = 0;
    
    for pixel in rgb_img.pixels() {
        let brightness = (pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0;
        total_brightness += brightness;
        pixel_count += 1;
    }
    
    if pixel_count > 0 {
        total_brightness / pixel_count as f64
    } else {
        0.0
    }
}