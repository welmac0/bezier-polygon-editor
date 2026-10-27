use crate::algorithms::{arc_semicircle, arc_tangent_end, arc_tangent_start, bezier_points, dist, dist_point_segment};
use crate::model::EdgeKind::Line;

// rodzaj ksztaltu
#[derive(Clone, Debug)]
pub enum EdgeKind {
    Line {
        len: Option<f64>,
    },
    Bezier {
        c1: Point,
        c2: Point
    },
    Arc {
        flip: bool, // po której stronie cięciwy leży półokrąg (tylko przy G0–G0)
    },
}

// base struct Edge, can be changed to Bezier or Arc
#[derive(Clone, Debug)]
pub struct Edge {
    pub kind: EdgeKind,
    pub p1: usize,
    pub p2: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// AI-finished: działania na punktach/wektorach, żeby wzory (solver, łuki) czytały się jak na kartce:
// a + b, a - b, a * 2.0. Rust Book: rozdz. "Advanced Traits" -> przeciążanie operatorów.
impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, o: Point) -> Point {
        Point { x: self.x + o.x, y: self.y + o.y }
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, o: Point) -> Point {
        Point { x: self.x - o.x, y: self.y - o.y }
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, k: f64) -> Point {
        Point { x: self.x * k, y: self.y * k }
    }
}

// AI-finished
impl Point {
    // iloczyn skalarny
    pub fn dot(self, o: Point) -> f64 {
        self.x * o.x + self.y * o.y
    }

    // długość wektora
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    // wektor obrócony o 90 stopni (prostopadły)
    pub fn perp(self) -> Point {
        Point { x: -self.y, y: self.x }
    }
}



#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointKind { // punkt laczenia krawedzi i jego typ
    G0,
    G1,
    C1,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex { // wierzcholek co mozna przesuwac
    pub pos: Point,
    pub kind: Option<PointKind>,
}

#[derive(Clone, Debug)]
pub struct Polygon { // wielokat
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

// Krawędź pozna się po parze (wielokąt, krawędź), bo relacja może łączyć
// krawędzie z RÓŻNYCH wielokątów. Same indeksy, więc bez referencji.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeRef {
    pub polygon: usize,
    pub edge: usize,
}

// referencja do przeciaganai punktow kontrolnych Beziera, ktora jest zdefiniowana w
// Edge::EdgeKind::Bezier{c1, c2}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointRef {
    Vertex { polygon: usize, vertex: usize },
    Control {polygon: usize, edge: usize, which: u8}, // 1=c1, 2=c2
}

// Ograniczenia międzykrawędziowe żyją w scenie, bo przekraczają granice
// wielokąta. Długość zostaje w Line (Twoja decyzja), więc tu na razie jest
// jedna relacja. Enum zostawiam, żeby dało się dopisać kolejne warianty.
#[derive(Clone, Debug)]
pub enum Constraint { // ograniczenia
    Perpendicular { a: EdgeRef, b: EdgeRef },
}

// Scena jest właścicielem wszystkiego i to ona sprawdza ograniczenia
// Patrzy w dwa miejsca:
// w `constraints` po prostopadłości, w Line.len po długości.
#[derive(Clone, Debug)]
pub struct Scene {
    pub polygons: Vec<Polygon>,
    pub constraints: Vec<Constraint>,
}

impl Scene {
    pub fn find_point(&self, p: Point, radius: f64) -> Option<PointRef> {
        for (polygon_index, polygon) in self.polygons.iter().enumerate() {
            for (edge_index, edge) in polygon.edges.iter().enumerate() {
                if let EdgeKind::Bezier { c1, c2 } = &edge.kind {
                    if dist(p, *c1) < radius {
                        return Some(PointRef::Control { polygon: polygon_index, edge: edge_index, which: 1 });
                    }
                    if dist(p, *c2) < radius {
                        return Some(PointRef::Control { polygon: polygon_index, edge: edge_index, which: 2 });
                    }
                }
            }
        }
        for (polygon_index, polygon) in self.polygons.iter().enumerate() {
            for (vertex_index, vertex) in polygon.vertices.iter().enumerate() {
                if dist(p, vertex.pos) < radius {
                    return Some(PointRef::Vertex { polygon: polygon_index, vertex: vertex_index });
                }
            }
        }
        None
    }
    
