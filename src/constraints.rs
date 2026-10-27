// AI-finished: cały plik
//
// Ograniczenia (długość, prostopadłość) i ciągłość w wierzchołkach (G0 / G1 / C1).
//
// ALGORYTM (relaksacja iteracyjna):
//  1. Użytkownik przesuwa punkt X (wierzchołek albo punkt kontrolny Béziera).
//     X jest "zablokowany" – solver nigdy go nie rusza, więc X zawsze jest pod kursorem.
//  2. Powtarzamy (maks. MAX_ITER razy) przejście po wszystkich warunkach:
//       a) długość krawędzi            (EdgeKind::Line { len: Some(l) })
//       b) prostopadłość dwóch krawędzi (scene.constraints)
//       c) ciągłość G1 / C1 w wierzchołkach przy Bézierze
//     Każdy niespełniony warunek poprawiamy lokalnie, przesuwając tylko punkty niezablokowane.
//  3. Jeśli w całym przejściu wszystkie warunki były spełnione -> sukces.
//  4. Jeśli po MAX_ITER coś dalej nie gra (warunki są sprzeczne albo zablokowane) -> porażka.
//     Wtedy move_point cofa ruch i przesuwa cały wielokąt (przesunięcie nie psuje żadnego warunku).
//
// Każda poprawka zmienia geometrię tylko trochę i tylko lokalnie; kolejne przejścia
// "rozprowadzają" zmianę po wielokącie (a przez prostopadłość – także po innych wielokątach).

use crate::model::{is_smooth, unit, Constraint, EdgeKind, EdgeRef, Point, PointKind, PointRef, Scene};

const MAX_ITER: usize = 200;
const EPS: f64 = 0.5; // tolerancja odległości w pikselach
const EPS_ANGLE: f64 = 1e-3; // tolerancja kąta (cos / różnica wektorów jednostkowych)

impl Scene {
    // ---------- dostęp do punktów po adresie (PointRef) ----------

    pub fn get_point(&self, r: PointRef) -> Point {
        match r {
            PointRef::Vertex { polygon, vertex } => self.polygons[polygon].vertices[vertex].pos,
            PointRef::Control { polygon, edge, which } => match &self.polygons[polygon].edges[edge].kind {
                EdgeKind::Bezier { c1, c2 } => {
                    if which == 1 { *c1 } else { *c2 }
                }
                _ => Point { x: 0.0, y: 0.0 }, // nie powinno się zdarzyć: tylko Bézier ma punkty kontrolne
            },
        }
    }

    pub fn set_point(&mut self, r: PointRef, p: Point) {
        match r {
            PointRef::Vertex { polygon, vertex } => {
                self.polygons[polygon].vertices[vertex].pos = p;
            }
            PointRef::Control { polygon, edge, which } => {
                if let EdgeKind::Bezier { c1, c2 } = &mut self.polygons[polygon].edges[edge].kind {
                    if which == 1 {
                        *c1 = p;
                    } else {
                        *c2 = p;
                    }
                }
            }
        }
    }

    // adresy obu końców krawędzi
    fn edge_end_refs(&self, e: EdgeRef) -> (PointRef, PointRef) {
        let edge = &self.polygons[e.polygon].edges[e.edge];
        (
            PointRef::Vertex { polygon: e.polygon, vertex: edge.p1 },
            PointRef::Vertex { polygon: e.polygon, vertex: edge.p2 },
        )
    }

    fn is_line(&self, e: EdgeRef) -> bool {
        matches!(self.polygons[e.polygon].edges[e.edge].kind, EdgeKind::Line { .. })
    }

    // ---------- API dla panelu ----------

    // Ograniczenie długości. Zwraca false (i nic nie zmienia), jeśli nie da się go spełnić.
    pub fn set_length(&mut self, e: EdgeRef, l: f64) -> bool {
        if !self.is_line(e) {
            return false;
        }
        let backup = self.clone();
        self.polygons[e.polygon].edges[e.edge].kind = EdgeKind::Line { len: Some(l) };
        if self.solve(&[]) {
            true
        } else {
            *self = backup;
            false
        }
    }

    pub fn remove_length(&mut self, e: EdgeRef) {
        self.polygons[e.polygon].edges[e.edge].kind = EdgeKind::Line { len: None };
    }

