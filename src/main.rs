
use eframe::egui;
use eframe::egui::Pos2;

mod model;
mod raster;
mod algorithms;
mod constraints; // AI-finished: ograniczenia, ciągłość, solver
mod docs; // AI-finished: dokumentacja w aplikacji

use raster::Raster;
use model::Scene;
use crate::algorithms::bezier_points;
use crate::model::{Constraint, EdgeKind, EdgeRef, PointKind, PointRef, Polygon};
use crate::model::Point;

const WIDTH: usize = 900;
const HEIGHT: usize = 600;
// AI-finished: stała szerokość prawego panelu (z marginesami) i zapas na marginesy / pasek nad płótnem;
// z nich liczony jest rozmiar okna
const PANEL_WIDTH: f32 = 300.0;
const WINDOW_MARGIN: f32 = 20.0;
const TOP_BAR_HEIGHT: f32 = 30.0;
const APP_NAME: &str = "Bezier-Poligon-Editor";
const VERTEX_RADIUS: f32 = 5.0;
const VERTEX_FILL_COLOR: egui::Color32 = egui::Color32::CYAN;
const BEZIER_STEPS: usize = 200;

#[derive(PartialEq)]
enum DrawingAlg {
    Library,
    Bresenham,
}

// co aktualnie trzymamy myszą (indeksy, jak w EdgeRef)
#[derive(Clone, Copy)] // AI-gen: pozwala skopiować wartość w `match self.dragged`
enum Dragged {
    Polygon {
        polygon: usize
    },
    Point(PointRef),
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([
            WIDTH as f32 + PANEL_WIDTH + WINDOW_MARGIN,     // AI-finished: płótno + panel + marginesy
            HEIGHT as f32 + TOP_BAR_HEIGHT + WINDOW_MARGIN, // AI-finished: płótno + pasek u góry + marginesy
        ]),
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
    selected_edge: Option<EdgeRef>,
    length_input: String,                    // AI-finished: pole tekstowe długości w panelu
    pending_perpendicular: Option<EdgeRef>,  // AI-finished: pierwsza krawędź, czekamy na kliknięcie drugiej
    message: String,                         // AI-finished: komunikat dla użytkownika (np. błąd)
    show_docs: bool,                         // AI-finished: czy okno dokumentacji jest otwarte
}

impl Default for MyApp {
    // co sie dzieje przy wywolaniu apki
    fn default() -> Self {
        let raster = Raster::new(WIDTH, HEIGHT);

        Self {
            raster,
            texture: None,
            drawing_alg: DrawingAlg::Bresenham,
            scene: Scene::demo(),
            dragged: None,
            selected_edge: None,
            length_input: String::new(),
            pending_perpendicular: None,
            message: String::new(),
            show_docs: false,
        }
    }
}

