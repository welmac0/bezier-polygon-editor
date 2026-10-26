use crate::model::EdgeKind::Line;
use crate::model::PointKind::G0;

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
        flip: bool,

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

// AI-gen
// Odległość punktu P od odcinka AB (do trafiania myszą w krawędź).
//
// Narzędzia:
// - wektor między punktami: AB = B - A
// - iloczyn skalarny: u·v = u.x*v.x + u.y*v.y
//   (AP·AB / |AB| = długość "cienia" AP na kierunku AB)
// - rzut prostopadły P na prostą AB: Q = A + t*AB, gdzie t = (AP·AB) / |AB|²
//   (t = 0 -> A, t = 1 -> B, t = 0.5 -> środek)
// - obcięcie t do [0, 1], żeby Q nie wyszło poza odcinek
//   (inaczej mierzylibyśmy odległość od nieskończonej prostej)
// - odległość euklidesowa (Pitagoras): |PQ| = sqrt(dx² + dy²)
//
// Przykłady do sprawdzenia:
// A=(0,0) B=(10,0) P=(3,4)  -> t=0.3,          wynik 4
// A=(0,0) B=(4,4)  P=(0,4)  -> t=0.5, Q=(2,2), wynik ~2.83
// A=(0,0) B=(10,0) P=(15,2) -> t=1.5 -> 1, Q=B, wynik ~5.39
pub fn dist_point_segment(p: Point, a: Point, b: Point) -> f64 {
    // 1. wektor krawędzi AB i wektor do kliknięcia AP
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let apx = p.x - a.x;
    let apy = p.y - a.y;

    // 2. |AB|²; gdy A = B, odcinek to punkt -> odległość P-A (bez dzielenia przez 0)
    let len2 = abx * abx + aby * aby;
    if len2 == 0.0 {
        return (apx * apx + apy * apy).sqrt();
    }

    // 3. parametr rzutu i obcięcie do odcinka
    let mut t = (apx * abx + apy * aby) / len2;
    if t < 0.0 {
        t = 0.0;
    }
    if t > 1.0 {
        t = 1.0;
    }

    // 4. najbliższy punkt odcinka Q i odległość P-Q
    let qx = a.x + t * abx;
    let qy = a.y + t * aby;
    let dx = p.x - qx;
    let dy = p.y - qy;
    (dx * dx + dy * dy).sqrt()
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
    // AI-gen (podział na get_triangle i get_square)
    pub fn demo() -> Scene {
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
                kind: EdgeKind::Line { len: None },
                p1: 0,
                p2: 1,
            };

            let e2 = Edge {
                kind: EdgeKind::Line { len: None },
                p1: 1,
                p2: 2,
            };

            let e3 = Edge {
                kind: EdgeKind::Line { len: None },
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
                kind: EdgeKind::Line { len: None },
                p1: 0,
                p2: 1,
            };

            let e2 = Edge {
                kind: EdgeKind::Line { len: None },
                p1: 1,
                p2: 2,
            };

            let e3 = Edge {
                kind: EdgeKind::Line { len: None },
                p1: 2,
                p2: 3,
            };

            let e4 = Edge {
                kind: EdgeKind::Line { len: None },
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

    pub fn find_edge(&self, p: Point, radius: f64) -> Option<EdgeRef> {
        for i in 0..self.polygons.len() {
            for j in 0..self.polygons[i].edges.len() {
                let (a, b) = self.polygons[i].edge_points(j);
                let dist = dist_point_segment(p, a, b);
                if dist <= radius {
                    return Some(EdgeRef { polygon: i, edge: j }); // return konczy natychmiast
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
        poly.edges.insert(edge_id + 1, Edge { kind: EdgeKind::Line {len: None}, p1: 0, p2: 0 });
        // p1, p2 sa byle jakie
        poly.edges[edge_id].kind = EdgeKind::Line {len: None}; // ze strony:
        //  | dodanie wierzchołka na krawędzi lub usunięcie wierzchołka powinno powodować usunięcie
        //  | ograniczeń związanych z tą krawędzią oraz odpowiednią aktualizację relacji
        //  | krawędzi sąsiednich
        poly.renumber_edges();

        // TODO: poprawić self.constraints (po ostatnim użyciu `poly`):
        //  - usunąć ograniczenia z krawędzią (polygon_id, edge_id)
        //  - w pozostałych z tego wielokąta: edge > edge_id -> edge + 1
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

        // TODO: poprawić self.constraints (po ostatnim użyciu `poly`, stare numery krawędzi):
        //  - usunąć ograniczenia z krawędzią (polygon, prev) lub (polygon, vertex_id)
        //  - w pozostałych z tego wielokąta: edge > vertex_id -> edge - 1

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
}
