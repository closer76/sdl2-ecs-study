use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::time::Duration;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;
const GRID_W: u32 = 40;
const GRID_H: u32 = 40;
const GRID_COUNT_X: usize = (SCREEN_W / GRID_W) as usize;
const GRID_COUNT_Y: usize = (SCREEN_H / GRID_H) as usize;

#[derive(Debug)]
struct Player {
    position: Point,
    sprite: Rect,
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

    let players = vec![
        Player {
            position: Point::new(0, 0),
            sprite: Rect::new(0, 0, 26, 36),
        },
        Player {
            position: Point::new(-100, -100),
            sprite: Rect::new(26, 0, 26, 36),
        },
    ];

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