    // Prostopadłość krawędzi a i b (mogą być z różnych wielokątów).
    // Krawędź a zostaje na miejscu (jej końce blokujemy), dopasowuje się b.
    pub fn add_perpendicular(&mut self, a: EdgeRef, b: EdgeRef) -> bool {
        if a == b || !self.is_line(a) || !self.is_line(b) {
            return false;
        }
        // ta sama para już istnieje?
        for c in &self.constraints {
            match c {
                Constraint::Perpendicular { a: x, b: y } => {
                    if (*x == a && *y == b) || (*x == b && *y == a) {
                        return false;
                    }
                }
            }
        }
        let backup = self.clone();
        self.constraints.push(Constraint::Perpendicular { a, b });
        let (a1, a2) = self.edge_end_refs(a);
        if self.solve(&[a1, a2]) {
            true
        } else {
            *self = backup;
            false
        }
    }

    pub fn remove_constraint(&mut self, index: usize) {
        if index < self.constraints.len() {
            self.constraints.remove(index);
        }
    }

    // numery ograniczeń (w scene.constraints), które dotyczą krawędzi e
    pub fn constraints_of(&self, e: EdgeRef) -> Vec<usize> {
        let mut result = Vec::new();
        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Perpendicular { a, b } => {
                    if *a == e || *b == e {
                        result.push(i);
                    }
                }
            }
        }
        result
    }

    // druga krawędź ograniczenia o numerze index (ta, która nie jest e)
    pub fn other_edge(&self, index: usize, e: EdgeRef) -> EdgeRef {
        match self.constraints[index] {
            Constraint::Perpendicular { a, b } => {
                if a == e { b } else { a }
            }
        }
    }

    // usuwa wszystkie ograniczenia między krawędziami, w których występuje e
    pub fn remove_constraints_with_edge(&mut self, e: EdgeRef) {
        let mut kept = Vec::new();
        for c in &self.constraints {
            match c {
                Constraint::Perpendicular { a, b } => {
                    if *a != e && *b != e {
                        kept.push(c.clone());
                    }
                }
            }
        }
        self.constraints = kept;
    }

    // po wstawieniu/usunięciu wierzchołka: krawędzie wielokąta `polygon` o numerze > after
    // przesunęły się o delta (+1 albo -1)
    pub fn shift_constraint_edges(&mut self, polygon: usize, after: usize, delta: isize) {
        for c in &mut self.constraints {
            match c {
                Constraint::Perpendicular { a, b } => {
                    for r in [a, b] {
                        if r.polygon == polygon && r.edge > after {
                            r.edge = (r.edge as isize + delta) as usize;
                        }
                    }
                }
            }
        }
    }

    // Ciągłość w wierzchołku wybrana przez użytkownika.
    pub fn set_vertex_kind(&mut self, polygon: usize, vertex: usize, kind: PointKind) {
        self.polygons[polygon].vertices[vertex].kind = Some(kind);

        // łuk może mieć G1 tylko na JEDNYM końcu: drugi koniec sąsiedniego łuku -> G0
        if kind != PointKind::G0 {
            let n = self.polygons[polygon].vertices.len();
            for e in [(vertex + n - 1) % n, vertex] {
                let edge = &self.polygons[polygon].edges[e];
                if let EdgeKind::Arc { .. } = edge.kind {
                    let other = if edge.p1 == vertex { edge.p2 } else { edge.p1 };
                    if is_smooth(self.polygons[polygon].vertices[other].kind) {
                        self.polygons[polygon].vertices[other].kind = Some(PointKind::G0);
                    }
                }
            }
        }
        self.solve(&[]);
    }

    // ---------- solver ----------

    // Zwraca true, jeśli wszystkie warunki są spełnione. `locked` = punkty, których nie wolno ruszać.
    pub fn solve(&mut self, locked: &[PointRef]) -> bool {
        for _ in 0..MAX_ITER {
            let mut all_ok = true;

            // a) długości
            for pi in 0..self.polygons.len() {
                for ei in 0..self.polygons[pi].edges.len() {
                    if let EdgeKind::Line { len: Some(l) } = self.polygons[pi].edges[ei].kind {
                        if !self.fix_length(EdgeRef { polygon: pi, edge: ei }, l, locked) {
                            all_ok = false;
                        }
                    }
                }
            }

            // b) prostopadłości
            for ci in 0..self.constraints.len() {
                match self.constraints[ci] {
                    Constraint::Perpendicular { a, b } => {
                        if !self.fix_perpendicular(a, b, locked) {
                            all_ok = false;
                        }
                    }
                }
            }

            // c) ciągłość
            for pi in 0..self.polygons.len() {
                for vi in 0..self.polygons[pi].vertices.len() {
                    if !self.fix_continuity(pi, vi, locked) {
                        all_ok = false;
                    }
                }
            }

            if all_ok {
                return true;
            }
        }
        false
    }

    // Długość: przesuwamy końce wzdłuż krawędzi.
    // Jeden koniec zablokowany -> rusza się drugi; żaden -> oba symetrycznie względem środka.
    // Zwraca true, jeśli warunek BYŁ spełniony (nic nie trzeba było robić).
    fn fix_length(&mut self, e: EdgeRef, l: f64, locked: &[PointRef]) -> bool {
        let (ra, rb) = self.edge_end_refs(e);
        let a = self.get_point(ra);
        let b = self.get_point(rb);
        let d = (b - a).length();
        if (d - l).abs() < EPS {
            return true;
        }
        let dir = match unit(b - a) {
            Some(u) => u,
            None => Point { x: 1.0, y: 0.0 }, // końce w jednym punkcie: dowolny kierunek
        };
        let la = locked.contains(&ra);
        let lb = locked.contains(&rb);
        if la && lb {
            return false; // oba końce zablokowane – nie da się
        }
        if la {
            self.set_point(rb, a + dir * l);
        } else if lb {
            self.set_point(ra, b - dir * l);
        } else {
            let m = (a + b) * 0.5;
            self.set_point(ra, m - dir * (l / 2.0));
            self.set_point(rb, m + dir * (l / 2.0));
        }
        false
    }

    // Prostopadłość: obracamy jedną krawędź (zachowując jej długość),
    // aż będzie prostopadła do drugiej. Obracamy tę, która ma mniej zablokowanych końców.
    fn fix_perpendicular(&mut self, a: EdgeRef, b: EdgeRef, locked: &[PointRef]) -> bool {
        if !self.is_line(a) || !self.is_line(b) {
            return true; // dotyczy tylko odcinków
        }
        let (a1, a2) = self.edge_end_refs(a);
        let (b1, b2) = self.edge_end_refs(b);
        let u = self.get_point(a2) - self.get_point(a1);
        let v = self.get_point(b2) - self.get_point(b1);
        let (uu, vv) = match (unit(u), unit(v)) {
            (Some(uu), Some(vv)) => (uu, vv),
            _ => return true, // krawędź zerowej długości nie ma kierunku
        };
        if uu.dot(vv).abs() < EPS_ANGLE {
            return true; // cos kąta ~ 0 -> prostopadłe
        }

        let locked_a = locked.contains(&a1) as u8 + locked.contains(&a2) as u8;
        let locked_b = locked.contains(&b1) as u8 + locked.contains(&b2) as u8;
        if locked_b <= locked_a {
            if !self.rotate_perpendicular(b, uu, locked) {
                self.rotate_perpendicular(a, vv, locked);
            }
        } else if !self.rotate_perpendicular(a, vv, locked) {
            self.rotate_perpendicular(b, uu, locked);
        }
        false
    }

    // Ustawia krawędź e prostopadle do kierunku w (jednostkowego), zachowując długość e.
    // Zwraca false, jeśli oba końce e są zablokowane.
    fn rotate_perpendicular(&mut self, e: EdgeRef, w: Point, locked: &[PointRef]) -> bool {
        let (rp, rq) = self.edge_end_refs(e);
        let p = self.get_point(rp);
        let q = self.get_point(rq);
        let len = (q - p).length();

        // dwa kierunki prostopadłe do w; wybieramy bliższy obecnemu, żeby krawędź się nie "przewracała"
        let mut t = w.perp();
        if t.dot(q - p) < 0.0 {
            t = t * -1.0;
        }

        let lp = locked.contains(&rp);
        let lq = locked.contains(&rq);
        if lp && lq {
            return false;
        }
        if lp {
            self.set_point(rq, p + t * len); // obrót wokół p
        } else if lq {
            self.set_point(rp, q - t * len); // obrót wokół q
        } else {
            let m = (p + q) * 0.5; // obrót wokół środka
            self.set_point(rp, m - t * (len / 2.0));
            self.set_point(rq, m + t * (len / 2.0));
        }
        true
    }

    // Ciągłość w wierzchołku V (G1 lub C1), gdy co najmniej jedna sąsiednia krawędź to Bézier.
    //
    // Pochodne w V (krawędź wchodząca kończy się w V, wychodząca zaczyna):
    //   Bézier wchodzący:  D_in  = 3 * (V - c2)       odcinek wchodzący (W -> V): D_in  = V - W
    //   Bézier wychodzący: D_out = 3 * (c1 - V)       odcinek wychodzący (V -> W): D_out = W - V
    //   C1: D_out = D_in          G1: ten sam kierunek, długości dowolne
    // Np. Bézier–odcinek z C1: c2 leży na przedłużeniu odcinka w 1/3 jego długości.
    //
    // Co przesuwamy: najpierw punkt kontrolny (wychodzący, potem wchodzący), a jeśli oba są
    // zablokowane albo to odcinki – drugi koniec odcinka. Samego V nie ruszamy.
    // Łuk jako sąsiad: pomijamy, bo łuk sam liczy się z G1 (styczny do sąsiada).
    fn fix_continuity(&mut self, pi: usize, vi: usize, locked: &[PointRef]) -> bool {
        let poly = &self.polygons[pi];
        let n = poly.vertices.len();
        let kind = poly.vertices[vi].kind;
        if !is_smooth(kind) {
            return true; // G0 / brak – tylko wspólny punkt, zawsze spełnione
        }
        let c1_mode = kind == Some(PointKind::C1);
        let prev_e = (vi + n - 1) % n;
        let next_e = vi;

        // punkt wyznaczający styczną "przed" V, czy to punkt kontrolny, mnożnik pochodnej
        let (q_in, in_ctrl, f_in) = match poly.edges[prev_e].kind {
            EdgeKind::Bezier { .. } => (PointRef::Control { polygon: pi, edge: prev_e, which: 2 }, true, 3.0),
            EdgeKind::Line { .. } => (PointRef::Vertex { polygon: pi, vertex: (vi + n - 1) % n }, false, 1.0),
            EdgeKind::Arc { .. } => return true,
        };
        // punkt wyznaczający styczną "za" V
        let (q_out, out_ctrl, f_out) = match poly.edges[next_e].kind {
            EdgeKind::Bezier { .. } => (PointRef::Control { polygon: pi, edge: next_e, which: 1 }, true, 3.0),
            EdgeKind::Line { .. } => (PointRef::Vertex { polygon: pi, vertex: (vi + 1) % n }, false, 1.0),
            EdgeKind::Arc { .. } => return true,
        };
        if !in_ctrl && !out_ctrl {
            return true; // odcinek–odcinek: ciągłość nie dotyczy
        }

        let v = poly.vertices[vi].pos;
        let p_in = self.get_point(q_in);
        let p_out = self.get_point(q_out);
        let d_in = (v - p_in) * f_in;
        let d_out = (p_out - v) * f_out;

        // czy już spełnione?
        if c1_mode {
            if (d_out - d_in).length() < EPS {
                return true;
            }
        } else {
            match (unit(d_in), unit(d_out)) {
                (Some(ui), Some(uo)) => {
                    if (ui - uo).length() < EPS_ANGLE {
                        return true;
                    }
                }
                _ => return true, // zerowy wektor – brak kierunku
            }
        }

        // poprawka: przejście 0 – punkty kontrolne, przejście 1 – wierzchołki odcinków
        for pass in 0..2 {
            let want_ctrl = pass == 0;
            // strona wychodząca: ustawiamy q_out tak, by D_out pasowało do D_in
            if out_ctrl == want_ctrl && !locked.contains(&q_out) {
                let target = if c1_mode {
                    v + d_in * (1.0 / f_out)
                } else {
                    match unit(d_in) {
                        Some(u) => v + u * (p_out - v).length(),
                        None => return true,
                    }
                };
                self.set_point(q_out, target);
                return false;
            }
            // strona wchodząca: ustawiamy q_in tak, by D_in pasowało do D_out
            if in_ctrl == want_ctrl && !locked.contains(&q_in) {
                let target = if c1_mode {
                    v - d_out * (1.0 / f_in)
                } else {
                    match unit(d_out) {
                        Some(u) => v - u * (v - p_in).length(),
                        None => return true,
                    }
                };
                self.set_point(q_in, target);
                return false;
            }
        }
        false // wszystko, co można by ruszyć, jest zablokowane
    }
}