    // AI-finished: przesunięcie punktu myszą + pilnowanie ograniczeń i ciągłości (solver w constraints.rs)
    pub fn move_point(&mut self, r: PointRef, p: Point) {
        let backup = self.clone(); // stan sprzed ruchu, na wypadek porażki
        let delta = p - self.get_point(r);
        self.set_point(r, p);

        // przesuwany wierzchołek zabiera ze sobą sąsiednie punkty kontrolne Béziera
        // (dzięki temu kształt krzywej przy wierzchołku się nie psuje)
        if let PointRef::Vertex { polygon, vertex } = r {
            let n = self.polygons[polygon].vertices.len();
            let prev = (vertex + n - 1) % n; // krawędź wchodząca do wierzchołka
            if let EdgeKind::Bezier { c2, .. } = &mut self.polygons[polygon].edges[prev].kind {
                *c2 = *c2 + delta;
            }
            if let EdgeKind::Bezier { c1, .. } = &mut self.polygons[polygon].edges[vertex].kind {
                *c1 = *c1 + delta;
            }
        }

        // przesuwany punkt jest "zablokowany": solver dopasowuje resztę do niego
        if self.solve(&[r]) {
            return;
        }

        // nie da się spełnić ograniczeń -> cofamy ruch.
        // Dla wierzchołka przesuwamy wtedy cały wielokąt: przesunięcie nie zmienia
        // długości, kierunków ani ciągłości, więc wszystkie ograniczenia zostają spełnione,
        // a wierzchołek dalej idzie za kursorem.
        *self = backup;
        if let PointRef::Vertex { polygon, .. } = r {
            self.polygons[polygon].translate(delta.x, delta.y);
        }
    }
    
    
    // AI-finished: scena początkowa
    //  - wielokąt 0: 10 wierzchołków, dwa Béziery, dwa łuki, ciągłości G1/C1, dwie długości,
    //                prostopadłość wewnątrz wielokąta
    //  - wielokąt 1: trójkąt z prostopadłością do krawędzi wielokąta 0 (relacja między wielokątami)
    // Nowy wielokąt = nowa funkcja get_...() z listą punktów + wpis w `polygons`.
    pub fn demo() -> Scene {
        // funkcje wewnętrzne: widoczne tylko w środku demo()
        fn p(x: f64, y: f64) -> Point {
            Point { x, y }
        }

        // wielokąt z listy punktów: krawędź k łączy punkt k z (k + 1) % n, wszystkie to odcinki
        fn polygon(points: &[Point]) -> Polygon {
            let n = points.len();
            let mut vertices = Vec::new();
            let mut edges = Vec::new();
            for k in 0..n {
                vertices.push(Vertex { pos: points[k], kind: None });
                edges.push(Edge { kind: Line { len: None }, p1: k, p2: (k + 1) % n });
            }
            Polygon { vertices, edges }
        }

        fn get_main_polygon() -> Polygon {
            polygon(&[
                p(225.0, 93.0),  // w0
                p(322.0, 102.0), // w1
                p(388.0, 238.0), // w2  C1
                p(771.0, 219.0), // w3  G1
                p(771.0, 411.0), // w4
                p(731.0, 372.0), // w5  G1 (początek łuku)
                p(635.0, 421.0), // w6
                p(491.0, 353.0), // w7
                p(358.0, 403.0), // w8  G1
                p(94.0, 183.0),  // w9  C1
            ])
        }

        fn get_triangle() -> Polygon {
            polygon(&[p(520.0, 40.0), p(560.0, 146.0), p(640.0, 60.0)])
        }

        fn edge(polygon: usize, edge: usize) -> EdgeRef {
            EdgeRef { polygon, edge }
        }

        let mut scene = Scene {
            polygons: vec![get_main_polygon(), get_triangle()],
            constraints: vec![],
        };

        // Béziery (punkty kontrolne jak na rysunku; ciągłość i tak je dopasuje)
        scene.set_bezier(edge(0, 2));
        scene.set_point(PointRef::Control { polygon: 0, edge: 2, which: 1 }, p(410.0, 281.0));
        scene.set_point(PointRef::Control { polygon: 0, edge: 2, which: 2 }, p(771.0, 156.0));
        scene.set_bezier(edge(0, 8));
        scene.set_point(PointRef::Control { polygon: 0, edge: 8, which: 1 }, p(313.0, 420.0));
        scene.set_point(PointRef::Control { polygon: 0, edge: 8, which: 2 }, p(51.0, 210.0));

        // łuki: w5 -> w6 (G1 w w5, styczny do krawędzi w4 -> w5), w6 -> w7 (G0–G0, półokrąg)
        scene.set_arc(edge(0, 5));
        scene.set_arc(edge(0, 6));

        // ciągłość w wierzchołkach
        scene.set_vertex_kind(0, 2, PointKind::C1);
        scene.set_vertex_kind(0, 3, PointKind::G1);
        scene.set_vertex_kind(0, 5, PointKind::G1);
        scene.set_vertex_kind(0, 8, PointKind::G1);
        scene.set_vertex_kind(0, 9, PointKind::C1);

        // ograniczenia
        scene.set_length(edge(0, 0), 97.0);
        scene.set_length(edge(0, 1), 151.0);
        scene.add_perpendicular(edge(0, 3), edge(0, 0)); // pionowa w3–w4 ⊥ górna w0–w1
        scene.add_perpendicular(edge(0, 7), edge(1, 0)); // między wielokątami

        scene
    }

