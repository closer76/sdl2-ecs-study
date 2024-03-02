use sdl2::rect::{Point, Rect};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match *self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Debug, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Velocity {
    pub speed: i32,
    pub direction: Direction,
}

#[derive(Debug, Component, Clone)]
#[storage(VecStorage)]
pub struct Sprite {
    /// Index of the spritesheet
    pub spritesheet: usize,
    /// Region in spritesheet that should be rendered
    pub region: Rect,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct MovementAnimation {
    pub current_frame: usize,
    pub up_frames: Vec<Sprite>,
    pub down_frames: Vec<Sprite>,
    pub left_frames: Vec<Sprite>,
    pub right_frames: Vec<Sprite>,
}
