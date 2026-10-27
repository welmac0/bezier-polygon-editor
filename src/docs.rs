// AI-finished: cały plik – dokumentacja dostępna z poziomu aplikacji (przycisk "Dokumentacja").

use eframe::egui;
use egui::{pos2, vec2, Color32, Pos2, Rect};

// Zawartość okna dokumentacji.
pub fn show_docs(ui: &mut egui::Ui) {
    // większy tekst i odstępy tylko w oknie dokumentacji
    let style = ui.style_mut();
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.5));
    style.spacing.item_spacing.y = 6.0;

    egui::ScrollArea::both().show(ui, |ui| {
        ui.set_max_width(760.0); // krótsze wiersze czyta się łatwiej
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Edytor wielokątów i wielokątów krzywoliniowych").size(22.0).strong());
        ui.label(egui::RichText::new("Rust + egui · odcinki, Béziery 3. stopnia, łuki, ograniczenia, ciągłość G0/G1/C1").weak());
        ui.add_space(10.0);

        egui::CollapsingHeader::new(section_title("1. Instrukcja obsługi")).default_open(true).show(ui, |ui| {
            sub_heading(ui, "Mysz na płótnie");
            key_table(ui, "mouse_keys", &[
                ("LPM + przeciągnij wierzchołek", "przesuwa wierzchołek (niebieska kropka)"),
                ("LPM + przeciągnij punkt kontrolny", "zmienia kształt Béziera (różowa kropka)"),
                ("Shift + LPM + przeciągnij wierzchołek", "przesuwa cały wielokąt"),
                ("LPM klik na krawędzi", "zaznacza krawędź (pomarańczowa) – opcje w panelu"),
                ("LPM klik w pustym miejscu", "odznacza krawędź"),
                ("PPM na wierzchołku", "usuwa wierzchołek (zostają min. 3)"),
                ("PPM na krawędzi", "dodaje wierzchołek w środku krawędzi"),
            ]);
            note(ui, "Przesuwane elementy nie wyjeżdżają poza płótno. Jedyny używany klawisz to Shift.");

            sub_heading(ui, "Panel po prawej (dla zaznaczonej krawędzi)");
            bullet(ui, "Typ krawędzi: Odcinek / Bézier / Łuk; przy łuku – odwrócenie półokręgu.");
            bullet(ui, "Długość: pole z aktualną długością; „Ustaw” dodaje ograniczenie, „Usuń” je zdejmuje.");
            bullet(ui, "„Prostopadła do…” i klik w drugą krawędź (może być z innego wielokąta).");
            bullet(ui, "Lista prostopadłości krawędzi – każda z przyciskiem „Usuń”.");
            bullet(ui, "Ciągłość w obu końcach krawędzi: G0 / G1 / C1 (C1 tylko przy Bézierze).");

            sub_heading(ui, "Pasek u góry i ikony");
            bullet(ui, "Bresenham / Library – algorytm rysowania odcinków (własny albo z egui).");
            bullet(ui, "„L=…” przy krawędzi – zadana długość.");
            bullet(ui, "Znak kąta prostego z numerem – para prostopadłych krawędzi (ten sam numer na obu).");
        });

        egui::CollapsingHeader::new(section_title("2. Założenia")).show(ui, |ui| {
            sub_heading(ui, "Wielokąty");
            bullet(ui, "Wielokąty są zamknięte: krawędź k łączy wierzchołek k z (k + 1) mod n.");
            bullet(ui, "Wstawienie lub usunięcie wierzchołka usuwa ograniczenia zmienionych krawędzi, a numery pozostałych krawędzi w ograniczeniach są przesuwane.");

            sub_heading(ui, "Ograniczenia");
            bullet(ui, "Długość i prostopadłość dotyczą tylko odcinków.");
            bullet(ui, "Zmiana krawędzi na Bézier lub łuk usuwa jej prostopadłości (długość znika razem z typem odcinka).");
            bullet(ui, "Jedna krawędź może mieć jednocześnie długość i kilka prostopadłości.");
            bullet(ui, "Jeśli po przesunięciu wierzchołka ograniczeń nie da się spełnić, przesuwa się cały wielokąt.");

            sub_heading(ui, "Ciągłość");
            bullet(ui, "Ciągłość G0 / G1 / C1 jest cechą wierzchołka; brak ustawienia oznacza G0.");
            bullet(ui, "Po zmianie krawędzi na Béziera końce bez ustawionej ciągłości dostają C1 (grupa L–Z). Wybór użytkownika nie jest nadpisywany.");
            bullet(ui, "Przesuwanie punktu kontrolnego nie rusza wierzchołków jego krawędzi, chyba że wymaga tego ciągłość z sąsiednim odcinkiem lub ograniczenia.");

            sub_heading(ui, "Łuki");
            bullet(ui, "G1 może mieć tylko jeden koniec łuku – ustawienie G1 na drugim zmienia pierwszy na G0.");
            bullet(ui, "C1 przy łuku działa jak G1.");
            bullet(ui, "Gdy sąsiad łuku też jest łukiem, jako styczną bierzemy kierunek jego cięciwy.");
        });

        egui::CollapsingHeader::new(section_title("3. Model danych")).show(ui, |ui| {
            note(ui, "Strzałka ciągła – „zawiera” (pole). Strzałka przerywana – „wskazuje indeksem”.");
            draw_model_diagram(ui);

            sub_heading(ui, "Typy");
            key_table(ui, "model_types", &[
                ("Scene", "właściciel wszystkiego: wielokąty i ograniczenia między krawędziami"),
                ("Polygon", "wektory wierzchołków i krawędzi; kolejność = obieg wielokąta"),
                ("Vertex", "pozycja + ciągłość (Option<PointKind>)"),
                ("Edge", "typ krawędzi + indeksy końców p1, p2 w polygon.vertices"),
                ("EdgeKind::Line { len }", "odcinek; zadana długość siedzi w samej krawędzi"),
                ("EdgeKind::Bezier { c1, c2 }", "punkty kontrolne w krawędzi, końce to wierzchołki"),
                ("EdgeKind::Arc { flip }", "łuk; geometria liczona z końców i ciągłości"),
                ("Constraint::Perpendicular", "w scenie, bo może łączyć krawędzie różnych wielokątów"),
                ("EdgeRef / PointRef", "adresy (indeksy) krawędzi i punktów zamiast referencji"),
                ("MyApp", "stan interfejsu: scena, bufor pikseli, przeciąganie, zaznaczenie"),
            ]);
            note(ui, "Indeksy zamiast referencji: scenę można swobodnie zmieniać (borrow checker), a wektory – wstawiać i usuwać elementy.");

            sub_heading(ui, "Pliki");
            key_table(ui, "model_files", &[
                ("model.rs", "model danych i operacje na scenie"),
                ("constraints.rs", "solver: ograniczenia i ciągłość"),
                ("algorithms.rs", "Bézier, łuki, geometria"),
                ("raster.rs", "bufor pikseli i Bresenham"),
                ("main.rs", "interfejs, mysz i rysowanie"),
                ("docs.rs", "ta dokumentacja"),
            ]);
        });

        egui::CollapsingHeader::new(section_title("4. Algorytmy")).show(ui, |ui| {
            sub_heading(ui, "Bresenham (raster.rs: bresenham, line)");
            bullet(ui, "bresenham liczy piksele dla 1. oktantu (0 ≤ dy ≤ dx) zmienną decyzyjną na liczbach całkowitych.");
            bullet(ui, "line sprowadza dowolny odcinek do 1. oktantu (zamiana x↔y, zamiana końców, odbicie y), a otrzymane piksele przekształca z powrotem.");
            bullet(ui, "Wersja biblioteczna: Painter::line_segment z egui (z antyaliasingiem).");

            sub_heading(ui, "Bézier 3. stopnia – algorytm przyrostowy (algorithms.rs: bezier_points)");
            bullet(ui, "Przejście do bazy potęgowej:");
            formula(ui, "P(t) = A3·t³ + A2·t² + A1·t + A0\nA0 = V0\nA1 = 3(V1 − V0)\nA2 = 3(V0 − 2V1 + V2)\nA3 = −V0 + 3V1 − 3V2 + V3");
            bullet(ui, "Różnice skończone dla kroku d = 1/n, a w pętli same dodawania:");
            formula(ui, "ΔP  = A3·d³ + A2·d² + A1·d\nΔ²P = 6·A3·d³ + 2·A2·d²\nΔ³P = 6·A3·d³\n\npowtórz n razy:  P += ΔP;  ΔP += Δ²P;  Δ²P += Δ³P");
            bullet(ui, "Krzywa to łamana z n + 1 punktów, rysowana wybranym algorytmem odcinków.");

            sub_heading(ui, "Łuki (algorithms.rs)");
            bullet(ui, "G0 na obu końcach: półokrąg – środek w środku krawędzi, promień = połowa długości.");
            bullet(ui, "G1 na początku A ze styczną t: okrąg przez A i B styczny do t w A:");
            formula(ui, "n = t obrócone o 90°\nC = A + s·n,   s = |AB|² / (2·(B − A)·n)\nkierunek obiegu: znak iloczynu wektorowego (A − C) × t");
            bullet(ui, "G1 na końcu B: to samo liczone od B z kierunkiem −t.");

            sub_heading(ui, "Ograniczenia i ciągłość (constraints.rs: solve)");
            note(ui, "Relaksacja iteracyjna – warunki poprawiane po kolei, aż wszystkie są spełnione.");
            numbered(ui, 1, "Przesuwany punkt jest zablokowany – zawsze zostaje pod kursorem.");
            numbered(ui, 2, "Do 200 razy przechodzimy po wszystkich warunkach i poprawiamy niespełnione:");
            ui.indent("solver_steps", |ui| {
                bullet(ui, "długość – końce przesuwane wzdłuż krawędzi (jeden zablokowany → rusza drugi, żaden → oba symetrycznie);");
                bullet(ui, "prostopadłość – obrót krawędzi z mniejszą liczbą zablokowanych końców (wokół zablokowanego końca albo środka), długość zachowana;");
                bullet(ui, "ciągłość w wierzchołku V – najpierw przesuwany jest punkt kontrolny, potem drugi koniec sąsiedniego odcinka:");
                formula(ui, "Bézier:  D_in = 3(V − c2)    D_out = 3(c1 − V)\nodcinek: D_in = V − W        D_out = W − V\n\nC1: D_out = D_in        G1: ten sam kierunek");
            });
            numbered(ui, 3, "Przejście bez żadnej poprawki oznacza sukces.");
            numbered(ui, 4, "Brak zbieżności oznacza porażkę: ruch jest cofany, a dla wierzchołka przesuwa się cały wielokąt (przesunięcie zachowuje długości, kierunki i ciągłość).");
            note(ui, "Dodanie ograniczenia albo zmiana ciągłości też uruchamia solver – jeśli się nie da, zmiana jest odrzucana.");
        });

        egui::CollapsingHeader::new(section_title("5. Kod z ostatniego etapu (AI-finished)")).show(ui, |ui| {
            ui.label("Funkcje dodane lub przerobione w ostatnim etapie projektu. W kodzie oznaczone komentarzem:");
            ui.label(egui::RichText::new("// AI-finished").monospace().color(Color32::from_rgb(30, 110, 230)));
            ui.add_space(8.0);

            code_section(ui, "constraints.rs", "cały plik – ograniczenia, ciągłość, solver", &[
                ("solve", "nowa", "relaksacja iteracyjna: poprawia warunki aż wszystkie są spełnione"),
                ("fix_length", "nowa", "przesuwa końce krawędzi do zadanej długości"),
                ("fix_perpendicular", "nowa", "wybiera, którą krawędź pary obrócić"),
                ("rotate_perpendicular", "nowa", "obraca krawędź do pozycji prostopadłej, zachowując długość"),
                ("fix_continuity", "nowa", "G1 / C1 w wierzchołku przy Bézierze"),
                ("set_length / remove_length", "nowa", "ograniczenie długości z panelu"),
                ("add_perpendicular", "nowa", "prostopadłość (odrzuca duplikaty i sprzeczne)"),
                ("remove_constraint", "nowa", "usuwa ograniczenie o danym numerze"),
                ("constraints_of / other_edge", "nowa", "lista ograniczeń krawędzi do panelu"),
                ("remove_constraints_with_edge", "nowa", "usuwa ograniczenia danej krawędzi"),
                ("shift_constraint_edges", "nowa", "przesuwa numery krawędzi po insert / remove"),
                ("set_vertex_kind", "nowa", "ciągłość wierzchołka + reguła jednego G1 na łuku"),
                ("get_point / set_point", "nowa", "odczyt i zapis punktu po adresie PointRef"),
                ("edge_end_refs / is_line", "nowa", "pomocnicze"),
            ]);

            code_section(ui, "model.rs", "model danych", &[
                ("Point: + − * dot length perp", "nowa", "działania na wektorach"),
                ("move_point", "zmieniona", "przesunięcie z solverem; porażka -> przesunięcie wielokąta"),
                ("find_edge", "zmieniona", "trafianie także w Béziera i łuk"),
                ("insert_vertex / remove_vertex", "zmieniona", "poprawianie ograniczeń (dawne TODO)"),
                ("set_bezier / set_line", "zmieniona", "usuwanie prostopadłości, uruchomienie solvera"),
                ("set_arc / flip_arc", "nowa", "łuk i odwrócenie półokręgu"),
                ("edge_polyline", "nowa", "krawędź jako łamana (odcinek, Bézier, łuk)"),
                ("tangent_into / tangent_out_of", "nowa", "styczne sąsiadów dla łuku z G1"),
                ("neighbor_kinds", "nowa", "czy przy wierzchołku jest Bézier / łuk"),
                ("is_smooth / unit", "nowa", "pomocnicze"),
                ("demo", "zmieniona", "nowa scena; poprzednia jako demo_legacy"),
            ]);

            code_section(ui, "algorithms.rs", "geometria łuków", &[
                ("arc_semicircle", "nowa", "półokrąg przy G0–G0"),
                ("arc_tangent_start / _end", "nowa", "łuk styczny w początku / końcu (G1)"),
                ("circle_points", "nowa", "punkty na okręgu"),
            ]);

            code_section(ui, "main.rs", "interfejs i rysowanie", &[
                ("edge_panel / continuity_row", "nowa", "panel: typ, ograniczenia, ciągłość"),
                ("select_edge", "nowa", "zaznaczenie + domyślna długość w polu"),
                ("draw_constraint_icons", "nowa", "ikony L=… i kąta prostego"),
                ("draw_selected_edge", "zmieniona", "podświetlenie wzdłuż krzywej"),
                ("draw_polygon_raster / _lib", "zmieniona", "rysowanie łuków"),
                ("handle_mouse", "zmieniona", "tryb 'prostopadła do…', ograniczenie do płótna"),
                ("clamp_to_canvas / clamp_polygon_delta", "nowa", "elementy nie wyjeżdżają poza płótno"),
                ("ui", "zmieniona", "stały panel, rozmiar okna, okno dokumentacji"),
            ]);

            code_section(ui, "docs.rs", "cały plik – ta dokumentacja", &[
                ("show_docs", "nowa", "treść dokumentacji"),
                ("draw_model_diagram", "nowa", "diagram modelu danych"),
                ("draw_box / draw_arrow / border_point", "nowa", "rysowanie diagramu"),
                ("code_section", "nowa", "ta tabela"),
            ]);
        });

        egui::CollapsingHeader::new(section_title("6. Autorstwo kodu")).show(ui, |ui| {
            ui.label("Kategorie:");
            legend_row(ui, "ja", "napisane samodzielnie (czasem na podstawie podpowiedzi: sygnatura, wzór, opis kroków)");
            legend_row(ui, "ja + AI", "mój kod poprawiony lub uzupełniony przez AI (dawniej: partly AI-gen)");
            legend_row(ui, "AI-gen", "napisane przez AI na moją prośbę w trakcie pracy (// AI-gen)");
            legend_row(ui, "AI-finished", "napisane przez AI w ostatnim etapie projektu (// AI-finished, sekcja 5)");
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Udział w liniach kodu").strong());
            ui.label("Liczone linie kodu programu bez pustych linii i komentarzy, przypisane do funkcji wg tabel");
            ui.label("poniżej. Dokumentacja (docs.rs) nie jest liczona. Kod poza funkcjami – struktury, enumy,");
            ui.label("stałe – liczony jako mój, poza constraints.rs (w całości AI).");
            egui::Grid::new("authorship_summary").striped(true).num_columns(3).spacing([14.0, 4.0]).show(ui, |ui| {
                ui.label(egui::RichText::new("kategoria").strong());
                ui.label(egui::RichText::new("linie").strong());
                ui.label(egui::RichText::new("udział").strong());
                ui.end_row();
                for (status, lines, percent) in [
                    ("ja", "324", "20%"),
                    ("ja + AI", "309", "19%"),
                    ("AI-gen", "174", "11%"),
                    ("AI-finished", "780", "49%"),
                ] {
                    ui.label(egui::RichText::new(status).color(status_color(status)).monospace());
                    ui.label(lines);
                    ui.label(percent);
                    ui.end_row();
                }
            });
            ui.add_space(4.0);
            ui.label("Samodzielnie: ok. 20% kodu; razem z kodem 'ja + AI' ok. 40%.");
            ui.label("Liczby są przybliżone – funkcje 'ja + AI' liczone w całości.");
            ui.add_space(10.0);

            code_section(ui, "raster.rs", "bufor pikseli i Bresenham", &[
                ("new / set_pixel", "ja", "bufor RGBA, zapalanie piksela z obcinaniem"),
                ("bresenham", "ja", "algorytm dla 1. oktantu"),
                ("line_lib", "ja", "odcinek biblioteczny (egui)"),
                ("point_to_pos2", "ja", "pomocnicza (nieużywana)"),
                ("raster_clear", "ja + AI", "poprawione indeksy pętli"),
                ("line", "AI-gen", "dowolny oktant przez symetrie"),
            ]);

            code_section(ui, "algorithms.rs", "algorytmy geometryczne", &[
                ("dist", "ja", "odległość punktów"),
                ("bezier_points", "ja + AI", "baza potęgowa moja; różnice skończone i pętla poprawione"),
                ("dist_point_segment", "AI-gen", "odległość punktu od odcinka"),
                ("arc_* / circle_points", "AI-finished", "łuki"),
            ]);

            code_section(ui, "model.rs", "model danych", &[
                ("struktury i enumy", "ja", "Point, Vertex, Edge, EdgeKind, Polygon, Scene, EdgeRef, PointRef, ..."),
                ("edge_points / set_line", "ja", "końce krawędzi, zamiana na odcinek"),
                ("demo_legacy", "ja", "poprzednia scena (trójkąt + kwadrat)"),
                ("find_point", "ja + AI", "AI dodało szukanie punktów kontrolnych"),
                ("find_edge", "ja + AI", "AI: trafianie w krzywe"),
                ("insert_vertex / remove_vertex", "ja + AI", "AI: poprawki ograniczeń"),
                ("set_bezier", "ja + AI", "AI: punkty kontrolne, C1, solver"),
                ("find_vertex / translate", "AI-gen", "szukanie wierzchołka, przesuwanie wielokąta"),
                ("renumber_edges", "AI-gen", "odświeżanie p1/p2 krawędzi"),
                ("pozostałe (sekcja 5)", "AI-finished", "move_point, łuki, styczne, operatory, demo, ..."),
            ]);

            code_section(ui, "main.rs", "interfejs i rysowanie", &[
                ("main / MyApp / Default", "ja", "start aplikacji i stan"),
                ("draw_vertices", "ja", "kropki wierzchołków"),
                ("ui", "ja + AI", "AI: kolejność płótno -> mysz -> rysowanie, panel, okno dokumentacji"),
                ("handle_mouse", "ja + AI", "PPM i zaznaczanie moje; przeciąganie i reszta AI"),
                ("draw_scene_raster", "ja + AI", "AI: poprawiona pętla"),
                ("draw_polygon_raster", "ja + AI", "AI: odcinek; Bézier mój; łuk AI"),
                ("draw_scene_lib / draw_polygon_lib", "AI-gen", "wersja biblioteczna"),
                ("screen_to_scene / scene_to_screen", "AI-gen", "przeliczanie współrzędnych"),
                ("draw_selected_edge / draw_bezier_controls", "AI-gen", "podświetlenie, łamana kontrolna"),
                ("pozostałe (sekcja 5)", "AI-finished", "panel, ikony, ograniczenie do płótna"),
            ]);

            code_section(ui, "constraints.rs", "w całości", &[
                ("wszystkie funkcje", "AI-finished", "solver, ograniczenia, ciągłość"),
            ]);
        });
    });
}

