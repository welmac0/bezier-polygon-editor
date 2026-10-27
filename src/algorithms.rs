use crate::model::Point;

pub fn bezier_points(a: &Point, b: &Point, c1: &Point, c2: &Point, n: usize) -> Vec<Point> {
    let mut pixel_to_draw: Vec<Point> = Vec::new();

    let v0 = a;
    let v1 = c1;
    let v2 = c2;
    let v3 = b;

    // baza wielomianowa
    let a0 = *v0; // * bo v0 to &Point, a potrzebujemy kopii punktu
    let a2 = Point { x: 3.0 * (v2.x - 2.0 * v1.x + v0.x), y: 3.0 * (v2.y - 2.0 * v1.y + v0.y) };
    let a1 = Point { x: 3.0 * (v1.x - v0.x), y: 3.0 * (v1.y - v0.y) }; //3*(V1 - V0);
    let a3 = Point { x: v3.x - 3.0 * v2.x + 3.0 * v1.x - v0.x, y: v3.y - 3.0 * v2.y + 3.0 * v1.y - v0.y };

    let d = 1.0 / n as f64; // krok t
    let d2 = d * d;
    let d3 = d2 * d;
    let mut p = a0;
    // różnice skończone w t = 0 (nie pochodne): każda ma potęgi kroku d
    let mut dp = Point { x: a3.x * d3 + a2.x * d2 + a1.x * d, y: a3.y * d3 + a2.y * d2 + a1.y * d };
    let mut ddp = Point { x: 6.0 * a3.x * d3 + 2.0 * a2.x * d2, y: 6.0 * a3.y * d3 + 2.0 * a2.y * d2 };
    let dddp = Point { x: 6.0 * a3.x * d3, y: 6.0 * a3.y * d3 }; // stała

    pixel_to_draw.push(p);
    for _ in 0..(n as usize) {
        p.x += dp.x;
        p.y += dp.y;

        dp.x += ddp.x;
        dp.y += ddp.y;

        ddp.x += dddp.x;
        ddp.y += dddp.y;
        pixel_to_draw.push(p);
    }
    // ostatni punkt dokładnie v3 (bez błędów zaokrągleń)
    let last = pixel_to_draw.len() - 1;
    pixel_to_draw[last] = *v3;

    pixel_to_draw
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

pub fn dist(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}


// AI-finished
// Punkty na okręgu o środku c i promieniu r: od kąta `start`, obrót o kąt `sweep`
// (sweep > 0 -> kąt rośnie, sweep < 0 -> maleje), n kroków -> n + 1 punktów.
fn circle_points(c: Point, r: f64, start: f64, sweep: f64, n: usize) -> Vec<Point> {
    let mut pts = Vec::new();
    for i in 0..=n {
        let t = start + sweep * (i as f64) / (n as f64);
        pts.push(Point { x: c.x + r * t.cos(), y: c.y + r * t.sin() });
    }
    pts
}

// AI-finished
// Łuk przy G0 na obu końcach: półokrąg nad cięciwą AB.
// Środek = środek AB, promień = |AB| / 2 (wymóg specyfikacji), flip wybiera stronę.
pub fn arc_semicircle(a: Point, b: Point, flip: bool, n: usize) -> Vec<Point> {
    let c = (a + b) * 0.5;
    let r = dist(a, b) / 2.0;
    let start = (a.y - c.y).atan2(a.x - c.x);
    let sweep = if flip { -std::f64::consts::PI } else { std::f64::consts::PI };
    let mut pts = circle_points(c, r, start, sweep, n);
    let last = pts.len() - 1;
    pts[last] = b; // dokładnie B, bez błędów zaokrągleń
    pts
}

// AI-finished
// Łuk przy G1 na początku: okrąg przez A i B, styczny w A do kierunku t.
//
// Geometria:
// - środek leży na prostej przez A prostopadłej do t: C = A + s * n, gdzie n = t obrócone o 90°
// - |C - A| = |C - B|  =>  s = |AB|² / (2 * (B - A)·n)
// - kierunek obiegu: jeśli (A - C) x t > 0, idziemy w stronę rosnącego kąta, inaczej malejącego
// - gdy B leży na prostej stycznej (mianownik ~ 0), okrąg byłby nieskończony -> odcinek AB
pub fn arc_tangent_start(a: Point, b: Point, t: Point, n: usize) -> Vec<Point> {
    let tl = t.length();
    if tl < 1e-9 {
        return vec![a, b];
    }
    let t = t * (1.0 / tl);
    let nrm = t.perp();
    let ab = b - a;
    let denom = 2.0 * ab.dot(nrm);
    if denom.abs() < 1e-9 {
        return vec![a, b];
    }
    let s = ab.dot(ab) / denom;
    let c = a + nrm * s;
    let r = s.abs();

    let ra = a - c;
    let rb = b - c;
    let start = ra.y.atan2(ra.x);
    let end = rb.y.atan2(rb.x);
    let ccw = ra.x * t.y - ra.y * t.x > 0.0; // iloczyn wektorowy (A - C) x t

    let two_pi = 2.0 * std::f64::consts::PI;
    let mut sweep = end - start;
    if ccw {
        while sweep <= 0.0 {
            sweep += two_pi;
        }
    } else {
        while sweep >= 0.0 {
            sweep -= two_pi;
        }
    }

    let mut pts = circle_points(c, r, start, sweep, n);
    let last = pts.len() - 1;
    pts[0] = a;
    pts[last] = b;
    pts
}

// AI-finished
// Łuk przy G1 na końcu: dochodzimy do B w kierunku t.
// To samo co łuk od B do A, który wychodzi z B w kierunku -t, tylko punkty w odwrotnej kolejności.
pub fn arc_tangent_end(a: Point, b: Point, t: Point, n: usize) -> Vec<Point> {
    let mut pts = arc_tangent_start(b, a, t * -1.0, n);
    pts.reverse();
    pts
}
