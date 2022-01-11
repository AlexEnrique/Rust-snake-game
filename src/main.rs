extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events, EventLoop};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent,
    ButtonArgs, ButtonEvent};
use piston::window::WindowSettings;
use piston::ButtonState;

extern crate piston_window;
use piston_window::Transformed;
use piston_window::text::*;

use graphics::types::{FontSize, *};
use graphics::{Context, Text, *};
use opengl_graphics::{GlyphCache};

extern crate gfx_device_gl;
use gfx_device_gl::Factory;
use std::rc::Rc;

use std::collections::LinkedList;

use rand::random;

/*trait DrawText {
    fn draw_text(
        &mut self,
        text: &str,
        r: [f64; 4],
        color: [f32; 4],
        size: FontSize,
        halign: TextAlignment,
        valign: TextVerticalAlignment,
        glyphs: &mut GlyphCache,
        c: &Context,
    );
}*/




fn random_pos(window_size: u32, grid_size: u32, constrain: Option<f64>) -> f64 {
    let mut x: f64 = ((random::<u32>() % (window_size / grid_size)) * grid_size).into();

    match constrain {
        Some(cons) => while x == cons {
            x = ((random::<u32>() % (window_size / grid_size)) * grid_size).into();
        },
        _ => (),
    }

    x 
}

pub struct Game {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
    window_size: u32,
    grid_size: u32, 
    score: u32, 
    factory: Factory,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::text;
        use graphics::glyph_cache::rusttype::GlyphCache;
        use graphics::ImageSize;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.8];
        //const text_score: Text = Text::new(12);

        //let mut glyph_cache = 

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen
            graphics::clear(GREEN, gl);

            //text_score.draw("test", &glyph_cache, &c.draw_state, c.transform, gl.into());
        });


        self.snake.render(args);
        self.food.render(args);
    }

    fn update(&mut self, args: &UpdateArgs) { // Handle collisions here
        if self.snake.update(args) { // Moved
            // Collided with food
            if *self.snake.body.front().unwrap() == self.food.pos { 
                let mut food_x: f64 = random_pos(self.window_size, self.grid_size, None);
                let mut food_y: f64 = random_pos(self.window_size, self.grid_size, None);
            
                while self.snake.body.contains(&[food_x, food_y]) {
                    food_x = random_pos(self.window_size, self.grid_size, None);
                    food_y = random_pos(self.window_size, self.grid_size, None);
                }

                self.food.pos = [food_x, food_y];

                self.snake.grow();
            }

            // Collied with itself
            let body_without_head = self.snake.body.clone().split_off(1);
            let head = self.snake.body.front().unwrap();
            if body_without_head.contains(head) { 
                self.reset();
            }

            // Collided with wall
            let head = self.snake.body.front().unwrap();

            if head.into_iter().any(|&x| x < 0.0 || x >= self.window_size.into()) {
                self.reset();
            }
        }
    }

    fn pressed(&mut self, args: &ButtonArgs) {
        self.snake.pressed(args);
    }

    fn reset(&mut self) {
        self.snake.mov_dir = Direction::None;
        self.snake.body = LinkedList::from([[0.0, 0.0]]);
        self.food.pos = [random_pos(self.window_size, self.grid_size, Some(0.0)), 
                         random_pos(self.window_size, self.grid_size, Some(0.0))];
    }
}

#[derive(PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
    None,
}

pub struct Snake {
    gl: GlGraphics,
    width: f64,
    body: LinkedList<[f64; 2]>,
    mov_dir: Direction, 
}

impl Snake {
    fn render(&mut self, args: &RenderArgs) {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        
        let mut iter = self.body.iter_mut();

        while let Some(pos) = iter.next() {
            let square = graphics::rectangle::square(pos[0], pos[1], self.width);

            self.gl.draw(args.viewport(), |c, gl| {
                let transform = c.transform;

                graphics::rectangle(RED, square, transform, gl);
            });
        }      
    }

    /// Move snake if valid, otherwise return false
    fn update(&mut self, _args: &UpdateArgs) -> bool {
        let mut pos = *self.body.front().unwrap(); 

        match self.mov_dir {
            Direction::Up => pos[1] = pos[1] - self.width,
            Direction::Down => pos[1] = pos[1] + self.width,
            Direction::Right => pos[0] = pos[0] + self.width,
            Direction::Left => pos[0] = pos[0] - self.width,
            _ => (),
        }

        self.body.push_front(pos);
        self.body.pop_back();

        match self.mov_dir {
            Direction::Up | Direction::Down | Direction::Right | Direction::Left => {return true;},
            _ => (),
        }

        false
    }


    fn pressed(&mut self, args: &ButtonArgs) { // Bug: If pressed to fast, the snake can turn back
        use piston::{Button, Key};

        match args.button {
            Button::Keyboard(key) => match key { 
                Key::Up if self.mov_dir != Direction::Down => self.mov_dir = Direction::Up,
                Key::Down if self.mov_dir != Direction::Up => self.mov_dir = Direction::Down,
                Key::Right if self.mov_dir != Direction::Left => self.mov_dir = Direction::Right,
                Key::Left if self.mov_dir != Direction::Right => self.mov_dir = Direction::Left,
                _ => (),
            },
            _ => (),
        }
    }

    fn grow(&mut self) {
        let tail: [f64; 2] = *self.body.back().unwrap();

        match self.mov_dir {
            Direction::Up => self.body.push_back([tail[0], tail[1] + self.width]),
            Direction::Down => self.body.push_back([tail[0], tail[1] - self.width]),
            Direction::Right => self.body.push_back([tail[0] - self.width, tail[1]]),
            Direction::Left => self.body.push_back([tail[0] + self.width, tail[1]]),
            _ => (),
        }
    }
}


// Food
pub struct Food {
    gl: GlGraphics,
    width: f64,
    pos: [f64; 2],
}

impl Food {
    fn render(&mut self, args: &RenderArgs) {
        let square = graphics::rectangle::square(self.pos[0], self.pos[1],
            self.width);

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(graphics::color::BLACK, square, transform, gl);
        });
    }
}


fn main() {
    // Using OpenGL
    let opengl = OpenGL::V3_2;

    const WINDOW_SIZE: u32 = 600;
    const GRID_SIZE: u32 = 20;

    // Creating the GlutinWindow
    let mut window: Window = WindowSettings::new("Snake Game", [WINDOW_SIZE, WINDOW_SIZE])
        .graphics_api(opengl)
        .build()
        .unwrap();

    // Create a new game and run it
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        snake: Snake {
            gl: GlGraphics::new(opengl),
            width: GRID_SIZE.into(),
            body: LinkedList::from([[0.0, 0.0]]),
            mov_dir: Direction::None,            
        },
        food: Food {
            gl: GlGraphics::new(opengl),
            width: GRID_SIZE.into(),
            pos: [random_pos(WINDOW_SIZE, GRID_SIZE, Some(0.0)), 
                  random_pos(WINDOW_SIZE, GRID_SIZE, Some(0.0))],
        },
        window_size: WINDOW_SIZE,
        grid_size: GRID_SIZE,
        score: 0,
        factory: Factory::new(Rc::new(5)),
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(args) = e.update_args() {
            game.update(&args);
        }

        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                game.pressed(&args);
            }
        }
    }

}
