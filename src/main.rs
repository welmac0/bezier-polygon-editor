
use eframe::egui;
mod model;
mod raster;
use raster::Raster;

const WIDTH: usize = 900;
const HEIGHT: usize = 600;
const APP_NAME: &str = "Bezier-Poligon-Editor";

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH as f32, HEIGHT as f32]),
        ..Default::default()
    };
    eframe::run_native(
        &APP_NAME, // title
        options, // options?
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    raster: Raster,
    texture: Option<egui::TextureHandle>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut raster = Raster::new(WIDTH, HEIGHT);
        for x in 10..100 {
            raster.set_pixel(x, 50, [255, 0, 0, 255]); // czerwona kreska na próbę
        };
        raster.bresenham(50, 50, 700, 500, [255, 0, 0, 255]);
        Self { raster, texture: None }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [self.raster.width, self.raster.height],
            &self.raster.pixels,
        );
        let opts = egui::TextureOptions::NEAREST;
        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("canvas", image.clone(), opts)
        });
        texture.set(image, opts);
        ui.image(&*texture);
    }
}