// ---------- pomocnicze elementy formatowania ----------

// tytuł sekcji (większy, pogrubiony)
fn section_title(text: &str) -> egui::RichText {
    egui::RichText::new(text).size(18.0).strong()
}

// podtytuł wewnątrz sekcji
fn sub_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(text).size(16.0).strong().color(Color32::from_rgb(30, 110, 230)));
}

// punkt listy (zawija się jak zwykły tekst)
fn bullet(ui: &mut egui::Ui, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label("•");
        ui.label(text);
    });
}

// punkt numerowany
fn numbered(ui: &mut egui::Ui, n: usize, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(format!("{}.", n)).strong());
        ui.label(text);
    });
}

// uwaga / komentarz (kursywa, przygaszony)
fn note(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).italics().weak());
}

// wzór w ramce, czcionką o stałej szerokości
fn formula(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).monospace());
        });
}

// tabela dwóch kolumn: pogrubiona nazwa | opis
fn key_table(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)]) {
    egui::Grid::new(id).striped(true).num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
        for (key, value) in rows {
            ui.label(egui::RichText::new(*key).strong());
            ui.label(*value);
            ui.end_row();
        }
    });
}

// jeden wiersz legendy: kolorowa etykieta + opis
fn legend_row(ui: &mut egui::Ui, status: &str, text: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(status).color(status_color(status)).monospace());
        ui.label(text);
    });
}

