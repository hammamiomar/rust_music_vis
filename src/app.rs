
use crate::audio_processor;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioVisualizerApp {
    
    #[serde(skip)]
    selected_audio_path: Option<String>,

    #[serde(skip)]
    visualization_texture: Option<egui::TextureHandle>,

    #[serde(skip)]
    is_processing:bool,
}

impl Default for AudioVisualizerApp {
    fn default() -> Self {
        Self {
            selected_audio_path: None,
            visualization_texture: None,
            is_processing: false,
       }
    }
}

impl AudioVisualizerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    
    fn update_visualization(&mut self, ctx: &egui::Context){
        if let Some(path) = &self.selected_audio_path{
            self.is_processing = true;

            match audio_processor::create_spectrogram_from_audio(path,
                2048,
                true,
                audio_processor::SpectrogramColormap::Viridis){
                Ok(image) => {
                    self.visualization_texture = Some(self.create_texture(ctx,image));
                    self.is_processing = false;
                }
                Err(e) => {
                    eprintln!("Error processing audio file: {:?}",e);
                    self.is_processing = false;
                }
            }
        }
    }

    fn create_texture(&self, ctx: &egui::Context, image: egui::ColorImage) -> egui::TextureHandle{
        ctx.load_texture("audio_vis", image, egui::TextureOptions::default(),)
    }
}

impl eframe::App for AudioVisualizerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if ui.button("Open Audio File...").clicked(){
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio", &["mp3","wav","ogg","flac"])
                            .pick_file()
                        {
                            self.selected_audio_path = Some(path.display().to_string());
                            ui.close_menu();

                            self.update_visualization(ctx);
                        }
                    }
                    ui.separator();
                    if !is_web{
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                    ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Music Visualizer");

            if let Some(path) = &self.selected_audio_path{
                ui.horizontal(|ui|{
                    ui.label("Selected Audio:");
                    ui.monospace(path.split('/').last().unwrap_or(path));

                    });
                if self.is_processing{
                    ui.spinner();
                    ui.label("Processing Audio");
                } else if let Some(texture) = &self.visualization_texture{
                    ui.image(texture);
                }
            }else{
                // Prompt to select an audio file if none is selected
                ui.vertical_centered(|ui| {
                    ui.label("No audio file selected");
                    if ui.button("Select Audio File").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
                            .pick_file()
                        {
                            self.selected_audio_path = Some(path.display().to_string());
                            
                            // Process the audio file and update visualization
                            self.update_visualization(ctx);
                        }
                    }
                });
            }
            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

        });
    }
}
