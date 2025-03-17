use aus::{self, WindowType, read, spectrum};
use image::{ImageBuffer, Rgb};
use egui::ColorImage;

/// Loads an audio file and creates a spectrogram suitable for display in egui
pub fn create_spectrogram_from_audio(
    file_path: &str, 
    fft_size: usize,
    normalize: bool,
    colormap: SpectrogramColormap
) -> Result<egui::ColorImage, String> {
    // Step 1: Load the audio file (MP3 or any other supported format)
    let mut audio = match read(file_path) {
        Ok(audio) => audio,
        Err(e) => return Err(format!("Failed to load audio: {:?}", e))
    };
    
    // If the audio has multiple channels, mix down to mono for spectrogram
    if audio.num_channels > 1 {
        aus::mixdown(&mut audio);
    }
    
    // Step 2: Compute the STFT
    let hop_size = fft_size / 2; // 50% overlap is typical for visualization
    let window_type = WindowType::Hanning;
    let stft = spectrum::rstft(&audio.samples[0], fft_size, hop_size, window_type);
    
    // Step 3: Generate the magnitude spectrogram (discard phase information)
    let (magnitude_spectrogram, _) = spectrum::complex_to_polar_rstft(&stft);
    
    // Convert magnitude to dB scale for better visualization
    let db_spectrogram = magnitude_to_db(&magnitude_spectrogram, -120.0, normalize);
    
    // Step 4: Convert spectrogram data to an image
    let img = spectrogram_to_image(&db_spectrogram, colormap);
    
    // Step 5: Convert the image to an egui texture
    let color_image = convert_image_to_egui_image(img);
    
    // Return the texture handle that can be used in egui
    Ok(color_image)
}

/// Converts magnitude values to decibels with specified floor and optional normalization
fn magnitude_to_db(
    magnitude_spectrogram: &[Vec<f64>], 
    db_floor: f64,
    normalize: bool
) -> Vec<Vec<f64>> {
    let mut db_spectrogram = Vec::with_capacity(magnitude_spectrogram.len());
    
    // Find the maximum value for normalization if requested
    let max_val = if normalize {
        magnitude_spectrogram.iter()
            .flat_map(|frame| frame.iter())
            .fold(0.0_f64, |max, &val| f64::max(max, val))
    } else {
        1.0
    };
    
    for frame in magnitude_spectrogram {
        let mut db_frame = Vec::with_capacity(frame.len());
        
        for &mag in frame {
            // Convert magnitude to dB, with a floor value
            let normalized_mag = mag / max_val;
            let db = if normalized_mag > 0.0 {
                20.0 * normalized_mag.log10()
            } else {
                db_floor
            };
            
            // Clip to floor
            let clipped_db = f64::max(db, db_floor);
            
            db_frame.push(clipped_db);
        }
        
        db_spectrogram.push(db_frame);
    }
    
    // Normalize the dB values to [0, 1] range
    if normalize {
        let min_db = db_floor;
        let max_db = 0.0; // 0 dB is the maximum for normalized values
        
        for frame in &mut db_spectrogram {
            for db in frame.iter_mut() {
                *db = (*db - min_db) / (max_db - min_db);
            }
        }
    }
    
    db_spectrogram
}

/// Available colormaps for spectrogram visualization
pub enum SpectrogramColormap {
    Viridis,
    Magma,
    Inferno,
    Grayscale,
    BlueToRed,
}

/// Converts spectrogram data to an RGB image
fn spectrogram_to_image(
    spectrogram: &[Vec<f64>],
    colormap: SpectrogramColormap
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let height = spectrogram[0].len(); // Number of frequency bins
    let width = spectrogram.len();     // Number of time frames
    
    let mut img = ImageBuffer::new(width as u32, height as u32);
    
    for (x, frame) in spectrogram.iter().enumerate() {
        for (y, &value) in frame.iter().enumerate() {
            // Invert y-axis so lower frequencies are at the bottom
            let y_inv = height - 1 - y;
            
            // Get the color based on the chosen colormap
            let color = match colormap {
                SpectrogramColormap::Viridis => viridis_colormap(value),
                SpectrogramColormap::Magma => magma_colormap(value),
                SpectrogramColormap::Inferno => inferno_colormap(value),
                SpectrogramColormap::Grayscale => grayscale_colormap(value),
                SpectrogramColormap::BlueToRed => blue_to_red_colormap(value),
            };
            
            img.put_pixel(x as u32, y_inv as u32, Rgb(color));
        }
    }
    
    img
}

/// Convert an image::ImageBuffer to egui::ColorImage
fn convert_image_to_egui_image(img: ImageBuffer<Rgb<u8>, Vec<u8>>) -> egui::ColorImage {
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    // Convert the image data to egui format
    let pixels = img.into_raw();
    let mut egui_pixels = Vec::with_capacity(width * height * 4);
    
    // Convert RGB to RGBA (egui expects RGBA)
    for chunk in pixels.chunks(3) {
        egui_pixels.push(chunk[0]); // R
        egui_pixels.push(chunk[1]); // G
        egui_pixels.push(chunk[2]); // B
        egui_pixels.push(255);      // A (fully opaque)
    }
    
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width, height],
        &egui_pixels
    );
    
    color_image
}