// kolor etykiety statusu w tabelach
fn status_color(status: &str) -> Color32 {
    match status {
        "nowa" => Color32::from_rgb(40, 150, 70),
        "zmieniona" => Color32::from_rgb(210, 130, 20),
        "ja" => Color32::from_rgb(40, 150, 70),
        "ja + AI" => Color32::from_rgb(190, 150, 0),
        "AI-gen" => Color32::from_rgb(210, 110, 20),
        _ => Color32::from_rgb(30, 110, 230), // AI-finished
    }
}

// Tabela funkcji jednego pliku: nagłówek (nazwa pliku + opis) i wiersze (funkcja | status | co robi).
fn code_section(ui: &mut egui::Ui, file: &str, about: &str, rows: &[(&str, &str, &str)]) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(file).strong().monospace().size(14.0));
        ui.label(egui::RichText::new(about).weak());
    });
    egui::Grid::new(file)
        .striped(true)
        .num_columns(3)
        .spacing([14.0, 4.0])
        .show(ui, |ui| {
            for (name, status, what) in rows {
                ui.label(egui::RichText::new(*name).monospace());
                let color = status_color(status);
                ui.label(egui::RichText::new(*status).color(color).small());
                ui.label(*what);
                ui.end_row();
            }
        });
    ui.add_space(10.0);
}

