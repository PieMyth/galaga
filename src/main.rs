// Copyright © 2018 William Haugen - Piemyth
// [This work is licensed under the "BSD 2-Clause License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate find_folder;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow;
use graphics::{clear, text, Image, Transformed};
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, Texture};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;

static WIDTH: i64 = 400;
static HEIGHT: i64 = 600;
static GRIDSIZE: i64 = 20;
static SPAWNRATE: u64 = 10;
static POINTS: u64 = 25;

//Starting out layout was used from the examples
//in the Piston Library and this video
//https://www.youtube.com/watch?v=HCwMb0KslX8
//Glyphs were pulled from the piston examples on github:
//https://github.com/PistonDevelopers/opengl_graphics/blob/master/examples/hello_world.rs
struct Game {
    gl: GlGraphics,
    ship: Ship,
    enemies: Enemy,
    ticks: u64,
    spawnrate: u64,
    score: u64,
}

impl Game {
    //Gets screen and renders ship all the sprites on the screen.
    fn render(&mut self, arg: &RenderArgs, player: &Texture, fighter: &Texture, rock: &Texture) {
        self.ship.render(&mut self.gl, arg, player);
        self.enemies.render(&mut self.gl, arg, fighter, rock);
    }

    fn score(&mut self) -> String {
        //Get the score to be rendered down in main.
        let mut score = "Score: ".to_string();
        score.push_str(&self.score.to_string());

        score
    }

    //Update based on event args time
    fn update(&mut self) -> bool {
        //Spawning system for enemy ships.
        //Will span more as time goes on to a limit of 5 ships per tick and
        //One astroid every 3 and 7 game ticks.
        let mut spawns =
            (self.ticks as f64 / self.spawnrate as f64).sqrt() / (SPAWNRATE * 10) as f64;

        if spawns < 1.0 {
            spawns = 1.0;
        }
        if spawns > 5.0 {
            spawns = 5.0;
        }
        for _ in 0..spawns as u64 {
            self.enemies.spawnship();
        }

        if (self.ticks % 3 == 0 && self.ticks > 240) || (self.ticks % 7 == 0 && self.ticks > 60) {
            self.enemies.spawnrock(self.ship.current_pos().0);
        }

        self.ticks += 1;

        self.ship.update(false);
        let hits = self.enemies
            .update(self.ship.current_pos(), self.ship.get_shots(), false);

        let result = self.ship.collision(hits);
        self.score += result.1;

        if result.0 {
            return true;
        }
        false
    }

    //Update Ship's movement or shoot depending on input
    //Returns a tuple left value indicates a reset, right value indicates game over.
    fn pressed(&mut self, btn: &Button, game_over: bool) -> (bool, bool) {
        if btn == &Button::Keyboard(Key::R) {
            return (true, false);
        }
        self.ship.kmove(btn);
        let hits = self.enemies
            .update(self.ship.current_pos(), self.ship.get_shots(), true);

        let result = self.ship.collision(hits);
        if !game_over {
            self.score += result.1;
        }
        (false, result.0)
    }

    //Restarts all sprites to default position or clear them
    //Set all game values to 0.
    fn restart(&mut self) {
        self.ship.restart();
        self.enemies.restart();
        self.ticks = 0;
        self.score = 0;
    }
}

struct Ship {
    pos_x: i64,
    pos_y: i64,
    shots: Vec<Bullet>,
}

struct Enemy {
    list: Vec<Ship>,
    rocks: Vec<Ship>,
}

struct Bullet {
    pos_x: i64,
    pos_y: i64,
}