// Colormap implementations - these convert a value in range [0, 1] to RGB

fn viridis_colormap(value: f64) -> [u8; 3] {
    // Simplified Viridis colormap (actual implementation has more complex interpolation)
    let v = value.clamp(0.0, 1.0);
    
    if v < 0.25 {
        let t = v / 0.25;
        return [
            (68.0 * (1.0 - t) + 33.0 * t) as u8,
            (1.0 * (1.0 - t) + 144.0 * t) as u8,
            (84.0 * (1.0 - t) + 140.0 * t) as u8,
        ];
    } else if v < 0.5 {
        let t = (v - 0.25) / 0.25;
        return [
            (33.0 * (1.0 - t) + 73.0 * t) as u8,
            (144.0 * (1.0 - t) + 211.0 * t) as u8,
            (140.0 * (1.0 - t) + 121.0 * t) as u8,
        ];
    } else if v < 0.75 {
        let t = (v - 0.5) / 0.25;
        return [
            (73.0 * (1.0 - t) + 190.0 * t) as u8,
            (211.0 * (1.0 - t) + 206.0 * t) as u8,
            (121.0 * (1.0 - t) + 86.0 * t) as u8,
        ];
    } else {
        let t = (v - 0.75) / 0.25;
        return [
            (190.0 * (1.0 - t) + 253.0 * t) as u8,
            (206.0 * (1.0 - t) + 231.0 * t) as u8,
            (86.0 * (1.0 - t) + 37.0 * t) as u8,
        ];
    }
}

fn magma_colormap(value: f64) -> [u8; 3] {
    // Simplified Magma colormap
    let v = value.clamp(0.0, 1.0);
    
    if v < 0.25 {
        let t = v / 0.25;
        return [
            (0.0 * (1.0 - t) + 88.0 * t) as u8,
            (0.0 * (1.0 - t) + 24.0 * t) as u8,
            (0.0 * (1.0 - t) + 69.0 * t) as u8,
        ];
    } else if v < 0.5 {
        let t = (v - 0.25) / 0.25;
        return [
            (88.0 * (1.0 - t) + 188.0 * t) as u8,
            (24.0 * (1.0 - t) + 80.0 * t) as u8,
            (69.0 * (1.0 - t) + 144.0 * t) as u8,
        ];
    } else if v < 0.75 {
        let t = (v - 0.5) / 0.25;
        return [
            (188.0 * (1.0 - t) + 249.0 * t) as u8,
            (80.0 * (1.0 - t) + 163.0 * t) as u8,
            (144.0 * (1.0 - t) + 137.0 * t) as u8,
        ];
    } else {
        let t = (v - 0.75) / 0.25;
        return [
            (249.0 * (1.0 - t) + 253.0 * t) as u8,
            (163.0 * (1.0 - t) + 231.0 * t) as u8,
            (137.0 * (1.0 - t) + 240.0 * t) as u8,
        ];
    }
}

fn inferno_colormap(value: f64) -> [u8; 3] {
    // Simplified Inferno colormap
    let v = value.clamp(0.0, 1.0);
    
    if v < 0.25 {
        let t = v / 0.25;
        return [
            (0.0 * (1.0 - t) + 73.0 * t) as u8,
            (0.0 * (1.0 - t) + 11.0 * t) as u8,
            (0.0 * (1.0 - t) + 68.0 * t) as u8,
        ];
    } else if v < 0.5 {
        let t = (v - 0.25) / 0.25;
        return [
            (73.0 * (1.0 - t) + 184.0 * t) as u8,
            (11.0 * (1.0 - t) + 71.0 * t) as u8,
            (68.0 * (1.0 - t) + 55.0 * t) as u8,
        ];
    } else if v < 0.75 {
        let t = (v - 0.5) / 0.25;
        return [
            (184.0 * (1.0 - t) + 253.0 * t) as u8,
            (71.0 * (1.0 - t) + 173.0 * t) as u8,
            (55.0 * (1.0 - t) + 47.0 * t) as u8,
        ];
    } else {
        let t = (v - 0.75) / 0.25;
        return [
            (253.0 * (1.0 - t) + 252.0 * t) as u8,
            (173.0 * (1.0 - t) + 255.0 * t) as u8,
            (47.0 * (1.0 - t) + 164.0 * t) as u8,
        ];
    }
}

fn grayscale_colormap(value: f64) -> [u8; 3] {
    // Simple grayscale colormap
    let v = (value.clamp(0.0, 1.0) * 255.0) as u8;
    [v, v, v]
}

fn blue_to_red_colormap(value: f64) -> [u8; 3] {
    // Simple blue to red through purple colormap
    let v = value.clamp(0.0, 1.0);
    let r = (v * 255.0) as u8;
    let b = ((1.0 - v) * 255.0) as u8;
    let g = 0;
    [r, g, b]
}