impl eframe::App for MyApp {
    // AI-gen (nowa kolejność: płótno -> mysz -> rysowanie)
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // panel
        egui::Panel::right("edge_panel")
            .resizable(false)        // AI-finished: użytkownik nie zmienia szerokości
            .exact_size(PANEL_WIDTH) // AI-finished: stała szerokość (razem z marginesami)
            .show(ui, |ui| {
            self.edge_panel(ui); // AI-finished: zawartość panelu w osobnej metodzie
        });
        // 1. przyciski
        // AI-finished: algorytm i dokumentacja w jednym wierszu (mniej miejsca nad płótnem)
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.drawing_alg, DrawingAlg::Bresenham, "Bresenham");
            ui.radio_value(&mut self.drawing_alg, DrawingAlg::Library, "Library");
            ui.separator();
            if ui.button("Dokumentacja").clicked() {
                self.show_docs = !self.show_docs;
            }
        });
        // AI-finished: dokumentacja w osobnym oknie systemowym (viewport), niezależnym od głównego.
        // show_viewport_immediate rysuje jego zawartość w tej samej klatce co główne okno.
        if self.show_docs {
            let ctx = ui.ctx().clone();
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("docs_window"),
                egui::ViewportBuilder::default()
                    .with_title("Dokumentacja")
                    .with_inner_size([780.0, 640.0]),
                |ui, _class| {
                    docs::show_docs(ui);
                    // krzyżyk w rogu okna -> przestajemy je pokazywać
                    if ui.input(|i| i.viewport().close_requested()) {
                        self.show_docs = false;
                    }
                },
            );
        }

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

        if let Some(e) = self.selected_edge {
            draw_selected_edge(&self.scene, e, ui.painter(), origin);
        }

        // AI-finished: ikony ograniczeń przy środkach krawędzi
        draw_constraint_icons(&self.scene, ui.painter(), origin);

        // rysowanie wierzcholkow
        for pol in &self.scene.polygons {
            draw_bezier_controls(&pol, ui.painter(), origin);
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
                // AI-gen: find_point zamiast find_vertex (łapie też punkty kontrolne)
                match self.scene.find_point(p, 2.0 * VERTEX_RADIUS as f64) {
                    Some(r) => {
                        if let PointRef::Vertex { polygon, .. } = r {
                            if shift {
                                // Shift + wierzchołek = cały wielokąt
                                self.dragged = Some(Dragged::Polygon { polygon });
                            } else {
                                self.dragged = Some(Dragged::Point(r));
                            }
                        } else {
                            self.dragged = Some(Dragged::Point(r));
                        }
                    }
                    None => self.dragged = None,
                }
            }
        }

        // w trakcie przeciągania: przesuwamy to, co trzymamy
        if response.dragged() {
            match self.dragged {
                Some(Dragged::Polygon { polygon }) => {
                    let d = response.drag_delta(); // ruch myszy od poprzedniej klatki
                    // AI-finished: nie pozwalamy wyjechać wielokątem poza płótno
                    let (dx, dy) = clamp_polygon_delta(&self.scene.polygons[polygon], d.x as f64, d.y as f64);
                    self.scene.polygons[polygon].translate(dx, dy);
                }
                Some(Dragged::Point(point)) => {
                    if let Some(pos) = response.interact_pointer_pos() {
                        // AI-finished: punkt zostaje na płótnie, nawet gdy kursor wyjedzie poza nie
                        self.scene.move_point(point, clamp_to_canvas(screen_to_scene(pos, origin)));
                    }
                }
                None => {}
            }
        }

        // ppm dodanie / usuwanie wierzcholkow
        if response.secondary_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let p = screen_to_scene(pos, origin);
                if let Some((pol_id, vertex_id)) = self.scene.find_vertex(p, VERTEX_RADIUS as f64) {
                    self.scene.remove_vertex(pol_id, vertex_id);
                } else if let Some(e_ref) = self.scene.find_edge(p, VERTEX_RADIUS as f64) {
                self.scene.insert_vertex(e_ref)
                }
            }
            self.dragged = None;
            self.selected_edge = None;
            self.pending_perpendicular = None; // AI-finished: numery krawędzi mogły się zmienić
        }

        // lpm zaznaczenie krawedzi
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let p = screen_to_scene(pos, origin);
                if let Some(e_ref) = self.scene.find_edge(p, VERTEX_RADIUS as f64) {
                    // AI-finished: tryb "prostopadła do…" – drugie kliknięcie tworzy ograniczenie
                    if let Some(first) = self.pending_perpendicular {
                        if self.scene.add_perpendicular(first, e_ref) {
                            self.message.clear();
                        } else {
                            self.message = "Nie można dodać prostopadłości (tylko odcinki, różne krawędzie, ograniczenia nie mogą być sprzeczne)".to_string();
                        }
                        self.pending_perpendicular = None;
                    }
                    self.select_edge(Some(e_ref));
                } else {
                    self.select_edge(None);
                }
            }
        }

        // koniec przeciągania: puszczamy
        if response.drag_stopped() {
            self.dragged = None;
        }
    }

    // AI-finished: zaznaczenie krawędzi + wpisanie jej aktualnej długości do pola (domyślna wartość)
    fn select_edge(&mut self, e: Option<EdgeRef>) {
        self.selected_edge = e;
        if let Some(e) = e {
            let (a, b) = self.scene.polygons[e.polygon].edge_points(e.edge);
            self.length_input = format!("{:.1}", (b - a).length());
        }
    }

    // AI-finished: panel boczny – typ krawędzi, ograniczenia, ciągłość w jej końcach
    fn edge_panel(&mut self, ui: &mut egui::Ui) {
        if !self.message.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.message);
            ui.separator();
        }

        if let Some(first) = self.pending_perpendicular {
            ui.label(format!("Prostopadła do {}/{}: kliknij drugą krawędź (odcinek)", first.polygon, first.edge));
            if ui.button("Anuluj").clicked() {
                self.pending_perpendicular = None;
            }
            ui.separator();
        }

        let e = match self.selected_edge {
            Some(e) => e,
            None => {
                ui.label("Kliknij krawędź (LPM)");
                return;
            }
        };
        ui.label(format!("Wielokąt {}, krawędź {}", e.polygon, e.edge));
        ui.separator();

        // --- typ krawędzi ---
        ui.label("Typ krawędzi:");
        ui.horizontal(|ui| {
            if ui.button("Odcinek").clicked() {
                self.scene.set_line(e);
            }
            if ui.button("Bézier").clicked() {
                self.scene.set_bezier(e);
            }
            if ui.button("Łuk").clicked() {
                self.scene.set_arc(e);
            }
        });
        let kind = self.scene.polygons[e.polygon].edges[e.edge].kind.clone();
        if let EdgeKind::Arc { .. } = kind {
            if ui.button("Odwróć półokrąg (G0–G0)").clicked() {
                self.scene.flip_arc(e);
            }
        }
        ui.separator();

        // --- ograniczenia (tylko odcinki) ---
        ui.label("Ograniczenia:");
        if let EdgeKind::Line { len } = kind {
            ui.horizontal(|ui| {
                ui.label("Długość:");
                ui.add(egui::TextEdit::singleline(&mut self.length_input).desired_width(60.0));
                if ui.button("Ustaw").clicked() {
                    // przecinek też akceptujemy; parse::<f64>() zamienia tekst na liczbę (Result)
                    match self.length_input.trim().replace(',', ".").parse::<f64>() {
                        Ok(l) if l > 0.0 => {
                            if self.scene.set_length(e, l) {
                                self.message.clear();
                            } else {
                                self.message = "Nie da się ustawić tej długości (sprzeczne ograniczenia)".to_string();
                            }
                        }
                        _ => self.message = "Niepoprawna długość".to_string(),
                    }
                }
            });
            if let Some(l) = len {
                ui.horizontal(|ui| {
                    ui.label(format!("L = {:.1}", l));
                    if ui.button("Usuń").clicked() {
                        self.scene.remove_length(e);
                    }
                });
            }

            if ui.button("Prostopadła do…").clicked() {
                self.pending_perpendicular = Some(e);
                self.message.clear();
            }
            for ci in self.scene.constraints_of(e) {
                let other = self.scene.other_edge(ci, e);
                let mut removed = false;
                ui.horizontal(|ui| {
                    ui.label(format!("Prostopadła #{} do {}/{}", ci + 1, other.polygon, other.edge));
                    if ui.button("Usuń").clicked() {
                        self.scene.remove_constraint(ci);
                        removed = true;
                    }
                });
                if removed {
                    break; // numery ograniczeń się przesunęły – resztę pokażemy w następnej klatce
                }
            }
        } else {
            ui.label("(tylko dla odcinków)");
        }
        ui.separator();

        // --- ciągłość w obu końcach krawędzi ---
        ui.label("Ciągłość w końcach:");
        let (v1, v2) = {
            let edge = &self.scene.polygons[e.polygon].edges[e.edge];
            (edge.p1, edge.p2)
        };
        for v in [v1, v2] {
            self.continuity_row(ui, e.polygon, v);
        }
    }

    // AI-finished: jeden wiersz G0 / G1 / C1 dla wierzchołka v
    fn continuity_row(&mut self, ui: &mut egui::Ui, polygon: usize, v: usize) {
        let (has_bezier, has_arc) = self.scene.polygons[polygon].neighbor_kinds(v);
        let current = self.scene.polygons[polygon].vertices[v].kind;
        ui.horizontal(|ui| {
            ui.label(format!("w{}:", v));
            if !has_bezier && !has_arc {
                ui.label("– (tylko przy Bézierze / łuku)");
                return;
            }
            let mut options = vec![(PointKind::G0, "G0"), (PointKind::G1, "G1")];
            if has_bezier {
                options.push((PointKind::C1, "C1")); // łuk ma tylko G0/G1
            }
            for (k, name) in options {
                if ui.radio(current == Some(k), name).clicked() {
                    self.scene.set_vertex_kind(polygon, v, k);
                }
            }
        });
    }
}