impl Ship {
    //renders the ship, also will render the shots when created.
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, texture: &Texture) {
        use graphics;

        let ship = Image::new().rect(graphics::rectangle::square(
            (self.pos_x * GRIDSIZE) as f64,
            (self.pos_y * GRIDSIZE) as f64,
            GRIDSIZE as f64,
        ));

        gl.draw(args.viewport(), |c, gl| {
            //Draw the image with the texture
            let draw_state = graphics::DrawState::new_alpha();
            ship.draw(texture, &draw_state, c.transform, gl)
        });

        for mut x in (&mut self.shots).iter_mut() {
            x.render(gl, args);
        }
    }

    //Moving the ship around or shooting
    fn kmove(&mut self, btn: &Button) {
        let updated_pos = match btn {
            Button::Keyboard(Key::Up) => (0, -1),
            Button::Keyboard(Key::Down) => (0, 1),
            Button::Keyboard(Key::Left) => (-1, 0),
            Button::Keyboard(Key::Right) => (1, 0),
            _ => (0, 0),
        };

        //Only allow 5 shots on the screen at a time.
        if self.shots.len() < 5 && btn == &Button::Keyboard(Key::Z) {
            let new_bullet = Bullet {
                pos_x: self.pos_x,
                pos_y: self.pos_y - 1,
            };

            self.shots.push(new_bullet);
        }

        //Set bounds fo where the ship can move.
        if self.pos_x + updated_pos.0 < (WIDTH / GRIDSIZE - 1)
            && self.pos_y + updated_pos.1 < (HEIGHT / GRIDSIZE - 3)
            && self.pos_x + updated_pos.0 >= 1
            && self.pos_y + updated_pos.1 > 3
        {
            self.pos_x += updated_pos.0;
            self.pos_y += updated_pos.1;
        }
    }

    //Update with gametick. Moved bool is to indicate if the player moved,
    //or it was wiht the regular update of a gametick.
    fn update(&mut self, moved: bool) {
        //Update aspects of the ship, mainly for the shots.
        let mut index: usize = 0;
        let mut to_remove: Vec<usize> = Vec::new();
        if !moved {
            for x in (&mut self.shots).iter_mut() {
                x.update();

                //If bullet goes above screen
                if x.get_pos().1 < 0 {
                    to_remove.push(index);
                } else {
                    index += 1;
                }
            }
        }

        self.remove_shots(to_remove);
    }

    //Gets the shots for checking position and possibly
    //removing them.
    fn get_shots(&mut self) -> &mut Vec<Bullet> {
        &mut self.shots
    }

    //Give current posotion of ship.
    fn current_pos(&mut self) -> (i64, i64) {
        (self.pos_x, self.pos_y)
    }

    fn remove_shots(&mut self, index: Vec<usize>) {
        //Removing bullets that were found as out of bounds or hit something.
        for x in index {
            if x < self.shots.len() {
                self.shots.remove(x);
            }
        }
    }

    fn collision(&mut self, hits: Vec<(i64, i64)>) -> (bool, u64) {
        let mut index: usize = 0;
        let mut score = 0;
        let mut to_remove: Vec<usize> = Vec::new();
        let mut matched: bool = false;

        //Check collision of the shots
        for x in (&mut self.shots).iter_mut() {
            let x = x.get_pos();
            for y in (&hits).iter() {
                //Check for coordinates to see if match.
                //If there was a match, add points to score to be
                //Returned back then displayed.
                if x.0 == y.0 && x.1 == y.1 {
                    to_remove.push(index);
                    score += POINTS;
                    matched = true;
                }
            }
            if !matched {
                index += 1;
            } else {
                matched = false;
            }
        }

        //Remove all shots that have a collision
        self.remove_shots(to_remove);

        //Reset matched for ship.
        matched = false;
        let ship_pos = self.current_pos();
        //Check to see if player ship was hit or not.
        for x in hits {
            if ship_pos.0 == x.0 && ship_pos.1 == x.1 {
                matched = true;
            }

            if matched {
                break;
            }
        }

        (matched, score)
    }

    //Clear the shots and reset ship to default position.
    fn restart(&mut self) {
        self.shots.clear();
        self.pos_x = 10;
        self.pos_y = 26;
    }
}

impl Bullet {
    //Draw the bullet on the screen
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let square = graphics::rectangle::square(
            (self.pos_x * GRIDSIZE + GRIDSIZE / 4) as f64,
            (self.pos_y * GRIDSIZE) as f64,
            (GRIDSIZE / 2) as f64,
        );

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(
                //Yellow in hex color
                graphics::color::hex("FFFF00"),
                square,
                transform,
                gl,
            );
        });
    }

    //Moves the bullet up the screen
    fn update(&mut self) {
        self.pos_y -= 1;
    }

    //Give shot's position in the form of a tuple.
    fn get_pos(&self) -> (i64, i64) {
        (self.pos_x, self.pos_y)
    }
}

impl Enemy {
    //renders the ship, also will render the shots when created.
    fn render(
        &mut self,
        gl: &mut GlGraphics,
        args: &RenderArgs,
        shiptexture: &Texture,
        rocktexture: &Texture,
    ) {
        use graphics;

        //Render all enemy ships in positions
        for ships in (&mut self.list).iter_mut() {
            let new_ship = Image::new().rect(graphics::rectangle::square(
                (ships.pos_x * GRIDSIZE) as f64,
                (ships.pos_y * GRIDSIZE) as f64,
                GRIDSIZE as f64,
            ));

            gl.draw(args.viewport(), |c, gl| {
                //Draw the image with the texture
                let draw_state = graphics::DrawState::new_alpha();
                new_ship.draw(shiptexture, &draw_state, c.transform, gl)
            });
        }

        //Render all rocks in their positions.
        for rock in (&mut self.rocks).iter_mut() {
            let new_rock = Image::new().rect(graphics::rectangle::square(
                (rock.pos_x * GRIDSIZE) as f64,
                (rock.pos_y * GRIDSIZE) as f64,
                GRIDSIZE as f64,
            ));

            gl.draw(args.viewport(), |c, gl| {
                //Draw the image with the texture
                let draw_state = graphics::DrawState::new_alpha();
                new_rock.draw(rocktexture, &draw_state, c.transform, gl)
            });
        }
    }

