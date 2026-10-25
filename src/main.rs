
use eframe::egui;
use eframe::egui::Pos2;

mod model;
mod raster;
use raster::Raster;
use model::Scene;
use crate::model::Polygon;
use crate::model::Point;

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

// co aktualnie trzymamy myszą (indeksy, jak w EdgeRef)
#[derive(Clone, Copy)] // AI-gen: pozwala skopiować wartość w `match self.dragged`
enum Dragged { Vertex { polygon: usize, vertex: usize }, Polygon { polygon: usize } }

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
    dragged: Option<Dragged>,
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
            dragged: None,
        }
    }
}

impl eframe::App for MyApp {
    // AI-gen (nowa kolejność: płótno -> mysz -> rysowanie)
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. przyciski
        ui.radio_value(&mut self.drawing_alg, DrawingAlg::Bresenham, "Bresenham");
        ui.radio_value(&mut self.drawing_alg, DrawingAlg::Library, "Library");

        // 2. rezerwujemy miejsce na płótno; dostajemy prostokąt i informację o myszy
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(WIDTH as f32, HEIGHT as f32),
            egui::Sense::click_and_drag(),
        );
        let origin = rect.min; // lewy górny róg płótna na ekranie

        // 3. mysz zmienia scenę, ZANIM ją narysujemy
        self.handle_mouse(ui, &response, origin);

        // 4. Bresenham rysuje do bufora PRZED wysłaniem go do tekstury
        self.raster.raster_clear([255, 255, 255, 255]);
        if self.drawing_alg == DrawingAlg::Bresenham {
            draw_scene_raster(&self.scene, &mut self.raster, [0, 0, 0, 255]);
        }

        // 5. bufor -> tekstura -> narysowana w zarezerwowanym prostokącie // tu sie dzieje cos z buforowaniem
        // tego obrazu i ogolnie skomplikowane rzeczy
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [self.raster.width, self.raster.height],
            &self.raster.pixels,
        );
        let opts = egui::TextureOptions::NEAREST;
        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("canvas", image.clone(), opts)
        });
        texture.set(image, opts);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)); // cała tekstura
        ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);

        // 6. biblioteka rysuje painterem PO obrazku, inaczej obrazek by ją zasłonił
        if self.drawing_alg == DrawingAlg::Library {
            draw_scene_lib(&self.scene, ui.painter(), origin, [255, 0, 0, 255]);
        }

        // rysowanie wierzcholkow
        for pol in &self.scene.polygons {
            draw_vertices(&pol, ui.painter(), origin);
        }
    }
}

impl MyApp {
    // AI-gen: przeciąganie wierzchołka (LPM) albo całego wielokąta (Shift + LPM na wierzchołku)
    fn handle_mouse(&mut self, ui: &egui::Ui, response: &egui::Response, origin: Pos2) {
        // początek przeciągania: co jest pod miejscem, gdzie wcisnęliśmy przycisk?
        if response.drag_started() {
            if let Some(press) = ui.input(|i| i.pointer.press_origin()) {
                let p = screen_to_scene(press, origin); // punkt klikniecia apki tlumaczony na punkt klikniecia na canvas
                let shift = ui.input(|i| i.modifiers.shift); // klawisz shift
                match self.scene.find_vertex(p, 2.0 * VERTEX_RADIUS as f64) {
                    Some((polygon, vertex)) => {
                        if shift {
                            self.dragged = Some(Dragged::Polygon { polygon });
                        } else {
                            self.dragged = Some(Dragged::Vertex { polygon, vertex });
                        }
                    }
                    None => self.dragged = None,
                }
            }
        }

        // w trakcie przeciągania: przesuwamy to, co trzymamy
        if response.dragged() {
            match self.dragged {
                Some(Dragged::Vertex { polygon, vertex }) => {
                    // wierzchołek idzie DOKŁADNIE za kursorem
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.scene.polygons[polygon].vertices[vertex].pos = screen_to_scene(pos, origin);
                    }
                }
                Some(Dragged::Polygon { polygon }) => {
                    let d = response.drag_delta(); // ruch myszy od poprzedniej klatki
                    self.scene.polygons[polygon].translate(d.x as f64, d.y as f64);
                }
                None => {}
            }
        }

        // koniec przeciągania: puszczamy
        if response.drag_stopped() {
            self.dragged = None;
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

// AI-gen: odwrotność origin + vec2(...) z draw_vertices
fn screen_to_scene(pos: Pos2, origin: Pos2) -> Point {
    Point {
        x: (pos.x - origin.x) as f64,
        y: (pos.y - origin.y) as f64,
    }
}