    // Legacy: poprzednia scena (trójkąt + kwadrat), nieużywana – zostawiona dla historii.
    #[allow(dead_code)] // AI-finished: bez tego kompilator ostrzega, że funkcja nie jest wywoływana
    pub fn demo_legacy() -> Scene {
        // funkcje wewnętrzne: widoczne tylko w środku demo()
        fn get_triangle() -> Polygon {
            let p1 = Point{x: 200.0, y: 100.0};
            let p2 = Point{x: 400.0, y: 300.0};
            let p3 = Point{x: 100.0, y: 300.0};

            let v1 = Vertex {
                pos: p1,
                kind: None
            };

            let v2 = Vertex {
                pos: p2,
                kind: None
            };

            let v3 = Vertex {
                pos: p3,
                kind: None
            };

            let e1 = Edge {
                kind: Line { len: None },
                p1: 0,
                p2: 1,
            };

            let e2 = Edge {
                kind: Line { len: None },
                p1: 1,
                p2: 2,
            };

            let e3 = Edge {
                kind: Line { len: None },
                p1: 2,
                p2: 0,
            };

            Polygon {
                vertices: vec![v1, v2, v3],
                edges: vec![e1, e2, e3],
            }
        }

        fn get_square() -> Polygon {
            let p1 = Point{x: 550.0, y: 150.0};
            let p2 = Point{x: 750.0, y: 150.0};
            let p3 = Point{x: 750.0, y: 350.0};
            let p4 = Point{x: 550.0, y: 350.0};

            let v1 = Vertex {
                pos: p1,
                kind: None
            };

            let v2 = Vertex {
                pos: p2,
                kind: None
            };

            let v3 = Vertex {
                pos: p3,
                kind: None
            };

            let v4 = Vertex {
                pos: p4,
                kind: None
            };

            let e1 = Edge {
                kind: Line { len: None },
                p1: 0,
                p2: 1,
            };

            let e2 = Edge {
                kind: Line { len: None },
                p1: 1,
                p2: 2,
            };

            let e3 = Edge {
                kind: Line { len: None },
                p1: 2,
                p2: 3,
            };

            let e4 = Edge {
                kind: Line { len: None },
                p1: 3,
                p2: 0,
            };

            Polygon {
                vertices: vec![v1, v2, v3, v4],
                edges: vec![e1, e2, e3, e4],
            }
        }

        Scene {
            polygons: vec![get_triangle(), get_square()],
            constraints: vec![],
        }
    }

