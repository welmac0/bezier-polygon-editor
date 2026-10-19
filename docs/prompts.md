# prompts

## prompt 1
ok, przeczytalem jeszcze raz wymagania i zrobilem sobie notatki:
Glowny obiekt to jest Shape - enum podzielony na Arc, bezier i segment.
Segment sie sklada z P1, P2, moze miec zdefiniowana dlugosc
Bezier sklada sie z P1, P2, C1, C2, nie wiem czy z dokumentacji projektu wynika ze ma miec ustalona dlugosc, chyba nie.
arc ma sie tworzyc chyba na podstawie dwoch punktow.
Czyli domyslnie rysuje odcinek, a potem moge zmienic odcinek na Beziera albo na Arc. Stad segment jest jakby klasa bazowa, a Bezier i Arc dziedzicza po Segment. Powiedz mi jak to powinno dzialac w rust.
Nastepnie ma istniec relacja prostopadlosci wobec dwoch roznych krawedzi, ale to jest chyba problem wysokopoziomowy na ten moment.
P, C to Point, ktory ma x i y. jest Segment ma referencje do danych punktow, bo punkty to osobny zbior. Gdzies trzeba zdefiniowac relacje Punkt-Segment. Nastepnie potrzebujemy jeszcze relacji Segment-Polygon, i nie wiem jak powinien wygladac Polygon bo to chyba wyjdzie w algorytmie rysowania, ale chociaz trzeba to zdefiniowac teraz na chwile.
Nie mam pomyslu
Powiedz czy o czyms nie zapomnialem. Odwolaj sie do kazdej rzeczy ktora napisalem i utworzmy wspolnie model.rs



