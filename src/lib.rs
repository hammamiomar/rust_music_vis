#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod audio_processor;
pub use app::AudioVisualizerApp;


#[cfg(test)]
mod tests{
    use rodio::cpal::traits::*;


    #[test]
    #[ignore]
    fn show_default_output_device(){
        use rodio::cpal;

        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            println!("{}", device.name().unwrap_or("Unknown device name".to_string()));
        } else {
            println!("No output device available");
        }
    }
    #[test]
    fn load_audio(){
        use aus::read;
        let file = read("assets/strippers.mp3").unwrap();
        println!("{}",file.duration);
    }
}
