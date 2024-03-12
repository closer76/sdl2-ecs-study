use crate::components::*;
use crate::constants::*;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

pub type SystemData<'a> = (ReadStorage<'a, Position>, ReadStorage<'a, Sprite>);

pub fn render(
    canvas: &mut WindowCanvas,
    levels: &Vec<Vec<f64>>,
    textures: &[Texture],
    data: SystemData,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // background
    canvas.copy(
        &textures[1],
        Rect::new(0, 0, SCREEN_W, SCREEN_H),
        Rect::new(0, 0, SCREEN_W, SCREEN_H),
    )?;

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

    for (pos, sprite) in (&data.0, &data.1).join() {
        let screen_position = screen_from_world(canvas, &pos.0)?;
        let screen_rect = Rect::from_center(
            screen_position,
            sprite.region.width(),
            sprite.region.height(),
        );
        canvas.copy(&textures[sprite.spritesheet], sprite.region, screen_rect)?;
    }
    canvas.present();
    Ok(())
}

fn screen_from_world(canvas: &WindowCanvas, world_coord: &Point) -> Result<Point, String> {
    let (width, height) = canvas.output_size()?;
    Ok(Point::new(
        width as i32 / 2 + world_coord.x,
        height as i32 / 2 + world_coord.y,
    ))
}