// ---------- diagram zależności między strukturami i enumami ----------

const BOX_W: f32 = 210.0;
const LINE_H: f32 = 15.0;

// Prostokąt typu: nagłówek + pola. Zwraca prostokąt (do zaczepiania strzałek).
fn draw_box(ui: &egui::Ui, origin: Pos2, x: f32, y: f32, lines: &[&str]) -> Rect {
    let rect = Rect::from_min_size(origin + vec2(x, y), vec2(BOX_W, 8.0 + LINE_H * lines.len() as f32));
    let painter = ui.painter();
    let visuals = ui.visuals();
    painter.rect_filled(rect, 4.0, visuals.extreme_bg_color);
    painter.rect_stroke(rect, 4.0, (1.0, visuals.text_color()), egui::StrokeKind::Inside);
    for (i, line) in lines.iter().enumerate() {
        let color = if i == 0 { Color32::from_rgb(30, 110, 230) } else { visuals.text_color() };
        painter.text(
            rect.min + vec2(6.0, 4.0 + LINE_H * i as f32),
            egui::Align2::LEFT_TOP,
            *line,
            egui::FontId::monospace(11.0),
            color,
        );
    }
    // linia pod nagłówkiem
    let y_sep = rect.min.y + 4.0 + LINE_H - 1.0;
    painter.line_segment([pos2(rect.min.x, y_sep), pos2(rect.max.x, y_sep)], (0.5, visuals.text_color()));
    rect
}

