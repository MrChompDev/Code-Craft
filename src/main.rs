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
    eframe::run_native("CodeCraft", options, Box::new(move |_| Ok(Box::new(state))))
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
        if self.frames >= 120 {
            self.capture_armed = true;
        }

        if self.capture_armed && !self.captured {
            self.captured = true;
            if let Some(path) = self.screenshot_path.clone() {
                if let Some(gl) = frame.gl() {
                    if let Some(img) = read_framebuffer(ctx, gl) {
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

fn read_framebuffer(ctx: &egui::Context, gl: &std::sync::Arc<glow::Context>) -> Option<image::RgbaImage> {
    use glow::HasContext;
    unsafe {
        let rect = ctx.screen_rect();
        let ppp = ctx.pixels_per_point();
        let w = (rect.width() * ppp).round().max(1.0) as i32;
        let h = (rect.height() * ppp).round().max(1.0) as i32;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        gl.read_pixels(0, 0, w, h, glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(Some(&mut buf)));
        let mut img = image::RgbaImage::new(w as u32, h as u32);
        for y in 0..h as u32 {
            let src = ((h as u32 - 1 - y) * w as u32 * 4) as usize;
            for x in 0..w as u32 {
                let s = src + (x * 4) as usize;
                img.put_pixel(x, y, image::Rgba([buf[s], buf[s + 1], buf[s + 2], 255]));
            }
        }
        Some(img)
    }
}
