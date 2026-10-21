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
    pub fn demo() -> Scene {
        let p1 = Point{x: 1.0, y: 0.0};
        let p2 = Point{x: 1.0, y: 1.0};
        let p3 = Point{x: 0.0, y: 0.0};

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

        let triangle = Polygon {
            vertices: vec![v1, v2, v3],
            edges: vec![e1, e2, e3],
        };

        Scene {
            polygons: vec![triangle],
            constraints: vec![],
        }
    }
}
