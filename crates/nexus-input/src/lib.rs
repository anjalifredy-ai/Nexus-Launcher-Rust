//! Input state for desktop/Android (touch mapped to look/move).

#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    pub move_x: f32,
    pub move_z: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub jump: bool,
    pub primary_action: bool,
}

impl InputState {
    pub fn apply_keys(&mut self, forward: bool, back: bool, left: bool, right: bool) {
        self.move_z = (forward as i8 - back as i8) as f32;
        self.move_x = (right as i8 - left as i8) as f32;
    }

    pub fn apply_touch_look(&mut self, dx: f32, dy: f32) {
        self.look_yaw += dx * 0.005;
        self.look_pitch = (self.look_pitch + dy * 0.005).clamp(-1.2, 1.2);
    }

    pub fn clear_edges(&mut self) {
        self.jump = false;
        self.primary_action = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 12.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Player {
    pub fn integrate(&mut self, input: &InputState, dt: f32) {
        self.yaw += input.look_yaw;
        self.pitch = (self.pitch + input.look_pitch).clamp(-1.2, 1.2);
        let speed = 12.0 * dt;
        let (sin, cos) = self.yaw.sin_cos();
        self.x += (input.move_x * cos + input.move_z * sin) * speed;
        self.z += (input.move_z * cos - input.move_x * sin) * speed;
        if input.jump {
            self.y += 0.5;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_forward() {
        let mut p = Player::default();
        let mut i = InputState::default();
        i.move_z = 1.0;
        p.integrate(&i, 0.1);
        assert!(p.x != 0.0 || p.z != 0.0);
    }
}
