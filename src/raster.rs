pub struct Raster {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA, 4 bajty na piksel A = alpha
}

impl Raster {
    pub fn new(width: usize, height: usize) -> Raster {
        Raster { width, height, pixels: vec![255; width * height * 4] } // całe białe
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        match (x, y) {
            (x, y) if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 => return,
            _ => (),
        }

        let i = ((y as usize) * self.width + (x as usize)) * 4;
        self.pixels[i..i + 4].copy_from_slice(&color);
    }

    pub fn bresenham(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let mut x = x0;
        let mut y = y0;

        let mut d = 2*dy - dx;
        // D_0 = y_l - y_m = dy/dx - 1/2 // *2dx aby pominac float
        // D_0 = dy - dx

        let incr_e = 2 * dy;
        // D_1 = D_0 + dy/dx // *2dx
        // D_1 = D_0 + 2dy

        let incr_ne = 2*dy - 2*dx;
        // D_1 = D_0 + dy/dx - 1 // *2dx
        // D_1 = D_0 + 2dy - 2dx
        self.set_pixel(x, y, color);

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
            self.set_pixel(x, y, color);
        }
    }
}