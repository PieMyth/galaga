// PieMyth @ github

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

static WIDTH:i64= 400;
static HEIGHT:i64 = 600;
static GRIDSIZE:i64 = 20;

//Starting out layout was used from the examples
//in the Piston Library and this video
//https://www.youtube.com/watch?v=HCwMb0KslX8
struct Game {
    gl: GlGraphics,
    ship: Ship,
    enemies: Enemy,
}

impl Game {
    //Gets screen set and renders ship.
    fn render(&mut self, arg: &RenderArgs) {
        use graphics;

        self.gl.draw(arg.viewport(), |_c, gl | {
            graphics::clear(graphics::color::BLACK, gl);
        });

        self.ship.render(&mut self.gl, arg);
    }

    //Update based on event args time
    fn update(&mut self) {
        self.ship.update();
    }

    //Update Ship's movement or shoot depending on input
    fn pressed(&mut self, btn: &Button) {
        self.ship.kmove(btn);
    }
}

struct Ship {
    pos_x: i64,
    pos_y: i64,
    shots: Vec<Bullet>,
}

struct Enemy {
    list: Vec<Ship>,
}

struct Bullet {
    pos_x: i64,
    pos_y: i64,
}

impl Ship {
    //renders the ship, also will render the shots when created.
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let square = graphics::rectangle::square((self.pos_x * GRIDSIZE) as f64,
                     (self.pos_y * GRIDSIZE) as f64, GRIDSIZE as f64);

        gl.draw(args.viewport(), |c,gl| {
            let transform = c.transform;

            graphics::rectangle(graphics::color::WHITE, square, transform, gl);
        });

        for mut x in self.shots.iter_mut() {
            x.render(gl, args);
        }
    }

    //Moving the ship around or shooting
    fn kmove(&mut self, btn: &Button) {
        let updated_pos = match btn {
            &Button::Keyboard(Key::Up) => (0,-1),
            &Button::Keyboard(Key::Down) => (0, 1),
            &Button::Keyboard(Key::Left) => (-1, 0),
            &Button::Keyboard(Key::Right) => (1, 0),
            _ => (0,0),
        };

        //Only allow 5 shots on the screen at a time.
        if self.shots.len() < 5 {
            if btn == &Button::Keyboard(Key::Z) {
                let new_bullet = Bullet{pos_x: self.pos_x, pos_y: self.pos_y};
                self.shots.push(new_bullet);
            }
        }

        //Set bounds fo where the ship can move.
        if self.pos_x + updated_pos.0 < (WIDTH/20 -1)  && self.pos_y + updated_pos.1 < (HEIGHT/20 -3)
                && self.pos_x + updated_pos.0 >= 1 && self.pos_y + updated_pos.1 > 3 {
            self.pos_x += updated_pos.0;
            self.pos_y += updated_pos.1;
        }
    }

    //Update aspects of the ship, mainly for the shots.
    fn update(&mut self) {
        let mut index: usize = 0;
        let mut to_remove: Vec<usize> = Vec::new();
        for x in self.shots.iter_mut() {
            x.update();

            //If bullet goes above screen
            if x.get_pos().1 < 0 {
                to_remove.push(index);
            }
            else {
                index += 1;
            }
        }
        

        //Removing bullets that were found as out of bounds.
        for x in to_remove {
            self.shots.remove(x);
        }
        
    }
}

impl Bullet {
    //Draw the bullet on the screen
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let square = graphics::rectangle::square((self.pos_x * GRIDSIZE) as f64,
                     (self.pos_y * GRIDSIZE) as f64, GRIDSIZE as f64);

        gl.draw(args.viewport(), |c,gl| {
            let transform = c.transform;

            graphics::rectangle(graphics::color::hex("FFFF00"), square, transform, gl);
        });

    }
    
    //Moves the bullet up the screen
    fn update(&mut self) {
        self.pos_y -= 1;
    }

    //Able to give its position away to be used to check collision
    fn get_pos(& self) -> (i64, i64) {
        (self.pos_x, self.pos_y)
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    //get the window framework
    let mut window: GlutinWindow = WindowSettings::new(
        "galaga",
        [400,600],
        ).opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

    //Initialize the game
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        ship: Ship{pos_x: 10, pos_y: 26, shots: Vec::new()},
        enemies: Enemy{list: Vec::new()},
    };

    let mut events = Events::new(EventSettings::new()).ups(6);
    while let Some(e) = events.next(&mut window) {
        //Initial window render
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        //Update the game data and render everything
        if let Some(u) = e.update_args() {
            game.update();
        }

        //Listen for some key presses
        if let Some(key) = e.button_args() {
            if key.state == ButtonState::Press {
                game.pressed(&key.button);
            }
        }
    }
}