    //Spawn ship's randomly on the x position.
    fn spawnship(&mut self) {
        let mut rng = rand::thread_rng();
        let pos_x: i64 = rng.gen_range(1, WIDTH / GRIDSIZE - 1);
        let new_ship = Ship {
            pos_x,
            pos_y: -1,
            shots: Vec::new(),
        };
        self.list.push(new_ship);
    }

    //Creats a rock, set x position to the player ship's
    //current x position.
    fn spawnrock(&mut self, pos_x: i64) {
        let new_ship = Ship {
            pos_x,
            pos_y: -1,
            shots: Vec::new(),
        };
        self.rocks.push(new_ship);
    }

    //Check collision for enemy ships
    fn ship_collision(&mut self, y: (i64, i64)) -> bool {
        let mut hit: bool = false;
        let mut index = 0;
        for x in self.current_pos() {
            //First checks if the x coordinate is the same.
            //Then checks to see if either the y coordinates match
            //or they are one apart.
            if x.0 == y.0 {
                if x.1 == y.1 || x.1 == y.1 + 1 {
                    //Remove if hit and set hit to true.
                    hit = true;
                    self.list.remove(index);
                } else {
                    //Increment if there wasn't a revmoval of list.
                    index += 1;
                }
            } else {
                //Another increment since didn't make it as far.
                index += 1;
            }

            //If hit, no need to continue,.
            if hit {
                break;
            }
        }

        //Return rresult if ship was hit or not.
        hit
    }

    //Check collision of rocks.
    fn rock_collision(&mut self, y: (i64, i64)) -> bool {
        let mut hit: bool = false;
        for x in (&mut self.rocks).iter_mut() {
            let mut x = x.current_pos();
            //Needs to check if it is on position or below one.
            //Can pass through if it doesn't check below one.
            if x.0 == y.0 && (x.1 == y.1 || x.1 == y.1 + 1) {
                //Don't remove since rocks to get destroyed.
                hit = true;
            }

            //If there was a hit on that rock, exit out of loop
            //No need to continue on.
            if hit {
                break;
            }
        }

        //Return result if rock was hit or not.
        hit
    }

    //Update aspects of the ships, check for collisions with shots or ship
    fn update(
        &mut self,
        ship_pos: (i64, i64),
        shot_pos: &mut Vec<Bullet>,
        movement: bool,
    ) -> Vec<(i64, i64)> {
        let mut hits: Vec<(i64, i64)> = Vec::new();
        let mut prev_hits: Vec<(i64, i64)> = Vec::new();
        let mut prev: bool = false;

        //Checks the positions of all the shots
        //Removes a the ship if hit and adds to a hit list
        //hit list will get passed back to player ship
        //and have collisions checked for there.
        for x in shot_pos.iter_mut() {
            let x = x.get_pos();
            for y in (&mut prev_hits).iter_mut() {
                if x.0 == y.0 && x.1 == y.1 {
                    prev = true;
                }
            }

            //If either ship or rock were hit, push the positions on the hit
            //list for player ship to remove.
            if !prev && (self.ship_collision(x) || self.rock_collision(x)) {
                hits.push(x);
                prev_hits.push(x);
                prev = false;
            }
        }

        //Checks collision with player ship.
        if self.ship_collision(ship_pos) || self.rock_collision(ship_pos) {
            hits.push(ship_pos);
        }

        //If the update wasn't for a player input, move the rocks and ships.
        if !movement {
            for x in (&mut self.list).iter_mut() {
                x.pos_y += 1;
            }
            for x in (&mut self.rocks).iter_mut() {
                x.pos_y += 1;
            }

            //Check collision against player ship again after the move.
            if self.ship_collision(ship_pos) || self.rock_collision(ship_pos) {
                hits.push(ship_pos);
            }
        }

        //Remove any enemies or rocks that have gone below where the player can go.
        let mut index = 0;
        for x in self.current_pos() {
            //If enemy goes below
            if x.1 > (HEIGHT / GRIDSIZE - 4) {
                self.list.remove(index);
            } else {
                //Only need to increment index if a ship wasn't removed.
                index += 1;
            }
        }
        //Reset index for use with rock positions.
        index = 0;
        for x in self.current_rock_pos() {
            //If enemy goes below
            if x.1 > (HEIGHT / GRIDSIZE - 4) {
                self.rocks.remove(index);
            } else {
                index += 1;
            }
        }

        //Return the list of all hits made on rocks and enemy ships.
        hits
    }