// punkt na brzegu prostokąta w kierunku `toward` (żeby strzałka zaczynała się na krawędzi pudełka)
fn border_point(r: Rect, toward: Pos2) -> Pos2 {
    let c = r.center();
    let d = toward - c;
    let tx = if d.x.abs() > 1e-3 { (r.width() / 2.0) / d.x.abs() } else { f32::MAX };
    let ty = if d.y.abs() > 1e-3 { (r.height() / 2.0) / d.y.abs() } else { f32::MAX };
    c + d * tx.min(ty)
}

// Strzałka od pudełka `from` do pudełka `to`; dashed = relacja przez indeks.
fn draw_arrow(ui: &egui::Ui, from: Rect, to: Rect, dashed: bool, label: &str) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    let a = border_point(from, to.center());
    let b = border_point(to, from.center());
    if dashed {
        painter.extend(egui::Shape::dashed_line(&[a, b], (1.0, color), 5.0, 4.0));
    } else {
        painter.line_segment([a, b], (1.0, color));
    }
    // grot
    let dir = (b - a).normalized();
    let side = vec2(-dir.y, dir.x);
    painter.line_segment([b, b - dir * 9.0 + side * 4.0], (1.0, color));
    painter.line_segment([b, b - dir * 9.0 - side * 4.0], (1.0, color));
    if !label.is_empty() {
        let m = a + (b - a) * 0.5 + side * 8.0;
        painter.text(m, egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(10.0), color);
    }
}

