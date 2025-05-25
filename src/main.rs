use raylib::prelude::*;
use rand::Rng;

const PI: f32 = 3.14159265359;
const WINDOW_D: (f32, f32) = (800.0, 580.0);
const DRAG: f32 = 1.5 * 1e-4;
const MAX_RADIUS: f32 = 50.0;

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
        .size(WINDOW_D.0 as i32, WINDOW_D.1 as i32)
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

    generate_asteroids(&mut state, 10);
    main_loop(&mut state);
}

fn generate_asteroids(state : &mut State, n : i32) {
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let radius = MAX_RADIUS * loop {
            let x = rng.gen::<f32>();
            if x >= 0.25 {
                break x;
            }
        };

        let mut x: f32;
        let mut y: f32;
        loop {
            // how do I let it be negative
            x = (rng.gen::<f32>() * 1.5 * WINDOW_D.0) - WINDOW_D.0;
            y = (rng.gen::<f32>() * 1.5 * WINDOW_D.1) - WINDOW_D.1;

            if x < 0.0 || x > WINDOW_D.0 || y < 0.0 || y > WINDOW_D.1 {
                break;
            }
        }

        const max_speed: f32 = 200.0;
        let speed = max_speed * loop {
            let x = rng.gen::<f32>();
            if x >= 0.25 {
                break x;
            }
        };

        let velocity = (Vector2::new(rng.gen::<f32>() * WINDOW_D.0, rng.gen::<f32>() * WINDOW_D.1) - Vector2::new(x,y)).normalized() * speed;

        let mut asteroid = Asteroid {
            radius: radius,
            points: Vec::new(),
            velocity: velocity,
            pos: Vector2::new(x, y),
        };

        let n_points = rng.gen_range(8..14);

        for i in 0..n_points {
            let magnitude = loop {
                let x = rng.gen::<f32>();
                if x >= 0.5 {
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

// make everything use (0, 0) as center coordinates
// convert to draw coordinates in render function (left top corner origin)
fn main_loop(state : &mut State) {

    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;

        // remove asteroids once off screen
        // then regenerate once less then...
        if state.asteroids.len() < 10 {
            generate_asteroids(state, 6);
        }

        update_player(state);
        update_asteroids(state);

        render_screen(state);


        prev_time = cur_time;
    }
}

fn to_draw_vector(point : Vector2) -> Vector2 {
    Vector2::new(point.x+WINDOW_D.0/2.0, -point.y+WINDOW_D.1/2.0)
}

fn in_bounds(point : Vector2) -> bool {
    point.x >= -WINDOW_D.0/2.0 && point.x <= WINDOW_D.0/2.0 && point.y >= -WINDOW_D.1/2.0 && point.y <= WINDOW_D.1/2.0
}

fn update_asteroids(state : &mut State) {
    let mut stale: Vec<usize> = Vec::new();
    let mut idx = 0;
    for asteroid in &mut state.asteroids {
        asteroid.pos += asteroid.velocity * (state.delta as f32);

        // let center_vector : Vector2 = -Vector2::new(asteroid.pos.x-WINDOW_D.0/2.0, WINDOW_D.1/2.0 - asteroid.pos.y);
        let center_vector = -asteroid.pos;
        if (!in_bounds(asteroid.pos)) && asteroid.velocity.dot(center_vector) < 0.0 {
            println!("Removing asteroid at index: {}", idx);
            stale.push(idx);
        }
        
        idx += 1;
    }

    let mut x = 0;
    for i in stale {
        state.asteroids.remove(i-x);
        x+=1;
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
            global_points.push(to_draw_vector(*point + asteroid.pos));
        }
        d.draw_line_strip(&global_points, Color::WHITE);
    }
}

fn update_player(state : &mut State) {
    const SPEED: f64 = 0.25;
    // rotate at constant velocity?
    // unless moving

    let theta: f32 = state.player.angle_r - (PI * 3.0/2.0);
    let direction : Vector2 = Vector2::new(theta.cos(), theta.sin()); 

    if state.rl_handle.is_key_down(KeyboardKey::KEY_RIGHT) { // right
        state.player.angle_r -= 0.0012;
    }
    if state.rl_handle.is_key_down(KeyboardKey::KEY_LEFT) {
        state.player.angle_r += 0.0012;
    }
    if state.rl_handle.is_key_down(KeyboardKey::KEY_UP) {
        state.player.velocity += direction * (SPEED * state.delta) as f32;
    }
    state.player.velocity.scale(1.0 - DRAG);

    state.player.pos += state.player.velocity;

    if state.player.pos.x >= WINDOW_D.0 / 2.0 {
        state.player.pos.x -= WINDOW_D.0;
    } else if state.player.pos.x <= -WINDOW_D.0 / 2.0 {
        state.player.pos.x += WINDOW_D.0;
    }

    if state.player.pos.y >= WINDOW_D.1 / 2.0 {
        state.player.pos.y -= WINDOW_D.1;
    } else if state.player.pos.y <= -WINDOW_D.1 / 2.0 {
        state.player.pos.y += WINDOW_D.1;
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
        // *i += Vector2::new(WINDOW_D.0/2.0, WINDOW_D.1/2.0);
        *i += state.player.pos;
        *i = to_draw_vector(*i);
    }

    ship_lines
}
