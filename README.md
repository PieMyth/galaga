# Galaga
This game was intended to be a simple version of the arcade games Galaga and Space Invaders with a twist of having some unbreakable astroids that need to be dodged. The goal of the game is to live as long as possible, while getting as many points as possible. There is a limit to only 5 shots on the screen at any time, so make your shots count!

### Tips
While shooting astroids may not break them, they still provide points when shot, try to focus efforts shooting those as early as possible.

When the rocks come in faster, try to have them spaced out so it is easy to avoid.

### Controls
The controls are pretty basic:
Arrow keys to go up, down, left, and right
'Z' to shoot
'R' to reset/restart

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

What things you need to install the software and how to install them. The dependencies are provided via Cargo.toml and mentioned later

```
rust "1.27.0"

```

### Installing

To build the game type:

```
cargo build
```

To run the game:

```
cargo run
```

When running it should look something like this:
(./game.png)

## Built With

* piston = "0.36.0" - Engine used to render
* piston2d-graphics = "0.26.0" - 2D graphics rendering engine
* pistoncore-glutin_window = "0.45.0" - Create a window
* piston2d-opengl_graphics = "0.52.0" - Ability to use openGL graphics
* rand = "0.5.4" - Random numbers, used for spawning enemy ships
* find_folder = "0.3.0" - Get assets from files into memory.

## Authors

* **William Haugen** - [PieMyth](https://github.com/PieMyth)

## License

This project is licensed under the BSD 2-Clause "Simplified" License - see the [LICENSE.md](LICENSE.md) file for details

## Acknowledgments

Special thanks to:
* YouCodeThings - [Making a Snake Game in Rust](https://www.youtube.com/watch?v=HCwMb0KslX8)
* PistonDevelopers - [Glyphs](https://github.com/PistonDevelopers/opengl_graphics/blob/master/examples/hello_world.rs)
