use raylib::prelude::*;
use rand::Rng;

const PI: f32 = 3.14159265359;
const WINDOW_D: (i32, i32) = (800, 580);
const DRAG: f32 = 1.5 * 1e-4;

const MONITORED_KEYS: [KeyboardKey; 31] = [
    KeyboardKey::KEY_A,
    KeyboardKey::KEY_B,
    KeyboardKey::KEY_C,
    KeyboardKey::KEY_D,
    KeyboardKey::KEY_E,
    KeyboardKey::KEY_F,
    KeyboardKey::KEY_G,
    KeyboardKey::KEY_H,
    KeyboardKey::KEY_I,
    KeyboardKey::KEY_J,
    KeyboardKey::KEY_K,
    KeyboardKey::KEY_L,
    KeyboardKey::KEY_M,
    KeyboardKey::KEY_N,
    KeyboardKey::KEY_O,
    KeyboardKey::KEY_P,
    KeyboardKey::KEY_Q,
    KeyboardKey::KEY_R,
    KeyboardKey::KEY_S,
    KeyboardKey::KEY_T,
    KeyboardKey::KEY_U,
    KeyboardKey::KEY_V,
    KeyboardKey::KEY_W,
    KeyboardKey::KEY_X,
    KeyboardKey::KEY_Y,
    KeyboardKey::KEY_Z,
    KeyboardKey::KEY_RIGHT,
    KeyboardKey::KEY_LEFT,
    KeyboardKey::KEY_UP,
    KeyboardKey::KEY_DOWN,
    KeyboardKey::KEY_SPACE,
];

struct Player {
    lives: i32,
    angle_r: f32, // radians
    ammo: i32,
    velocity: Vector2,
    pos: Vector2
}

struct Asteroid {
    radius: f32,
    points: Vec<Vector2>,
    velocity: Vector2,
    pos: Vector2
}

struct State {
    rl_handle: RaylibHandle,
    thread: RaylibThread,
    player: Player,
    delta: f64,
    asteroids: Vec<Asteroid>,
    keys: [bool; 31] // alphabet + arrows + space
}

fn main() {
    let (rl, thread) = raylib::init()
        .size(WINDOW_D.0, WINDOW_D.1)
        .title("Rusteroids")
        .build();

    let mut state = State {
        rl_handle: rl,
        thread: thread,
        player: Player {
            lives: 3,
            angle_r: PI,
            ammo: 5,
            velocity: Vector2::zero(),
            pos: Vector2::zero()
        },
        delta: 0.0,
        asteroids: Vec::new(),
        keys: [false; 31]
    };

    generate_asteroids(&mut state, 1);
    main_loop(&mut state);
}

fn generate_asteroids(state : &mut State, n : i32) {
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let mut asteroid = Asteroid {
            radius: 35.0,
            points: Vec::new(),
            velocity: Vector2::zero(),
            pos: Vector2::new((WINDOW_D.0 / 2) as f32, (WINDOW_D.1 / 2) as f32)
        };
        let n_points = rng.gen_range(6..14);

        println!("Points: {}", n_points);
        for i in 0..n_points {
            let magnitude = loop {
                let x = rng.gen::<f32>();
                if x >= 0.25 {
                    break x;
                }
            };
            let theta = (PI * 2.0 / n_points as f32) * i as f32;
            let dir: Vector2 = Vector2::new(theta.cos(), theta.sin());

            asteroid.points.push(dir * (asteroid.radius * magnitude));
        }
        asteroid.points.push(asteroid.points[0]);

        state.asteroids.push(asteroid);
    }
}

fn main_loop(state : &mut State) {

    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;

        update_inputs(state);
        update_player(state);

        render_screen(state);

        prev_time = cur_time;
    }
}

fn update_inputs(state : &mut State) {
    let mut idx = 0;
    for key in MONITORED_KEYS {
        state.keys[idx] = state.rl_handle.is_key_down(key);
        idx += 1;
    }
}

fn render_screen(state : &mut State) {
    let ship_lines = serialize_player(state);

    let mut d = state.rl_handle.begin_drawing(&state.thread);

    d.clear_background(Color::BLACK);
    d.draw_line_strip(&ship_lines, Color::WHITE);

    for asteroid in &state.asteroids {
        let mut global_points : Vec<Vector2> = Vec::new();
        for point in &asteroid.points {
            global_points.push(*point + asteroid.pos);
        }
        d.draw_line_strip(&global_points, Color::WHITE);
    }
}

fn update_player(state : &mut State) {
    const SPEED: f64 = 0.25;
    // rotate at constant velocity?
    // unless moving

    let theta: f32 = state.player.angle_r - (PI * 3.0/2.0);
    let mut direction : Vector2 = Vector2::new(theta.cos(), theta.sin()); 

    if state.keys[26] { // right
        state.player.angle_r += 0.0012;
    }
    if state.keys[27] { // left
        state.player.angle_r -= 0.0012;
    }
    if state.keys[28] {
        state.player.velocity += direction * (SPEED * state.delta) as f32;
    }
    state.player.velocity.scale(1.0 - DRAG);

    state.player.pos += state.player.velocity;

    if state.player.pos.x >= (WINDOW_D.0 / 2) as f32 {
        state.player.pos.x -= WINDOW_D.0 as f32;
    } else if state.player.pos.x <= - (WINDOW_D.0 / 2) as f32 {
        state.player.pos.x += WINDOW_D.0 as f32;
    }

    if state.player.pos.y >= (WINDOW_D.1 / 2) as f32 {
        state.player.pos.y -= WINDOW_D.1 as f32;
    } else if state.player.pos.y <= - (WINDOW_D.1 / 2) as f32 {
        state.player.pos.y += WINDOW_D.1 as f32;
    }
}

fn serialize_player(state : &State) -> [Vector2; 4]{
    // center should be at (0, 0)
    let mut ship_lines: [Vector2; 4] = [Vector2::new(-0.35, 0.0),
                                        Vector2::new(0.0, 1.0),
                                        Vector2::new(0.35, 0.0),
                                        Vector2::new(-0.35, 0.0)];
    for i in &mut ship_lines {
        i.rotate(state.player.angle_r);
        i.scale(25.0);
        *i += Vector2::new((WINDOW_D.0/2) as f32, (WINDOW_D.1/2) as f32);
        *i += state.player.pos;
    }

    ship_lines
}
