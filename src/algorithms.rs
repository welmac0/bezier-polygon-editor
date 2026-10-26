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