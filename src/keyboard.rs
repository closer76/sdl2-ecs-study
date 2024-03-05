use specs::prelude::*;

use crate::{KeyboardControlled, Velocity};

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = (
        ReadExpect<'a, Velocity>,
        ReadStorage<'a, KeyboardControlled>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let input_vel = &*data.0;

        for (_, vel) in (&data.1, &mut data.2).join() {
            vel.direction = input_vel.direction;
            vel.speed = input_vel.speed;
        }
    }
}
