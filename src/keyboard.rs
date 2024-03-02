use specs::prelude::*;

use crate::{InputBuffer, KeyboardControlled, Velocity};

const PLAYER_MOVEMENT_SPEED: i32 = 5;

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = (
        ReadExpect<'a, InputBuffer>,
        ReadStorage<'a, KeyboardControlled>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let input_buf = &*data.0;

        for (_, vel) in (&data.1, &mut data.2).join() {
            if let Some(last_dir) = input_buf.dir_queue.last() {
                vel.direction = *last_dir;
                if input_buf.dir_state[last_dir.opposite() as usize] {
                    vel.speed = 0;
                } else {
                    vel.speed = PLAYER_MOVEMENT_SPEED;
                }
            } else {
                vel.speed = 0;
            }
        }
    }
}
