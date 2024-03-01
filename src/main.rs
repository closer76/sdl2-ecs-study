use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::time::Duration;

use specs::prelude::*;
use specs_derive::Component;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;
const GRID_W: u32 = 40;
const GRID_H: u32 = 40;
const GRID_COUNT_X: usize = (SCREEN_W / GRID_W) as usize;
const GRID_COUNT_Y: usize = (SCREEN_H / GRID_H) as usize;

const PLAYER_MOVEMENT_SPEED: i32 = 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
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
#[storage(VecStorage)]
struct Position(Point);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Velocity {
    speed: i32,
    direction: Direction,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Sprite {
    /// Index of the spritesheet
    spritesheet: usize,
    /// Region in spritesheet that should be rendered
    region: Rect,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct MovementAnimation {
    current_frame: usize,
    up_frames: Vec<Sprite>,
    down_frames: Vec<Sprite>,
    left_frames: Vec<Sprite>,
    right_frames: Vec<Sprite>,
}

#[derive(Debug)]
struct InputBuffer {
    dir_queue: Vec<Direction>,
    dir_state: [bool; 4],
}

impl InputBuffer {
    pub fn add_direction(&mut self, dir: Direction) {
        self.dir_state[dir as usize] = true;
        self.dir_queue.push(dir);
    }

    pub fn remove_direction(&mut self, dir: Direction) {
        self.dir_state[dir as usize] = false;
        while let Some(last_dir) = self.dir_queue.pop() {
            if self.dir_state[last_dir as usize] {
                self.dir_queue.push(last_dir);
                break;
            }
        }
    }

    pub fn apply_to_player(&self, player: &mut Player) {
        if let Some(last_dir) = self.dir_queue.last() {
            player.direction = *last_dir;
            if self.dir_state[last_dir.opposite() as usize] {
                player.speed = 0;
            } else {
                player.speed = PLAYER_MOVEMENT_SPEED;
            }
        } else {
            player.speed = 0;
        }
    }
}

#[derive(Debug)]
struct Player {
    position: Point,
    sprite: Rect,
    sprite_index: i32,
    speed: i32,
    direction: Direction,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(InitFlag::JPG | InitFlag::PNG)?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", SCREEN_W, SCREEN_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/bardo.png")?;

    let delta_up = 0.1;
    let delta_down = -0.03;

    let mut levels = (0..GRID_COUNT_Y)
        .map(|_| (0..GRID_COUNT_X).map(|_| 0.0).collect::<Vec<f64>>())
        .collect::<Vec<_>>();

    let mut cur_x = -1;
    let mut cur_y = -1;

    let player_top_left_frame = Rect::new(0, 0, 26, 36);
    let movement_animation = MovementAnimation {
        current_frame: 0,
        up_frames: character_animation_frames(0, player_top_left_frame, Direction::Up),
        down_frames: character_animation_frames(0, player_top_left_frame, Direction::Down),
        left_frames: character_animation_frames(0, player_top_left_frame, Direction::Left),
        right_frames: character_animation_frames(0, player_top_left_frame, Direction::Right),
    };

    let mut players = vec![
        Player {
            position: Point::new(0, 0),
            sprite: Rect::new(0, 0, 26, 36),
            sprite_index: 0,
            speed: 0,
            direction: Direction::Right,
        },
        Player {
            position: Point::new(-100, -100),
            sprite: Rect::new(26, 0, 26, 36),
            sprite_index: 0,
            speed: 0,
            direction: Direction::Right,
        },
    ];

    let mut input_buffer = InputBuffer {
        dir_queue: vec![],
        dir_state: [false; 4],
    };

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // Handle input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => input_buffer.add_direction(Direction::Left),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => input_buffer.add_direction(Direction::Right),
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => input_buffer.add_direction(Direction::Up),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => input_buffer.add_direction(Direction::Down),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => input_buffer.remove_direction(Direction::Left),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => input_buffer.remove_direction(Direction::Right),
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => input_buffer.remove_direction(Direction::Up),
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => input_buffer.remove_direction(Direction::Down),

                Event::MouseMotion { x, y, .. } => {
                    cur_x = x;
                    cur_y = y;
                }
                _ => {}
            }
        }

        // Update levels
        let in_y = if cur_y >= 0 { cur_y / 40 } else { -1 };
        let in_x = if cur_x >= 0 { cur_x / 40 } else { -1 };
        for y in 0..GRID_COUNT_Y {
            for x in 0..GRID_COUNT_X {
                if (y as i32) == in_y && (x as i32) == in_x {
                    levels[y][x] += delta_up;
                    if levels[y][x] > 1.0 {
                        levels[y][x] = 1.0;
                    }
                } else {
                    levels[y][x] += delta_down;
                    if levels[y][x] < 0.0 {
                        levels[y][x] = 0.0;
                    }
                }
            }
        }

        // Update players
        input_buffer.apply_to_player(&mut players[0]);
        for player in &mut players {
            match player.direction {
                Direction::Left => {
                    player.position = player.position.offset(-player.speed, 0);
                }
                Direction::Right => {
                    player.position = player.position.offset(player.speed, 0);
                }
                Direction::Up => {
                    player.position = player.position.offset(0, -player.speed);
                }
                Direction::Down => {
                    player.position = player.position.offset(0, player.speed);
                }
            }

            if player.speed != 0 {
                player.sprite_index = (player.sprite_index + 1) % 3;
            } else {
                player.sprite_index = 0;
            }

            player.sprite.x = player.sprite_index * 26;
            player.sprite.y = direction_spritesheet_row(player.direction) * 36;
        }

        // Render
        render(&mut canvas, &levels, &texture, &players)?;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn screen_from_world(canvas: &WindowCanvas, world_coord: &Point) -> Result<Point, String> {
    let (width, height) = canvas.output_size()?;
    Ok(Point::new(
        width as i32 / 2 + world_coord.x,
        height as i32 / 2 + world_coord.y,
    ))
}

fn direction_spritesheet_row(direction: Direction) -> i32 {
    match direction {
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
        Direction::Down => 0,
    }
}

fn character_animation_frames(spritesheet: usize, top_left_frame: Rect, direction: Direction) -> Vec<Sprite> {
    let row = direction_spritesheet_row(direction);
    let (frame_width, frame_height) = top_left_frame.size();
    (0..3).map(|col| {
        Sprite {
            spritesheet,
            region: Rect::new(
                top_left_frame.x + col * frame_width as i32,
                top_left_frame.y + row * frame_height as i32,
                frame_width,
                frame_height
            )
        }
    })
    .collect()
}

fn render(
    canvas: &mut WindowCanvas,
    levels: &Vec<Vec<f64>>,
    texture: &Texture,
    players: &Vec<Player>,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for y in 0..GRID_COUNT_Y {
        for x in 0..GRID_COUNT_X {
            let red = (255.0 * levels[y][x]) as u8;
            canvas.set_draw_color(Color::RGB(red, 0, 0));
            canvas.fill_rect(Rect::new(
                (x as u32 * GRID_W + 1) as i32,
                (y as u32 * GRID_H + 1) as i32,
                (GRID_W - 2) as u32,
                (GRID_H - 2) as u32,
            ))?;
        }
    }

    players
        .iter()
        .map(|player| {
            let screen_position = screen_from_world(canvas, &player.position)?;
            let screen_rect = Rect::from_center(
                screen_position,
                player.sprite.width(),
                player.sprite.height(),
            );
            canvas.copy(texture, player.sprite, screen_rect)?;
            Ok(())
        })
        .collect::<Result<(), String>>()?;

    canvas.present();

    Ok(())
}
