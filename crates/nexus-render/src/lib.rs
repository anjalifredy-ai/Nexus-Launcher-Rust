//! Software render path — zero GPU deps.
//! Produces RGB frames (sky + ground + simple blocks) until wgpu links in.

use std::path::Path;

#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    /// RGB8 tightly packed
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

    pub fn from_scale(base_w: u32, base_h: u32, scale_percent: u8) -> Self {
        let s = (scale_percent as u32).clamp(50, 100);
        let w = (base_w * s / 100).max(64);
        let h = (base_h * s / 100).max(64);
        Self::new(w, h)
    }
}

/// Draw one scene frame. `tick` animates sun/blocks slightly.
pub fn draw_scene(fb: &mut Framebuffer, tick: u64, render_distance: u8) {
    let w = fb.width;
    let h = fb.height;
    let horizon = h * 55 / 100;

    for y in 0..horizon {
        let t = y as f32 / horizon as f32;
        let r = (80.0 + 40.0 * t) as u8;
        let g = (140.0 + 50.0 * t) as u8;
        let b = (220.0 - 30.0 * t) as u8;
        for x in 0..w {
            fb.put(x, y, r, g, b);
        }
    }

    let sun_x = w / 4 + ((tick / 8) % (w as u64 / 2)) as u32;
    let sun_y = h / 8;
    let sun_r = (h / 20).max(4);
    for dy in 0..sun_r * 2 {
        for dx in 0..sun_r * 2 {
            let cx = dx as i32 - sun_r as i32;
            let cy = dy as i32 - sun_r as i32;
            if cx * cx + cy * cy <= (sun_r as i32) * (sun_r as i32) {
                fb.put(
                    sun_x.saturating_add(dx).saturating_sub(sun_r),
                    sun_y.saturating_add(dy).saturating_sub(sun_r),
                    255,
                    240,
                    120,
                );
            }
        }
    }

    for y in horizon..h {
        let t = (y - horizon) as f32 / (h - horizon).max(1) as f32;
        let r = (34.0 + 40.0 * t) as u8;
        let g = (120.0 - 30.0 * t) as u8;
        let b = (40.0 + 10.0 * t) as u8;
        for x in 0..w {
            fb.put(x, y, r, g, b);
        }
    }

    let cols = (render_distance as u32).clamp(4, 16);
    let block = (w / (cols * 2)).max(4);
    for c in 0..cols {
        let bx = w / 8 + c * block * 2 + ((tick / 3) as u32 % 3);
        let height_blocks = 2 + ((c * 7 + (tick as u32 / 10)) % 6);
        for by in 0..height_blocks {
            let y0 = horizon.saturating_sub((by + 1) * block);
            fill_rect(fb, bx, y0, block, block, dirt_color(by));
            if by + 1 == height_blocks {
                fill_rect(fb, bx, y0, block, block / 4, (60, 160, 70));
            }
        }
    }

    let cx = w / 2;
    let cy = h / 2;
    for d in 0..6 {
        fb.put(cx.saturating_sub(d), cy, 255, 255, 255);
        fb.put(cx.saturating_add(d), cy, 255, 255, 255);
        fb.put(cx, cy.saturating_sub(d), 255, 255, 255);
        fb.put(cx, cy.saturating_add(d), 255, 255, 255);
    }
}

fn dirt_color(level: u32) -> (u8, u8, u8) {
    match level % 3 {
        0 => (120, 85, 55),
        1 => (100, 70, 45),
        _ => (90, 65, 40),
    }
}

fn fill_rect(fb: &mut Framebuffer, x: u32, y: u32, w: u32, h: u32, rgb: (u8, u8, u8)) {
    for row in y..y.saturating_add(h).min(fb.height) {
        for col in x..x.saturating_add(w).min(fb.width) {
            fb.put(col, row, rgb.0, rgb.1, rgb.2);
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
    fn draws_without_panic() {
        let mut fb = Framebuffer::new(320, 180);
        draw_scene(&mut fb, 42, 8);
        assert_eq!(fb.pixels.len(), 320 * 180 * 3);
        assert!(fb.pixels.iter().any(|&p| p > 0));
    }
}