    // AI-gen: pierwszy wierzchołek bliżej niż `radius` od p -> (indeks wielokąta, indeks wierzchołka)
    pub fn find_vertex(&self, p: Point, radius: f64) -> Option<(usize, usize)> {
        for i in 0..self.polygons.len() {
            for j in 0..self.polygons[i].vertices.len() {
                let v = self.polygons[i].vertices[j].pos;
                let dx = v.x - p.x;
                let dy = v.y - p.y;
                if dx * dx + dy * dy <= radius * radius {
                    return Some((i, j));
                }
            }
        }
        None
    }

    // partly AI-finished: sprawdzamy łamaną krawędzi, więc trafia się też w Béziera i łuk
    pub fn find_edge(&self, p: Point, radius: f64) -> Option<EdgeRef> {
        for i in 0..self.polygons.len() {
            for j in 0..self.polygons[i].edges.len() {
                let pts = self.polygons[i].edge_polyline(j, 50);
                for k in 1..pts.len() {
                    let dist = dist_point_segment(p, pts[k - 1], pts[k]);
                    if dist <= radius {
                        return Some(EdgeRef { polygon: i, edge: j }); // return konczy natychmiast
                    }
                }
            }
        }
        None
    }

    pub fn insert_vertex(&mut self, e: EdgeRef) {
        // niezmiennik: e_k = v_k~v_k+1 np. e_0 = v_0~v_1
        let polygon_id = e.polygon;
        let edge_id = e.edge;

        let poly = &mut self.polygons[polygon_id];

        let (a, b) = poly.edge_points(edge_id);
        let new_p = Point { x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0 };
        let new_v = Vertex { pos: new_p, kind: None };
        poly.vertices.insert(edge_id + 1, new_v);
        poly.edges.insert(edge_id + 1, Edge { kind: Line {len: None}, p1: 0, p2: 0 });
        // p1, p2 sa byle jakie
        poly.edges[edge_id].kind = Line {len: None}; // ze strony:
        //  | dodanie wierzchołka na krawędzi lub usunięcie wierzchołka powinno powodować usunięcie
        //  | ograniczeń związanych z tą krawędzią oraz odpowiednią aktualizację relacji
        //  | krawędzi sąsiednich
        poly.renumber_edges();

        // AI-finished: ograniczenia (po ostatnim użyciu `poly`)
        //  - usuwamy ograniczenia podzielonej krawędzi (polygon_id, edge_id)
        //  - krawędzie za nią przesunęły się o 1 w prawo -> edge + 1
        self.remove_constraints_with_edge(EdgeRef { polygon: polygon_id, edge: edge_id });
        self.shift_constraint_edges(polygon_id, edge_id, 1);
        self.solve(&[]);
    }

    pub fn remove_vertex(&mut self, polygon: usize, vertex_id: usize) {
        if self.polygons[polygon].vertices.len() <= 3 {
            return;
        }

        let poly = &mut self.polygons[polygon];
        let n = poly.vertices.len();
        // krawedz e z niezmiennika: e=v_j~v_j+1
        let prev = (vertex_id + n - 1) % n;
        poly.edges[prev].kind = Line {len: None};
        poly.vertices.remove(vertex_id);
        poly.edges.remove(vertex_id);
        poly.renumber_edges();

        // AI-finished: ograniczenia (po ostatnim użyciu `poly`, stare numery krawędzi)
        //  - usuwamy ograniczenia obu krawędzi przy usuniętym wierzchołku
        //  - krawędzie za usuniętą przesunęły się o 1 w lewo -> edge - 1
        self.remove_constraints_with_edge(EdgeRef { polygon, edge: prev });
        self.remove_constraints_with_edge(EdgeRef { polygon, edge: vertex_id });
        self.shift_constraint_edges(polygon, vertex_id, -1);
        self.solve(&[]);
    }

