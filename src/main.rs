
use eframe::egui;
use eframe::egui::Pos2;

mod model;
mod raster;
use raster::Raster;
use model::Scene;
use crate::model::Polygon;

const WIDTH: usize = 900;
const HEIGHT: usize = 600;
const APP_NAME: &str = "Bezier-Poligon-Editor";
const VERTEX_RADIUS: f32 = 5.0;
const VERTEX_FILL_COLOR: egui::Color32 = egui::Color32::CYAN;

#[derive(PartialEq)]
enum DrawingAlg {
    Library,
    Bresenham,
}

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

// zmienne w apce
struct MyApp {
    raster: Raster,
    texture: Option<egui::TextureHandle>,
    drawing_alg: DrawingAlg, // czy używać algorytmu biblioteki
    scene: Scene, // Scena
}

impl Default for MyApp {
    // co sie dzieje przy wywolaniu apki
    fn default() -> Self {
        let mut raster = Raster::new(WIDTH, HEIGHT);

        Self {
            raster,
            texture: None,
            drawing_alg: DrawingAlg::Bresenham,
            scene: Scene::demo(),
        }
    }
}

impl eframe::App for MyApp {
    // response.rect.min - wzgledne rysowanie po canvasie
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.radio_value(&mut self.drawing_alg, DrawingAlg::Bresenham, "Bresenham");
        ui.radio_value(&mut self.drawing_alg, DrawingAlg::Library, "Library");

        // AI-gen: Bresenham rysuje do bufora PRZED wysłaniem go do tekstury
        self.raster.raster_clear([255, 255, 255, 255]);
        if self.drawing_alg == DrawingAlg::Bresenham {
            draw_scene_raster(&self.scene, &mut self.raster, [0, 0, 0, 255]);
        }

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [self.raster.width, self.raster.height],
            &self.raster.pixels,
        );
        let opts = egui::TextureOptions::NEAREST;
        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("canvas", image.clone(), opts)
        });
        texture.set(image, opts);
        let response = ui.image(&*texture);

        // AI-gen: biblioteka rysuje painterem PO obrazku, inaczej obrazek by ją zasłonił
        if self.drawing_alg == DrawingAlg::Library {
            draw_scene_lib(&self.scene, ui.painter(), response.rect.min, [255, 0, 0, 255]);
        }

        // rysowanie wierzcholkow
        for pol in &self.scene.polygons {
            draw_vertices(&pol, ui.painter(), response.rect.min);
        }
    }
}

// rysowanie sceny
// AI-gen (poprawiona pętla)
fn draw_scene_raster(scene: &Scene, raster: &mut Raster, color: [u8; 4]) {
    for polygon in &scene.polygons {
        draw_polygon_raster(polygon, raster, color);
    }
}

// na razie każda krawędź jako odcinek; match po EdgeKind dojdzie przy Bézierze
// AI-gen
fn draw_polygon_raster(polygon: &Polygon, raster: &mut Raster, color: [u8; 4]) {
    for edge in &polygon.edges {
        let a = polygon.vertices[edge.p1].pos;
        let b = polygon.vertices[edge.p2].pos;
        raster.line(
            a.x.round() as i32,
            a.y.round() as i32,
            b.x.round() as i32,
            b.y.round() as i32,
            color,
        );
    }
}

// wersja biblioteczna: rysuje painterem nad obrazkiem, origin = lewy górny róg obrazka
// AI-gen
fn draw_scene_lib(scene: &Scene, painter: &egui::Painter, origin: Pos2, color: [u8; 4]) {
    for polygon in &scene.polygons {
        draw_polygon_lib(polygon, painter, origin, color);
    }
}

// partly AI-gen
fn draw_polygon_lib(polygon: &Polygon, painter: &egui::Painter, origin: Pos2, color: [u8; 4]) {
    for edge in &polygon.edges {
        let a = polygon.vertices[edge.p1].pos;
        let b = polygon.vertices[edge.p2].pos;
        Raster::line_lib(
            painter,
            origin,
            a.x.round() as i32,
            a.y.round() as i32,
            b.x.round() as i32,
            b.y.round() as i32,
            color,
        );
    }
}

fn draw_vertices(polygon: &Polygon, painter: &egui::Painter, origin: Pos2) {
    for vertex in &polygon.vertices {
        painter.circle_filled(origin + egui::vec2(vertex.pos.x as f32, vertex.pos.y as f32), VERTEX_RADIUS, VERTEX_FILL_COLOR);
    }
}