fn draw_model_diagram(ui: &mut egui::Ui) {
    let (response, _painter) = ui.allocate_painter(vec2(700.0, 520.0), egui::Sense::hover());
    let o = response.rect.min;

    // kolumna 0: strona aplikacji (main.rs)
    let drawing_alg = draw_box(ui, o, 0.0, 0.0, &["enum DrawingAlg", "Library | Bresenham"]);
    let my_app = draw_box(
        ui,
        o,
        0.0,
        80.0,
        &[
            "struct MyApp (main.rs)",
            "scene: Scene",
            "raster: Raster",
            "drawing_alg: DrawingAlg",
            "dragged: Option<Dragged>",
            "selected_edge: Option<EdgeRef>",
        ],
    );
    let dragged = draw_box(ui, o, 0.0, 230.0, &["enum Dragged (main.rs)", "Polygon { polygon }", "Point(PointRef)"]);
    let point_ref = draw_box(
        ui,
        o,
        0.0,
        330.0,
        &["enum PointRef", "Vertex { polygon, vertex }", "Control { polygon, edge,", "          which: 1|2 }"],
    );
    let point_kind = draw_box(ui, o, 0.0, 450.0, &["enum PointKind", "G0 | G1 | C1"]);

    // kolumna 1: scena -> wielokąt -> wierzchołek -> punkt
    let scene = draw_box(ui, o, 245.0, 80.0, &["struct Scene", "polygons: Vec<Polygon>", "constraints: Vec<Constraint>"]);
    let polygon = draw_box(ui, o, 245.0, 200.0, &["struct Polygon", "vertices: Vec<Vertex>", "edges: Vec<Edge>"]);
    let vertex = draw_box(ui, o, 245.0, 310.0, &["struct Vertex", "pos: Point", "kind: Option<PointKind>"]);
    let point = draw_box(ui, o, 245.0, 450.0, &["struct Point", "x, y: f64"]);

    // kolumna 2: ograniczenia i krawędzie
    let constraint = draw_box(ui, o, 490.0, 80.0, &["enum Constraint", "Perpendicular { a, b }"]);
    let edge_ref = draw_box(ui, o, 490.0, 160.0, &["struct EdgeRef", "polygon: usize", "edge: usize"]);
    let edge = draw_box(ui, o, 490.0, 290.0, &["struct Edge", "kind: EdgeKind", "p1, p2: usize"]);
    let edge_kind = draw_box(
        ui,
        o,
        490.0,
        400.0,
        &["enum EdgeKind", "Line { len: Option<f64> }", "Bezier { c1, c2: Point }", "Arc { flip: bool }"],
    );

    // zawieranie (pola)
    draw_arrow(ui, my_app, drawing_alg, false, "");
    draw_arrow(ui, my_app, scene, false, "");
    draw_arrow(ui, my_app, dragged, false, "");
    draw_arrow(ui, dragged, point_ref, false, "");
    draw_arrow(ui, scene, polygon, false, "Vec");
    draw_arrow(ui, scene, constraint, false, "Vec");
    draw_arrow(ui, constraint, edge_ref, false, "a, b");
    draw_arrow(ui, polygon, vertex, false, "Vec");
    draw_arrow(ui, polygon, edge, false, "Vec");
    draw_arrow(ui, vertex, point, false, "pos");
    draw_arrow(ui, vertex, point_kind, false, "kind");
    draw_arrow(ui, edge, edge_kind, false, "kind");
    draw_arrow(ui, edge_kind, point, false, "c1, c2");

    // wskazywanie indeksem
    draw_arrow(ui, edge, vertex, true, "p1, p2");
    draw_arrow(ui, edge_ref, polygon, true, "");
    draw_arrow(ui, edge_ref, edge, true, "");
    draw_arrow(ui, point_ref, vertex, true, "");
}
