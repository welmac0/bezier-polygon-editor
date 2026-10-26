use eframe::egui;
use eframe::egui::{pos2, Pos2};

pub struct Raster {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA, 4 bajty na piksel A = alpha
}

impl Raster {
    pub fn new(width: usize, height: usize) -> Raster {
        Raster { width, height, pixels: vec![255; width * height * 4] } // całe białe
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        match (x, y) {
            (x, y) if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 => return,
            _ => (),
        }

        let i = ((y as usize) * self.width + (x as usize)) * 4;
        self.pixels[i..i + 4].copy_from_slice(&color);
    }

    fn bresenham(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let mut x = x0;
        let mut y = y0;
        let mut pixels = Vec::new();

        let mut d = 2*dy - dx;
        // D_0 = y_l - y_m = dy/dx - 1/2 // *2dx aby pominac float
        // D_0 = dy - dx

        let incr_e = 2 * dy;
        // D_1 = D_0 + dy/dx // *2dx
        // D_1 = D_0 + 2dy

        let incr_ne = 2*dy - 2*dx;
        // D_1 = D_0 + dy/dx - 1 // *2dx
        // D_1 = D_0 + 2dy - 2dx
        // self.set_pixel(x, y, color);
        pixels.push((x, y));

        while x < x1 {
            if d <= 0 {
                // E
                x += 1;
                d += incr_e;
            } else {
                x += 1;
                y += 1;
                d += incr_ne;
            }
            // self.set_pixel(x, y, color);
            pixels.push((x, y));
        }

        pixels
    }

    // Odcinek w dowolnym kierunku: sprowadzamy go do 1. oktantu,
    // liczymy piksele bresenhamem i odbijamy je z powrotem.
    // AI-gen
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let mut x0 = x0;
        let mut y0 = y0;
        let mut x1 = x1;
        let mut y1 = y1;

        // 1. stromy odcinek (|dy| > |dx|) -> zamiana x z y
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            (x0, y0) = (y0, x0);
            (x1, y1) = (y1, x1);
        }

        // 2. zawsze od lewej do prawej (zamiana końców nie zmienia pikseli)
        if x0 > x1 {
            (x0, x1) = (x1, x0);
            (y0, y1) = (y1, y0);
        }

        // 3. odcinek w dół -> odbicie względem osi x (y -> -y)
        let mirror = y1 < y0;
        if mirror {
            y0 = -y0;
            y1 = -y1;
        }

        let pixels = Self::bresenham(x0, y0, x1, y1);

        // odwrotne przekształcenia w odwrotnej kolejności: najpierw 3, potem 1
        for (x, y) in pixels {
            let mut px = x;
            let mut py = y;
            if mirror {
                py = -py;
            }
            if steep {
                (px, py) = (py, px);
            }
            self.set_pixel(px, py, color);
        }
    }

    pub fn line_lib(painter: &egui::Painter, origin: egui::Pos2, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let points: [Pos2; 2] = [
            pos2(origin.x + x0 as f32, origin.y + y0 as f32),
            pos2(origin.x + x1 as f32, origin.y + y1 as f32),
        ];
        let color32 = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
        painter.line_segment(points, egui::Stroke::new(1.0, color32));
    }

    // AI-gen (poprawione indeksy pętli)
    pub fn raster_clear(&mut self, color: [u8; 4]) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_pixel(x as i32, y as i32, color);
            }
        }
    }

    pub fn point_to_pos2(x: f64, y: f64) -> Pos2 {
        pos2(x as f32, y as f32)
    }
}