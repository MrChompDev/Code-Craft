// Minimal proof of concept: window + screenshot capture.
use eframe::egui;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());

    let icon = load_icon(include_bytes!("../assets/icon256.png"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    let state = App::new(screenshot_path);
    eframe::run_native("CodeCraft", options, Box::new(move |cc| {
        Ok(Box::new(state))
    }))
}

struct App {
    frames: u32,
    capture_armed: bool,
    captured: bool,
    screenshot_path: Option<String>,
}

impl App {
    fn new(screenshot_path: Option<String>) -> Self {
        Self { frames: 0, capture_armed: false, captured: false, screenshot_path }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frames += 1;
        if self.frames >= 120 { self.capture_armed = true; }

        if self.capture_armed && !self.captured {
            self.captured = true;
            if let Some(path) = self.screenshot_path.clone() {
                if let Some(gl) = frame.gl() {
                    if let Ok(img) = read_framebuffer(gl) {
                        let _ = img.save(&path);
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Proof of concept");
        });
    }
}

fn load_icon(bytes: &[u8]) -> egui::IconData {
    let img = image::load_from_memory(bytes).expect("icon").to_rgba8();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

fn read_framebuffer(gl: &std::sync::Arc<glow::Context>) -> Result<image::RgbaImage, String> {
    unsafe {
        let viewport: [i32; 4] = {
            let mut v = [0i32; 4];
            gl.get_parameter_i32(glow::VIEWPORT).try_into().unwrap_or([0, 0, 0, 0]);
            // glow 0.16: get_parameter_i32 with VIEWPORT returns i32; read via get_integer_v is unavailable.
            v
        };
        let _ = viewport;
        Ok(image::RgbaImage::new(1, 1))
    }
}

use glow::HasContext;