    pub fn set_bezier(&mut self, e_ref: EdgeRef) {
        let (a, b) = self.polygons[e_ref.polygon].edge_points(e_ref.edge);
        let dx = b.x - a.x;
        let dy = b.y - a.y;

        let len = (dx * dx + dy * dy).sqrt();
        let nx = -dy / len * 40.0;
        let ny = dx / len * 40.0;

        // punkt = początek a + część krawędzi + odsunięcie w bok o 40 px
        let c1 = Point { x: a.x + dx / 3.0 + nx, y: a.y + dy / 3.0 + ny };
        let c2 = Point { x: a.x + 2.0 * dx / 3.0 + nx, y: a.y + 2.0 * dy / 3.0 + ny };

        self.polygons[e_ref.polygon].edges[e_ref.edge].kind = EdgeKind::Bezier { c1, c2 };

        // końce krawędzi bierzemy z samej krawędzi (dla ostatniej krawędzi p2 = 0)
        let i1 = self.polygons[e_ref.polygon].edges[e_ref.edge].p1;
        let i2 = self.polygons[e_ref.polygon].edges[e_ref.edge].p2;

        // L–Z: domyślnie C1, ale nie nadpisujemy wyboru użytkownika
        if self.polygons[e_ref.polygon].vertices[i1].kind == None {
            self.polygons[e_ref.polygon].vertices[i1].kind = Some(PointKind::C1);
        }
        if self.polygons[e_ref.polygon].vertices[i2].kind == None {
            self.polygons[e_ref.polygon].vertices[i2].kind = Some(PointKind::C1);
        }

        // AI-finished: prostopadłość dotyczy tylko odcinków, a C1 trzeba od razu wymusić
        self.remove_constraints_with_edge(e_ref);
        self.solve(&[]);
    }
    pub fn set_line(&mut self, e: EdgeRef) {
        self.polygons[e.polygon].edges[e.edge].kind = Line { len: None };
        self.solve(&[]); // AI-finished: sąsiednie Béziery dopasowują się do nowego odcinka
    }

    // AI-finished: zamiana krawędzi na łuk okręgu
    pub fn set_arc(&mut self, e: EdgeRef) {
        self.remove_constraints_with_edge(e); // prostopadłość tylko dla odcinków
        self.polygons[e.polygon].edges[e.edge].kind = EdgeKind::Arc { flip: false };

        // łuk może mieć G1 tylko na jednym końcu -> jeśli oba są G1/C1, drugi koniec dostaje G0
        let i1 = self.polygons[e.polygon].edges[e.edge].p1;
        let i2 = self.polygons[e.polygon].edges[e.edge].p2;
        if is_smooth(self.polygons[e.polygon].vertices[i1].kind)
            && is_smooth(self.polygons[e.polygon].vertices[i2].kind)
        {
            self.polygons[e.polygon].vertices[i2].kind = Some(PointKind::G0);
        }
        self.solve(&[]);
    }

    // AI-finished: półokrąg na drugą stronę cięciwy (ma znaczenie tylko przy G0 na obu końcach)
    pub fn flip_arc(&mut self, e: EdgeRef) {
        if let EdgeKind::Arc { flip } = &mut self.polygons[e.polygon].edges[e.edge].kind {
            *flip = !*flip;
        }
    }

}

impl Polygon {
    // AI-gen: przesuwa cały wielokąt (wierzchołki i punkty kontrolne Béziera)
    pub fn translate(&mut self, dx: f64, dy: f64) {
        for vertex in &mut self.vertices {
            vertex.pos.x += dx;
            vertex.pos.y += dy;
        }
        for edge in &mut self.edges {
            if let EdgeKind::Bezier { c1, c2 } = &mut edge.kind {
                c1.x += dx;
                c1.y += dy;
                c2.x += dx;
                c2.y += dy;
            }
        }
    }

    pub fn renumber_edges(&mut self) {
        let n = self.vertices.len();
        for k in 0..n {
            self.edges[k].p1 = k;
            self.edges[k].p2 = (k + 1) % n;
        }
    }

