// PieMyth @ github

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use rand::Rng;


static WIDTH:i64= 400;
static HEIGHT:i64 = 600;
static GRIDSIZE:i64 = 20;
static SPAWNRATE:u64 = 10;

//Starting out layout was used from the examples
//in the Piston Library and this video
//https://www.youtube.com/watch?v=HCwMb0KslX8
struct Game {
    gl: GlGraphics,
    ship: Ship,
    enemies: Enemy,
    ticks: u64,
    spawnrate: u64,
}

impl Game {
    //Gets screen set and renders ship.
    fn render(&mut self, arg: &RenderArgs) {
        use graphics;

        self.gl.draw(arg.viewport(), |_c, gl | {
            graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
        });

        self.ship.render(&mut self.gl, arg);
        self.enemies.render(&mut self.gl, arg);
    }

    //Update based on event args time
    fn update(&mut self) {
        self.ship.update();

        let mut spawns = (self.ticks as f64 / self.spawnrate as f64).sqrt()/2.0;
        if spawns < 1.0 {
            spawns = 1.0;
        }
        else {
            if spawns > 5.0 {
                spawns = 5.0;
            }
        }
        for _ in 0..spawns as u64{
            self.enemies.spawn();
        }
        
        
        self.ticks += 1;

        let hits = self.enemies.update(
            self.ship.current_pos(), 
            self.ship.get_shots());

        self.ship.collision(hits);
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


        let ship = graphics::rectangle::square(
            (self.pos_x * GRIDSIZE) as f64,
            (self.pos_y * GRIDSIZE) as f64, 
            GRIDSIZE as f64);

        gl.draw(args.viewport(), |c,gl| {
            let transform = c.transform;

            graphics::rectangle(
                graphics::color::WHITE,
                ship,
                transform,
                gl);
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
                let new_bullet = Bullet{
                    pos_x: self.pos_x,
                    pos_y: self.pos_y};

                self.shots.push(new_bullet);
            }
        }

        //Set bounds fo where the ship can move.
        if self.pos_x + updated_pos.0 < (WIDTH/GRIDSIZE -1)  
            && self.pos_y + updated_pos.1 < (HEIGHT/GRIDSIZE -3)
            && self.pos_x + updated_pos.0 >= 1 
            && self.pos_y + updated_pos.1 > 3 {

            self.pos_x += updated_pos.0;
            self.pos_y += updated_pos.1;
        }
    }

    fn update(&mut self) {
    //Update aspects of the ship, mainly for the shots.
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

        self.remove_shots(to_remove);
    }

    fn get_shots(&mut self) -> &mut Vec<Bullet> {
        &mut self.shots
    }

    fn current_pos(&mut self) -> (i64, i64) {
        (self.pos_x, self.pos_y)
    }

    fn remove_shots(&mut self, index: Vec<usize>) {
        //Removing bullets that were found as out of bounds.
        for x in index {
            if x < self.shots.len() {
                self.shots.remove(x);
            }
        }
    }

    fn collision(&mut self, hits: Vec<(i64,i64)>) {
        let mut index: usize = 0;
        let mut to_remove: Vec<usize> = Vec::new();
        let mut matched:bool = false;
        for x in self.shots.iter_mut() {
            let x = x.get_pos();
            for y in hits.iter() {
                if x.0 == y.0 && x.1 == y.1 {
                    to_remove.push(index);
                    matched = true;
                }
            }
            if !matched {
                index += 1;
            }
            else {
                matched = false;
            }
        }

        self.remove_shots(to_remove);
    }
}

impl Bullet {
    //Draw the bullet on the screen
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let square = graphics::rectangle::square(
                    (self.pos_x * GRIDSIZE) as f64,
                    (self.pos_y * GRIDSIZE) as f64,
                    GRIDSIZE as f64);

        gl.draw(args.viewport(), |c,gl| {
            let transform = c.transform;

            graphics::rectangle(
                graphics::color::hex("FFFF00"), 
                square, 
                transform, 
                gl);
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

impl Enemy {

    //renders the ship, also will render the shots when created.
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        for ships in self.list.iter_mut() {
            let new_ship = graphics::rectangle::square(
                        (ships.pos_x * GRIDSIZE) as f64,
                        (ships.pos_y * GRIDSIZE) as f64, 
                        GRIDSIZE as f64);

            gl.draw(args.viewport(), |c,gl| {
                let transform = c.transform;

                graphics::rectangle(
                    graphics::color::hex("ff0000"), 
                    new_ship, 
                    transform, 
                    gl);
            });
        }

    }

    fn spawn(&mut self) {
        let mut rng = rand::thread_rng();
        let pos_x: i64 = rng.gen_range(1, WIDTH/GRIDSIZE -1);
        let new_ship = Ship {pos_x, pos_y: -1, shots: Vec::new()};
        self.list.push(new_ship);
    }

    fn check_collision(&mut self, y: (i64, i64)) -> bool {
        let ships = self.current_pos();
        let mut hit: bool = false;
        let mut index = 0;
        for x in ships {
            if x.0 == y.0 && x.1 == y.1 {
                hit = true;
                self.list.remove(index);
            }
            else {
                index += 1;
            }

            if hit {
                break
            }
        } 

        hit
    }

    //Update aspects of the ships, check for collisions with shots or ship
    fn update(&mut self, 
              ship_pos: (i64, i64), 
              shot_pos: &mut Vec<Bullet>) -> Vec<(i64, i64)> {
        let mut hits: Vec<(i64, i64)> = Vec::new();

        for x in shot_pos.iter_mut() {
            let x = x.get_pos();
            if self.check_collision(x) {
                hits.push(x);
            }
        }

        if self.check_collision(ship_pos) {
            hits.push(ship_pos);
        }

        for x in self.list.iter_mut() {
            x.pos_y += 1;
        }

        for x in shot_pos.iter_mut() {
            let x = x.get_pos();
            if self.check_collision(x) {
                hits.push(x);
            }
        }

        if self.check_collision(ship_pos) {
            hits.push(ship_pos);
        }

        let mut index = 0;
        for x in self.current_pos().iter_mut() {
            //If enemy goes below
            if x.1 > (HEIGHT/GRIDSIZE -4) {
                self.list.remove(index);
            }
            else {
                index += 1;
            }
        }

        hits
    }
    
    fn current_pos(&mut self) -> Vec<(i64, i64)> {
        let mut current_pos: Vec<(i64, i64)> = Vec::new();
        for ships in self.list.iter_mut() {
            current_pos.push((ships.pos_x, ships.pos_y))
        }
        current_pos
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
        ticks: 0,
        spawnrate: SPAWNRATE,
    };

    let mut events = Events::new(EventSettings::new()).ups(6);
    while let Some(e) = events.next(&mut window) {
        //Initial window render
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        //Update the game data and render everything
        if let Some(_u) = e.update_args() {
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
