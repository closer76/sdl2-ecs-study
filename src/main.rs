extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

use rand::prelude::*;

pub fn main() -> Result<(), String> {
    const SCREEN_W: u32 = 800;
    const SCREEN_H: u32 = 600;
    const GRID_W: u32 = 40;
    const GRID_H: u32 = 40;
    const GRID_COUNT_X: usize = (SCREEN_W / GRID_W) as usize;
    const GRID_COUNT_Y: usize = (SCREEN_H / GRID_H) as usize;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", SCREEN_W, SCREEN_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut rng = thread_rng();
    let delta_up = 0.1;
    let delta_down = -0.03;

    let mut levels = (0..GRID_COUNT_Y)
        .map(|_| (0..GRID_COUNT_X).map(|_| rng.gen()).collect::<Vec<f64>>())
        .collect::<Vec<_>>();

    let mut deltas = (0..GRID_COUNT_Y)
        .map(|_y| {
            (0..GRID_COUNT_X)
                .map(|_x| if rng.gen() { delta_up } else { delta_down })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // update levels
        for y in 0..GRID_COUNT_Y {
            for x in 0..GRID_COUNT_X {
                levels[y][x] += deltas[y][x];
                if levels[y][x] > 1.0 {
                    levels[y][x] = 1.0;
                    deltas[y][x] = delta_down;
                } else if levels[y][x] < 0.0 {
                    levels[y][x] = 0.0;
                    deltas[y][x] = delta_up;
                }
                let red = (255.0 * levels[y][x]) as u8;
                canvas.set_draw_color(Color::RGB(red, 0, 0));
                canvas.draw_rect(Rect::new(
                    (x as u32 * GRID_W + 1) as i32,
                    (y as u32 * GRID_H + 1) as i32,
                    (GRID_W - 2) as u32,
                    (GRID_H - 2) as u32,
                ))?;
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