    //Grabs the positions of all the ships.
    fn current_pos(&mut self) -> Vec<(i64, i64)> {
        let mut current_pos: Vec<(i64, i64)> = Vec::new();
        for ships in (&self.list).iter() {
            current_pos.push((ships.pos_x, ships.pos_y))
        }
        current_pos
    }

    //Grabs the positions of all rocks.
    fn current_rock_pos(&mut self) -> Vec<(i64, i64)> {
        let mut current_pos: Vec<(i64, i64)> = Vec::new();
        for rock in (&self.rocks).iter() {
            current_pos.push((rock.pos_x, rock.pos_y))
        }
        current_pos
    }

    //Clears all enemies and rocks on screen
    fn restart(&mut self) {
        self.list.clear();
        self.rocks.clear();
    }
}

fn main() {
    //If there's an error with opengl, change the version
    //and uncomment the .opengl() argument for the window
    //below when window is created.
    let opengl = OpenGL::V3_2;

    //get the window framework
    let mut window: GlutinWindow = WindowSettings::new(
        "galaga",
        [400,600],
        )
        //.opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    //Initialize the game
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        ship: Ship {
            pos_x: 10,
            pos_y: 26,
            shots: Vec::new(),
        },
        enemies: Enemy {
            list: Vec::new(),
            rocks: Vec::new(),
        },
        ticks: 0,
        spawnrate: SPAWNRATE,
        score: 0,
    };

    //Load all of the images and fonts from assets folder.
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let background = assets.join("background.png");
    let ship = assets.join("ship.png");
    let fighter = assets.join("enemy.png");
    let rock = assets.join("rock.png");
    let font = assets.join("FiraSans-Regular.ttf");
    //Create the image object and attach a square Rectangle object inside.
    //Used for background.
    let image = Image::new().rect(graphics::rectangle::square(0.0, 0.0, HEIGHT as f64));

    //A texture to use with the image, using the paths
    //created above from assets
    let background_texture =
        Texture::from_path(background, &opengl_graphics::TextureSettings::new()).unwrap();
    let ship_texture = Texture::from_path(ship, &opengl_graphics::TextureSettings::new()).unwrap();
    let fighter_texture =
        Texture::from_path(fighter, &opengl_graphics::TextureSettings::new()).unwrap();
    let rock_texture = Texture::from_path(rock, &opengl_graphics::TextureSettings::new()).unwrap();

    //Convert font into a glyphcache
    let mut glyphs = GlyphCache::new(font, (), opengl_graphics::TextureSettings::new()).unwrap();

    //ups is the number of times it will run per second.
    let mut events = Events::new(EventSettings::new()).ups(6);
    let mut game_over = false;
    let mut reset = false;
    while let Some(e) = events.next(&mut window) {
        let score = game.score();
        //Initial window render
        if let Some(r) = e.render_args() {
            game.gl.draw(r.viewport(), |c, gl| {
                //Clear the screen
                clear([0.0, 0.0, 0.0, 1.0], gl);
                let draw_state = graphics::DrawState::new_alpha();

                //Render the background image
                image.draw(&background_texture, &draw_state, c.transform, gl);

                //Position and render the score on the screen
                let transform = c.transform.trans(1.0, (HEIGHT) as f64);
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
                    .draw(&score, &mut glyphs, &c.draw_state, transform, gl)
                    .unwrap();
            });

            game.render(&r, &ship_texture, &fighter_texture, &rock_texture);

            if game_over {
                game.gl.draw(r.viewport(), |c, gl| {
                    //Position the text in the location.
                    let transform = c.transform
                        .trans((WIDTH / 4 + 15) as f64, (HEIGHT / 2 - 20) as f64);

                    text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
                        .draw("GAME OVER", &mut glyphs, &c.draw_state, transform, gl)
                        .unwrap();

                    //Relocate where text is to be rendered
                    let transform = c.transform
                        .trans((WIDTH / 4 - 5) as f64, (HEIGHT / 2 + 10) as f64);

                    text::Text::new_color([1.0, 1.0, 1.0, 1.0], 24)
                        .draw(
                            "Press 'R' To Restart",
                            &mut glyphs,
                            &c.draw_state,
                            transform,
                            gl,
                        )
                        .unwrap();
                });
            }
        }

        //Update the game data and render everything
        //if the game over conditions haven't occured.
        if !game_over {
            if let Some(_u) = e.update_args() {
                game_over = game.update();
            }
        }

        //Listen for some key presses
        if let Some(key) = e.button_args() {
            if key.state == ButtonState::Press {
                let result = game.pressed(&key.button, game_over);
                //If a user pushes r, reset the game
                if result.0 {
                    reset = true;
                }
                //If a user collides with a block, game over
                if result.1 {
                    game_over = true;
                }
            }
        }

        //If a user decided to restart the game.
        //Call restart to reset everything about the agme.
        if reset {
            game.restart();
            reset = false;
            game_over = false;
        }
    }
}
