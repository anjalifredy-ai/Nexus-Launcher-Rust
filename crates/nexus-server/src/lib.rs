//! Integrated game server — authoritative world, players, block edits.
//! Singleplayer = client + this server in-process.

use nexus_input::{InputState, Player};
use nexus_mesh::{mesh_chunk_solid, Mesh};
use nexus_world::{BlockId, Chunk, World, CHUNK_SIZE};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub tick_rate: u32,
    pub view_chunks: i32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            tick_rate: 20,
            view_chunks: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemotePlayer {
    pub id: u32,
    pub name: String,
    pub player: Player,
}

pub struct GameServer {
    pub config: ServerConfig,
    pub world: World,
    pub players: Vec<RemotePlayer>,
    pub mesh: Mesh,
    pub tick: u64,
    mesh_dirty: bool,
    next_id: u32,
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        let mut world = World::default();
        let v = config.view_chunks;
        for cz in -v..=v {
            for cx in -v..=v {
                let _ = world.ensure_chunk(cx, cz);
            }
        }
        let mut mesh = Mesh::default();
        for cz in -v..=v {
            for cx in -v..=v {
                let ch = Chunk::generate(cx, cz);
                mesh_chunk_solid(&ch, &mut mesh);
            }
        }
        Self {
            config,
            world,
            players: Vec::new(),
            mesh,
            tick: 0,
            mesh_dirty: false,
            next_id: 1,
        }
    }

    pub fn join_player(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.players.push(RemotePlayer {
            id,
            name: name.into(),
            player: Player {
                x: 0.5,
                y: 20.0,
                z: 0.5,
                yaw: 0.0,
                pitch: 0.0,
            },
        });
        id
    }

    pub fn player_mut(&mut self, id: u32) -> Option<&mut RemotePlayer> {
        self.players.iter_mut().find(|p| p.id == id)
    }

    pub fn player(&self, id: u32) -> Option<&RemotePlayer> {
        self.players.iter().find(|p| p.id == id)
    }

    pub fn apply_input(&mut self, id: u32, input: &InputState, dt: f32) {
        let (gx, gz) = {
            let Some(p) = self.player_mut(id) else {
                return;
            };
            p.player.integrate(input, dt);
            (p.player.x.floor() as i32, p.player.z.floor() as i32)
        };
        let h = self.column_height(gx, gz) as f32;
        if let Some(p) = self.player_mut(id) {
            if p.player.y < h + 1.6 {
                p.player.y = h + 1.6;
            }
        }
    }

    pub fn column_height(&mut self, wx: i32, wz: i32) -> u8 {
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cz = wz.div_euclid(CHUNK_SIZE as i32);
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ch = self.world.ensure_chunk(cx, cz);
        ch.column(lx, lz).height
    }

    pub fn set_block(&mut self, x: i32, _y: i32, z: i32, block: BlockId) -> bool {
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cz = z.div_euclid(CHUNK_SIZE as i32);
        let lx = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = z.rem_euclid(CHUNK_SIZE as i32) as usize;
        let _ = self.world.ensure_chunk(cx, cz);
        self.world.set_top(cx, cz, lx, lz, block);
        self.mesh_dirty = true;
        true
    }

    pub fn break_block_at_feet(&mut self, id: u32) -> bool {
        let (x, z) = match self.player(id) {
            Some(p) => (p.player.x.floor() as i32, p.player.z.floor() as i32),
            None => return false,
        };
        self.set_block(x, 0, z, BlockId::Air)
    }

    pub fn place_block_ahead(&mut self, id: u32) -> bool {
        let (x, z, yaw) = match self.player(id) {
            Some(p) => (p.player.x, p.player.z, p.player.yaw),
            None => return false,
        };
        let (s, c) = yaw.sin_cos();
        let tx = (x + s * 2.0).floor() as i32;
        let tz = (z + c * 2.0).floor() as i32;
        self.set_block(tx, 0, tz, BlockId::Stone)
    }

    pub fn server_tick(&mut self) {
        self.tick += 1;
        if self.mesh_dirty {
            self.rebuild_mesh();
            self.mesh_dirty = false;
        }
    }

    fn rebuild_mesh(&mut self) {
        self.mesh.clear();
        let v = self.config.view_chunks;
        for cz in -v..=v {
            for cx in -v..=v {
                let ch = Chunk::generate(cx, cz);
                let live = self.world.ensure_chunk(cx, cz);
                let mut merged = ch;
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let o = live.column(x, z);
                        if o.top == BlockId::Air && o.height == 0 {
                            merged.columns[z][x].height = 0;
                            merged.columns[z][x].top = BlockId::Air;
                        } else {
                            merged.columns[z][x] = o.clone();
                        }
                    }
                }
                mesh_chunk_solid(&merged, &mut self.mesh);
            }
        }
    }

    pub fn mesh_tris(&self) -> usize {
        self.mesh.triangle_count()
    }

    pub fn chunk_count(&self) -> usize {
        self.world.chunk_count()
    }
}
