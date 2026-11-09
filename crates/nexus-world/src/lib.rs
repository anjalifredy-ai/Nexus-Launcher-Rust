//! Lightweight block world — column height storage for renderer + future mesh.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockId {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
}

#[derive(Debug, Clone)]
pub struct ChunkColumn {
    pub height: u8,
    pub top: BlockId,
}

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: u8 = 32;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub columns: [[ChunkColumn; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn generate(cx: i32, cz: i32) -> Self {
        let mut columns = std::array::from_fn(|_| {
            std::array::from_fn(|_| ChunkColumn {
                height: 0,
                top: BlockId::Air,
            })
        });
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let wx = cx * CHUNK_SIZE as i32 + x as i32;
                let wz = cz * CHUNK_SIZE as i32 + z as i32;
                let h = terrain_height(wx, wz);
                columns[z][x] = ChunkColumn {
                    height: h,
                    top: if h > 12 {
                        BlockId::Stone
                    } else if h > 4 {
                        BlockId::Grass
                    } else {
                        BlockId::Dirt
                    },
                };
            }
        }
        Self { cx, cz, columns }
    }

    pub fn column(&self, x: usize, z: usize) -> &ChunkColumn {
        &self.columns[z % CHUNK_SIZE][x % CHUNK_SIZE]
    }
}

fn terrain_height(x: i32, z: i32) -> u8 {
    let n = (x.wrapping_mul(374761393))
        .wrapping_add(z.wrapping_mul(668265263))
        .wrapping_add(x.wrapping_mul(z));
    let h = (n.rem_euclid(13) + 4) as u8;
    h.min(CHUNK_HEIGHT - 1)
}

#[derive(Debug, Default)]
pub struct World {
    chunks: Vec<Chunk>,
}

impl World {
    pub fn ensure_chunk(&mut self, cx: i32, cz: i32) -> &Chunk {
        if let Some(i) = self.chunks.iter().position(|c| c.cx == cx && c.cz == cz) {
            return &self.chunks[i];
        }
        self.chunks.push(Chunk::generate(cx, cz));
        self.chunks.last().unwrap()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_chunk() {
        let c = Chunk::generate(0, 0);
        assert!(c.column(0, 0).height >= 4);
    }
}
