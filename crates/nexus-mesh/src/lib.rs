//! Chunk mesh with neighbor face culling. GPU-ready.

use nexus_world::{BlockId, Chunk, CHUNK_SIZE};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

fn color(block: BlockId) -> (u8, u8, u8) {
    match block {
        BlockId::Air => (0, 0, 0),
        BlockId::Grass => (56, 150, 64),
        BlockId::Dirt => (110, 78, 48),
        BlockId::Stone => (100, 100, 105),
    }
}

fn push_quad(out: &mut Mesh, v: [Vertex; 4]) {
    let i = out.vertices.len() as u32;
    out.vertices.extend_from_slice(&v);
    out.indices
        .extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
}

#[inline]
fn height_at(chunk: &Chunk, x: i32, z: i32) -> u8 {
    if x < 0 || z < 0 || x >= CHUNK_SIZE as i32 || z >= CHUNK_SIZE as i32 {
        return 0;
    }
    chunk.column(x as usize, z as usize).height
}

pub fn mesh_chunk_solid(chunk: &Chunk, out: &mut Mesh) {
    let base_x = chunk.cx as f32 * CHUNK_SIZE as f32;
    let base_z = chunk.cz as f32 * CHUNK_SIZE as f32;

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let col = chunk.column(x, z);
            if col.height == 0 || col.top == BlockId::Air {
                continue;
            }
            let (tr, tg, tb) = color(col.top);
            let (sr, sg, sb) = if col.height > 12 {
                color(BlockId::Stone)
            } else {
                color(BlockId::Dirt)
            };
            let y1 = col.height as f32;
            let x0 = base_x + x as f32;
            let z0 = base_z + z as f32;
            let x1 = x0 + 1.0;
            let z1 = z0 + 1.0;
            let xi = x as i32;
            let zi = z as i32;

            push_quad(
                out,
                [
                    Vertex { x: x0, y: y1, z: z0, r: tr, g: tg, b: tb },
                    Vertex { x: x1, y: y1, z: z0, r: tr, g: tg, b: tb },
                    Vertex { x: x1, y: y1, z: z1, r: tr, g: tg, b: tb },
                    Vertex { x: x0, y: y1, z: z1, r: tr, g: tg, b: tb },
                ],
            );

            if height_at(chunk, xi, zi - 1) < col.height {
                push_quad(
                    out,
                    [
                        Vertex { x: x0, y: 0.0, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: 0.0, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: y1, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: y1, z: z0, r: sr, g: sg, b: sb },
                    ],
                );
            }
            if height_at(chunk, xi, zi + 1) < col.height {
                push_quad(
                    out,
                    [
                        Vertex { x: x1, y: 0.0, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: 0.0, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: y1, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: y1, z: z1, r: sr, g: sg, b: sb },
                    ],
                );
            }
            if height_at(chunk, xi - 1, zi) < col.height {
                push_quad(
                    out,
                    [
                        Vertex { x: x0, y: 0.0, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: 0.0, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: y1, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x0, y: y1, z: z1, r: sr, g: sg, b: sb },
                    ],
                );
            }
            if height_at(chunk, xi + 1, zi) < col.height {
                push_quad(
                    out,
                    [
                        Vertex { x: x1, y: 0.0, z: z0, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: 0.0, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: y1, z: z1, r: sr, g: sg, b: sb },
                        Vertex { x: x1, y: y1, z: z0, r: sr, g: sg, b: sb },
                    ],
                );
            }
        }
    }
}

pub fn mesh_chunk_tops(chunk: &Chunk, out: &mut Mesh) {
    mesh_chunk_solid(chunk, out);
}
