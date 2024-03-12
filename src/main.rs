mod animator;
mod components;
mod constants;
mod keyboard;
mod physics;
mod renderer;

use crate::components::*;
use crate::constants::*;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadSurface, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use specs::prelude::*;
use std::time::Duration;
use tiled::LayerType;
use tiled::Loader;
use tiled::Map;

#[derive(Debug)]
pub struct InputBuffer {
    pub dir_queue: Vec<Direction>,
    pub dir_state: [bool; 4],
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

    pub fn get_velocity(&self, saved_dir: Direction) -> Velocity {
        let mut vel = Velocity {
            direction: saved_dir,
            speed: 0,
        };
        if let Some(last_dir) = self.dir_queue.last() {
            vel.direction = *last_dir;
            if self.dir_state[last_dir.opposite() as usize] {
                vel.speed = 0;
            } else {
                vel.speed = PLAYER_MOVEMENT_SPEED;
            }
        } else {
            vel.speed = 0;
        }

        vel
    }
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

    // Creates ECS
    let mut dispatcher = DispatcherBuilder::new()
        .with(keyboard::Keyboard, "Keyboard", &[])
        .with(physics::Physics, "Phyisics", &["Keyboard"])
        .with(animator::Animator, "Animator", &["Keyboard"])
        .build();
    let mut world = World::new();
    dispatcher.setup(&mut world);

    // Loads textures
    let texture_creator = canvas.texture_creator();
    let mut textures = vec![
        texture_creator.load_texture("assets/bardo.png")?,
    ];

    // Loads tile map
    let mut tiled_loader = Loader::new();
    let tile_map = tiled_loader
        .load_tmx_map("assets/combat_Dungeon.tmx")
        .unwrap();

    let tile_texture = compose_tile_map(&tile_map, &texture_creator)?;
    textures.push(tile_texture);

    // Creates grids data
    let delta_up = 0.1;
    let delta_down = -0.03;

    let mut levels = (0..GRID_COUNT_Y)
        .map(|_| (0..GRID_COUNT_X).map(|_| 0.0).collect::<Vec<f64>>())
        .collect::<Vec<_>>();

    let mut cur_x = -1;
    let mut cur_y = -1;

    // Creates entities
    let player_top_left_frame = Rect::new(0, 0, 26, 36);
    let player_animation = MovementAnimation {
        current_frame: 0,
        up_frames: character_animation_frames(0, player_top_left_frame, Direction::Up),
        down_frames: character_animation_frames(0, player_top_left_frame, Direction::Down),
        left_frames: character_animation_frames(0, player_top_left_frame, Direction::Left),
        right_frames: character_animation_frames(0, player_top_left_frame, Direction::Right),
    };

    let default_sprite = player_animation.right_frames[0].clone();
    world
        .create_entity()
        .with(KeyboardControlled)
        .with(Position(Point::new(0, 0)))
        .with(Velocity {
            speed: 0,
            direction: Direction::Right,
        })
        .with(default_sprite.clone()) // sets default sprite
        .with(player_animation)
        .build();

    world
        .create_entity()
        .with(Position(Point::new(-100, -100)))
        .with(Velocity {
            speed: 0,
            direction: Direction::Right,
        })
        .with(default_sprite.clone()) // sets default sprite
        .build();

    // Create resources
    let mut input_buffer = InputBuffer {
        dir_queue: vec![],
        dir_state: [false; 4],
    };
    let mut last_dir = input_buffer.get_velocity(Direction::Right).direction;
    world.insert(input_buffer.get_velocity(last_dir));

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
                } => {
                    input_buffer.add_direction(Direction::Left);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    input_buffer.add_direction(Direction::Right);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    input_buffer.add_direction(Direction::Up);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    input_buffer.add_direction(Direction::Down);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    input_buffer.remove_direction(Direction::Left);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    input_buffer.remove_direction(Direction::Right);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    input_buffer.remove_direction(Direction::Up);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    input_buffer.remove_direction(Direction::Down);
                }

                Event::MouseMotion { x, y, .. } => {
                    cur_x = x;
                    cur_y = y;
                }
                _ => {}
            }
            let new_vel = input_buffer.get_velocity(last_dir);
            last_dir = new_vel.direction;
            let mut vel = world.write_resource();
            *vel = new_vel;
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

        // Update
        dispatcher.dispatch(&world);
        world.maintain();

        // Render
        renderer::render(&mut canvas, &levels, &textures, world.system_data())?;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn direction_spritesheet_row(direction: Direction) -> i32 {
    match direction {
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
        Direction::Down => 0,
    }
}

fn character_animation_frames(
    spritesheet: usize,
    top_left_frame: Rect,
    direction: Direction,
) -> Vec<Sprite> {
    let row = direction_spritesheet_row(direction);
    let (frame_width, frame_height) = top_left_frame.size();
    (0..3)
        .map(|col| Sprite {
            spritesheet,
            region: Rect::new(
                top_left_frame.x + col * frame_width as i32,
                top_left_frame.y + row * frame_height as i32,
                frame_width,
                frame_height,
            ),
        })
        .collect()
}

fn compose_tile_map<'a>(
    map: &Map,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>, String> {
    let tile_width = map.tile_width;
    let tile_height = map.tile_height;
    let layer = match map
        .get_layer(0)
        .ok_or(String::from("Can't get layer 0."))?
        .layer_type()
    {
        LayerType::Tiles(layer) => Ok(layer),
        _ => Err(String::from("Incorrect layer type.")),
    }?;
    let (x_count, y_count) = match layer {
        tiled::TileLayer::Finite(finite) => (finite.width(), finite.height()),
        _ => return Err(String::from("format error.")),
    };

    let mut canvas = Surface::new(
        tile_width * x_count,
        tile_height * y_count,
        PixelFormatEnum::ARGB8888,
    )?;

    let tileset = map.tilesets().first().ok_or("No tilesets.")?;
    let image_info = tileset.image.as_ref().unwrap();
    let src = Surface::from_file(image_info.source.clone())?;

    for y in 0..y_count {
        for x in 0..x_count {
            let tile_id = layer.get_tile(x as i32, y as i32).unwrap().id();
            let tile_x = tile_id % tileset.columns;
            let tile_y = tile_id / tileset.columns;
            src.blit(
                Rect::new(
                    (tile_x * (tileset.spacing + tile_width) + tileset.margin) as i32,
                    (tile_y * (tileset.spacing + tile_height) + tileset.margin) as i32,
                    tile_width,
                    tile_height,
                ),
                &mut canvas,
                Rect::new(
                    (x * tile_width) as i32,
                    (y * tile_height) as i32,
                    tile_width,
                    tile_height,
                ),
            )?;
        }
    }

    canvas
        .as_texture(texture_creator)
        .map_err(|e| e.to_string())
}
