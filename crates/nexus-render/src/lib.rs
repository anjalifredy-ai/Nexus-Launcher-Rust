//! Fast software renderer — row-fill sky/ground, heightmap terrain, camera.

use std::path::Path;

#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let n = (width as usize) * (height as usize) * 3;
        Self {
            width,
            height,
            pixels: vec![0; n],
        }
    }

    pub fn from_scale(base_w: u32, base_h: u32, scale_percent: u8) -> Self {
        let s = (scale_percent as u32).clamp(50, 100);
        Self::new((base_w * s / 100).max(64), (base_h * s / 100).max(64))
    }

    #[inline]
    fn put(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = ((y as usize) * (self.width as usize) + (x as usize)) * 3;
        self.pixels[i] = r;
        self.pixels[i + 1] = g;
        self.pixels[i + 2] = b;
    }

    #[inline]
    fn fill_span(&mut self, y: u32, x0: u32, x1: u32, r: u8, g: u8, b: u8) {
        if y >= self.height {
            return;
        }
        let x0 = x0.min(self.width);
        let x1 = x1.min(self.width);
        if x0 >= x1 {
            return;
        }
        let row = (y as usize) * (self.width as usize) * 3;
        for x in x0..x1 {
            let i = row + (x as usize) * 3;
            self.pixels[i] = r;
            self.pixels[i + 1] = g;
            self.pixels[i + 2] = b;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
        }
    }
}

impl Camera {
    pub fn advance(&mut self, tick: u64) {
        self.yaw = (tick as f32) * 0.01;
        self.x = self.yaw.sin() * 8.0;
        self.y = self.yaw.cos() * 4.0;
    }
}

pub fn draw_scene(fb: &mut Framebuffer, tick: u64, render_distance: u8, cam: &Camera) {
    let w = fb.width;
    let h = fb.height;
    let horizon =
        (h as f32 * (0.52 + cam.y * 0.002)).clamp(h as f32 * 0.35, h as f32 * 0.7) as u32;

    for y in 0..horizon {
        let t = y as f32 / horizon.max(1) as f32;
        fb.fill_span(
            y,
            0,
            w,
            (70.0 + 50.0 * t) as u8,
            (130.0 + 60.0 * t) as u8,
            (210.0 - 20.0 * t) as u8,
        );
    }

    let sun_x = ((w as f32 * 0.3) + cam.x * 3.0) as u32;
    let sun_y = h / 10;
    let sun_r = (h / 18).max(3);
    fill_circle(fb, sun_x, sun_y, sun_r, 255, 236, 110);

    for y in horizon..h {
        let t = (y - horizon) as f32 / (h - horizon).max(1) as f32;
        fb.fill_span(
            y,
            0,
            w,
            (30.0 + 50.0 * t) as u8,
            (110.0 - 25.0 * t) as u8,
            (35.0 + 15.0 * t) as u8,
        );
    }

    let cols = ((render_distance as u32) * 2).clamp(10, 28);
    let col_w = (w / cols).max(2);
    let scroll = (cam.x * 2.0) as i32 + (tick as i32 / 2);

    for c in 0..cols {
        let world_x = c as i32 + scroll;
        let height = heightmap(world_x);
        let blocks = (2 + height).min(14);
        let bx = c * col_w;
        for by in 0..blocks {
            let y0 = horizon.saturating_sub((by + 1) * col_w);
            let (cr, cg, cb) = block_color(by, blocks);
            fill_rect_fast(fb, bx, y0, col_w, col_w, cr, cg, cb);
        }
    }

    let cx = w / 2;
    let cy = h / 2;
    for d in 0..5 {
        fb.put(cx.saturating_sub(d), cy, 255, 255, 255);
        fb.put(cx.saturating_add(d), cy, 255, 255, 255);
        fb.put(cx, cy.saturating_sub(d), 255, 255, 255);
        fb.put(cx, cy.saturating_add(d), 255, 255, 255);
    }
}

fn heightmap(x: i32) -> u32 {
    let x = x.rem_euclid(256);
    let a = ((x * 13) ^ (x * 7)) & 7;
    let b = ((x * 3) + 11) & 5;
    (a as u32 + b as u32) % 9
}

fn block_color(level: u32, top: u32) -> (u8, u8, u8) {
    if level + 1 == top {
        (56, 150, 64)
    } else if level + 2 >= top {
        (110, 78, 48)
    } else {
        (90, 90, 90)
    }
}

fn fill_rect_fast(fb: &mut Framebuffer, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8) {
    let y1 = y.saturating_add(h).min(fb.height);
    let x1 = x.saturating_add(w).min(fb.width);
    for row in y..y1 {
        fb.fill_span(row, x, x1, r, g, b);
    }
}

fn fill_circle(fb: &mut Framebuffer, cx: u32, cy: u32, radius: u32, r: u8, g: u8, b: u8) {
    let rad = radius as i32;
    for dy in -rad..=rad {
        for dx in -rad..=rad {
            if dx * dx + dy * dy <= rad * rad {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x >= 0 && y >= 0 {
                    fb.put(x as u32, y as u32, r, g, b);
                }
            }
        }
    }
}

pub fn write_ppm(fb: &Framebuffer, path: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    write!(f, "P6\n{} {}\n255\n", fb.width, fb.height)?;
    f.write_all(&fb.pixels)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_fast_path() {
        let mut fb = Framebuffer::new(640, 360);
        let cam = Camera::default();
        draw_scene(&mut fb, 10, 8, &cam);
        assert!(fb.pixels.iter().any(|&p| p > 0));
    }
}
