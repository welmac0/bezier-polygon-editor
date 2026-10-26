use crate::algorithms::dist_point_segment;
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
        poly.edges.insert(edge_id + 1, Edge { kind: Line {len: None}, p1: 0, p2: 0 });
        // p1, p2 sa byle jakie
        poly.edges[edge_id].kind = Line {len: None}; // ze strony:
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
    }
    pub fn set_line(&mut self, e: EdgeRef) {
        self.polygons[e.polygon].edges[e.edge].kind = Line { len: None };
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