// rysowanie sceny
// AI-gen (poprawiona pętla)
fn draw_scene_raster(scene: &Scene, raster: &mut Raster, color: [u8; 4]) {
    for polygon in &scene.polygons {
        draw_polygon_raster(polygon, raster, color);
    }
}

fn draw_polygon_raster(polygon: &Polygon, raster: &mut Raster, color: [u8; 4]) {
    for (k, edge) in polygon.edges.iter().enumerate() {
        match &edge.kind {
            EdgeKind::Bezier { c1, c2} => {
                let a = polygon.vertices[edge.p1].pos;
                let b = polygon.vertices[edge.p2].pos;

                let pts = bezier_points(&a, &b, c1, c2, BEZIER_STEPS);
                let n = pts.len();
                for i in 1..n {
                    raster.line(
                        pts[i-1].x.round() as i32,
                        pts[i-1].y.round() as i32,
                        pts[i].x.round() as i32,
                        pts[i].y.round() as i32,
                        color,
                    )
                }
            },
            EdgeKind::Line { len: _ } => {
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
            // AI-finished: łuk = łamana z punktów na okręgu, rysowana tym samym algorytmem odcinków
            EdgeKind::Arc { .. } => {
                let pts = polygon.edge_polyline(k, BEZIER_STEPS);
                for i in 1..pts.len() {
                    raster.line(
                        pts[i - 1].x.round() as i32,
                        pts[i - 1].y.round() as i32,
                        pts[i].x.round() as i32,
                        pts[i].y.round() as i32,
                        color,
                    );
                }
            }
        }
    }
}

// wersja biblioteczna: rysuje painterem nad obrazkiem, origin = lewy górny róg obrazka
// AI-gen
fn draw_scene_lib(scene: &Scene, painter: &egui::Painter, origin: Pos2, color: [u8; 4]) {
    for polygon in &scene.polygons {
        draw_polygon_lib(polygon, painter, origin, color);
    }
}

fn draw_polygon_lib(polygon: &Polygon, painter: &egui::Painter, origin: Pos2, color: [u8; 4]) {
    for (k, edge) in polygon.edges.iter().enumerate() {
        let a = polygon.vertices[edge.p1].pos;
        let b = polygon.vertices[edge.p2].pos;

        match &edge.kind {
            EdgeKind::Bezier { c1, c2 } => {
                let pts = bezier_points(&a, &b, c1, c2, BEZIER_STEPS);
                for i in 1..pts.len() {
                    Raster::line_lib(
                        painter,
                        origin,
                        pts[i - 1].x.round() as i32,
                        pts[i - 1].y.round() as i32,
                        pts[i].x.round() as i32,
                        pts[i].y.round() as i32,
                        color,
                    );
                }
            }
            EdgeKind::Line { len: _ } => {
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
            // AI-finished: łuk tak samo jak w wersji rastrowej
            EdgeKind::Arc { .. } => {
                let pts = polygon.edge_polyline(k, BEZIER_STEPS);
                for i in 1..pts.len() {
                    Raster::line_lib(
                        painter,
                        origin,
                        pts[i - 1].x.round() as i32,
                        pts[i - 1].y.round() as i32,
                        pts[i].x.round() as i32,
                        pts[i].y.round() as i32,
                        color,
                    );
                }
            }
        }
    }
}

fn draw_vertices(polygon: &Polygon, painter: &egui::Painter, origin: Pos2) {
    for vertex in &polygon.vertices {
        painter.circle_filled(origin + egui::vec2(vertex.pos.x as f32, vertex.pos.y as f32), VERTEX_RADIUS, VERTEX_FILL_COLOR);
    }
}

fn screen_to_scene(pos: Pos2, origin: Pos2) -> Point {
    Point {
        x: (pos.x - origin.x) as f64,
        y: (pos.y - origin.y) as f64,
    }
}

// AI-finished: punkt przycięty do prostokąta płótna [0, WIDTH-1] x [0, HEIGHT-1]
fn clamp_to_canvas(p: Point) -> Point {
    let max_x = WIDTH as f64 - 1.0;
    let max_y = HEIGHT as f64 - 1.0;
    let mut q = p;
    if q.x < 0.0 {
        q.x = 0.0;
    }
    if q.x > max_x {
        q.x = max_x;
    }
    if q.y < 0.0 {
        q.y = 0.0;
    }
    if q.y > max_y {
        q.y = max_y;
    }
    q
}

// AI-finished: przesunięcie (dx, dy) zmniejszone tak, żeby żaden wierzchołek nie wyszedł poza płótno
fn clamp_polygon_delta(polygon: &Polygon, dx: f64, dy: f64) -> (f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for v in &polygon.vertices {
        min_x = min_x.min(v.pos.x);
        max_x = max_x.max(v.pos.x);
        min_y = min_y.min(v.pos.y);
        max_y = max_y.max(v.pos.y);
    }
    let mut dx = dx;
    let mut dy = dy;
    if min_x + dx < 0.0 {
        dx = -min_x;
    }
    if max_x + dx > WIDTH as f64 - 1.0 {
        dx = WIDTH as f64 - 1.0 - max_x;
    }
    if min_y + dy < 0.0 {
        dy = -min_y;
    }
    if max_y + dy > HEIGHT as f64 - 1.0 {
        dy = HEIGHT as f64 - 1.0 - max_y;
    }
    (dx, dy)
}

fn scene_to_screen(p: Point, origin: Pos2) -> Pos2 {
    origin + egui::vec2(p.x as f32, p.y as f32)
}

// AI-Gen
fn draw_selected_edge(scene: &Scene, e: EdgeRef, painter: &egui::Painter, origin: Pos2) {
    // AI-finished: podświetlamy całą łamaną, więc działa też dla Béziera i łuku
    let pts = scene.polygons[e.polygon].edge_polyline(e.edge, 50);
    for i in 1..pts.len() {
        let pa = scene_to_screen(pts[i - 1], origin);
        let pb = scene_to_screen(pts[i], origin);
        painter.line_segment([pa, pb], (3.0, egui::Color32::ORANGE));
    }
}

// AI-finished: ikony ograniczeń obok środka krawędzi
//  - długość:        tekst "L=200" po jednej stronie krawędzi
//  - prostopadłość:  znak kąta prostego + numer ograniczenia po drugiej stronie
//                    (ten sam numer na obu krawędziach pary)
fn draw_constraint_icons(scene: &Scene, painter: &egui::Painter, origin: Pos2) {
    let font = egui::FontId::proportional(13.0);
    let blue = egui::Color32::from_rgb(30, 90, 220);

    // punkt obok środka krawędzi, przesunięty prostopadle o `side` pikseli
    let beside = |e: EdgeRef, side: f64| -> Pos2 {
        let (a, b) = scene.polygons[e.polygon].edge_points(e.edge);
        let m = (a + b) * 0.5;
        let n = match model::unit((b - a).perp()) {
            Some(n) => n,
            None => Point { x: 0.0, y: -1.0 },
        };
        scene_to_screen(m + n * side, origin)
    };

    for (pi, polygon) in scene.polygons.iter().enumerate() {
        for (ei, edge) in polygon.edges.iter().enumerate() {
            if let EdgeKind::Line { len: Some(l) } = edge.kind {
                let pos = beside(EdgeRef { polygon: pi, edge: ei }, 14.0);
                painter.text(pos, egui::Align2::CENTER_CENTER, format!("L={:.0}", l), font.clone(), blue);
            }
        }
    }

    for (ci, c) in scene.constraints.iter().enumerate() {
        match c {
            Constraint::Perpendicular { a, b } => {
                for e in [*a, *b] {
                    let p = beside(e, -14.0);
                    // znak kąta prostego: "L" z dwóch odcinków
                    let s = 5.0;
                    let stroke = (1.5, blue);
                    painter.line_segment([p + egui::vec2(-s, -s), p + egui::vec2(-s, s)], stroke);
                    painter.line_segment([p + egui::vec2(-s, s), p + egui::vec2(s, s)], stroke);
                    painter.text(p + egui::vec2(s + 6.0, 0.0), egui::Align2::CENTER_CENTER, format!("{}", ci + 1), font.clone(), blue);
                }
            }
        }
    }
}

fn draw_bezier_controls(polygon: &Polygon, painter: &egui::Painter, origin: Pos2) {
    for edge in &polygon.edges {
        if let EdgeKind::Bezier { c1, c2 } = &edge.kind {
            let a = polygon.vertices[edge.p1].pos;
            let b = polygon.vertices[edge.p2].pos;
            let pts = [
                scene_to_screen(a, origin),
                scene_to_screen(*c1, origin),
                scene_to_screen(*c2, origin),
                scene_to_screen(b, origin),
            ];
            // łamana a -> c1 -> c2 -> b, kreski 6 px, przerwy 4 px
            painter.extend(egui::Shape::dashed_line(&pts, (1.0, egui::Color32::GRAY), 6.0, 4.0));
            painter.circle_filled(pts[1], VERTEX_RADIUS, egui::Color32::MAGENTA);
            painter.circle_filled(pts[2], VERTEX_RADIUS, egui::Color32::MAGENTA);
        }
    }
}





