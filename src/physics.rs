use crate::components::*;
use specs::prelude::*;

use crate::Velocity;

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);
    fn run(&mut self, mut data: Self::SystemData) {
        for (pos, vel) in (&mut data.0, &data.1).join() {
            match vel.direction {
                Direction::Left => pos.0 = pos.0.offset(-vel.speed, 0),
                Direction::Right => pos.0 = pos.0.offset(vel.speed, 0),
                Direction::Up => pos.0 = pos.0.offset(0, -vel.speed),
                Direction::Down => pos.0 = pos.0.offset(0, vel.speed),
            }
        }
    }
}