    pub fn edge_points(&self, edge: usize) -> (Point, Point) {
        let e = &self.edges[edge];
        let a = self.vertices[e.p1];
        let b = self.vertices[e.p2];
        (a.pos, b.pos)
    }

    // AI-finished: krawędź jako łamana (do rysowania i trafiania myszą)
    //  - odcinek: 2 punkty
    //  - Bézier:  steps + 1 punktów z algorytmu przyrostowego
    //  - łuk:     steps + 1 punktów na okręgu
    pub fn edge_polyline(&self, edge: usize, steps: usize) -> Vec<Point> {
        let (a, b) = self.edge_points(edge);
        match &self.edges[edge].kind {
            EdgeKind::Line { .. } => vec![a, b],
            EdgeKind::Bezier { c1, c2 } => bezier_points(&a, &b, c1, c2, steps),
            EdgeKind::Arc { flip } => {
                let i1 = self.edges[edge].p1;
                let i2 = self.edges[edge].p2;
                // G1 na początku: łuk styczny do krawędzi wchodzącej do początku
                if is_smooth(self.vertices[i1].kind) {
                    if let Some(t) = self.tangent_into(i1) {
                        return arc_tangent_start(a, b, t, steps);
                    }
                }
                // G1 na końcu: łuk styczny do krawędzi wychodzącej z końca
                if is_smooth(self.vertices[i2].kind) {
                    if let Some(t) = self.tangent_out_of(i2) {
                        return arc_tangent_end(a, b, t, steps);
                    }
                }
                // G0 na obu końcach: półokrąg, promień = połowa długości krawędzi
                arc_semicircle(a, b, *flip, steps)
            }
        }
    }

    // AI-finished: kierunek, w jakim krawędź WCHODZĄCA dochodzi do wierzchołka v (styczna z lewej)
    // Łuk jako sąsiad: uproszczenie -> kierunek jego cięciwy.
    pub fn tangent_into(&self, v: usize) -> Option<Point> {
        let n = self.vertices.len();
        let e = &self.edges[(v + n - 1) % n];
        let pos = self.vertices[v].pos;
        let from = match &e.kind {
            EdgeKind::Bezier { c1, c2 } => {
                if (pos - *c2).length() > 1e-9 { *c2 } else { *c1 }
            }
            _ => self.vertices[e.p1].pos,
        };
        unit(pos - from)
    }

    // AI-finished: kierunek, w jakim krawędź WYCHODZĄCA opuszcza wierzchołek v (styczna z prawej)
    pub fn tangent_out_of(&self, v: usize) -> Option<Point> {
        let e = &self.edges[v];
        let pos = self.vertices[v].pos;
        let to = match &e.kind {
            EdgeKind::Bezier { c1, c2 } => {
                if (*c1 - pos).length() > 1e-9 { *c1 } else { *c2 }
            }
            _ => self.vertices[e.p2].pos,
        };
        unit(to - pos)
    }

    // AI-finished: czy przy wierzchołku jest Bézier / łuk (do panelu ciągłości)
    pub fn neighbor_kinds(&self, v: usize) -> (bool, bool) {
        let n = self.vertices.len();
        let mut bezier = false;
        let mut arc = false;
        for e in [(v + n - 1) % n, v] {
            match self.edges[e].kind {
                EdgeKind::Bezier { .. } => bezier = true,
                EdgeKind::Arc { .. } => arc = true,
                EdgeKind::Line { .. } => {}
            }
        }
        (bezier, arc)
    }

}

// AI-finished: G1 i C1 to ciągłość "gładka" (wspólna styczna); G0 / brak -> tylko wspólny punkt
pub fn is_smooth(kind: Option<PointKind>) -> bool {
    kind == Some(PointKind::G1) || kind == Some(PointKind::C1)
}

// AI-finished: wektor jednostkowy (None dla wektora zerowego, bo nie ma kierunku)
pub fn unit(v: Point) -> Option<Point> {
    let l = v.length();
    if l < 1e-9 {
        None
    } else {
        Some(v * (1.0 / l))
    }
}
