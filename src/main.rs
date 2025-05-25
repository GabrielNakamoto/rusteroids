use raylib::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;

const PI: f32 = 3.14159265359;
const WINDOW_D: (f32, f32) = (800.0, 580.0);
const DRAG: f32 = 1.5 * 1e-4;
const MAX_RADIUS: f32 = 50.0;
const MAX_SPEED: f32 = 200.0;

struct ShipSegment {
    dir: f32,
    speed: f32,
    angle: f32,
    ds: Vector2,
    p1: Vector2,
    p2: Vector2
}

struct Player {
    lives: i32,
    angle_r: f32, // radians
    ammo: i32,
    velocity: Vector2,
    pos: Vector2,
    exploding: bool,
    points: [Vector2; 4],
    segments: Vec<ShipSegment>
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
            pos: Vector2::zero(),
            exploding: false,
            points: [Vector2::new(-0.35, 0.0),
                     Vector2::new(0.0, 1.0),
                     Vector2::new(0.35, 0.0),
                     Vector2::new(-0.35, 0.0)],
            segments: Vec::new()
        },
        delta: 0.0,
        asteroids: Vec::new(),
    };

    for i in 0..state.player.points.len() {
        state.player.segments.push(ShipSegment {
            speed: 0.0,
            ds: Vector2::zero(),
            dir: 0.0,
            angle: 0.0,
            p1: state.player.points[i],
            p2: state.player.points[(i+1)%state.player.points.len()]
        });
    }

    generate_asteroids(&mut state, 10);
    main_loop(&mut state);
}

fn rng_min(rng : &mut ThreadRng, min : f32) -> f32 {
    let mut x = 0.0;
    while x < min {
        x = rng.gen::<f32>();
    }
    x
}

fn generate_explosion(state : &mut State) {
    let mut rng = rand::thread_rng();
    for segment in &mut state.player.segments {
        segment.speed = 5.0 * rng_min(&mut rng, 0.5);
        segment.dir = 2.0 * PI * rng.gen::<f32>();
        segment.angle = 2.0 * PI * rng.gen::<f32>();
    }
}

fn generate_asteroids(state : &mut State, n : i32) {
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        // TODO: fix this crap
        let mut x: f32;
        let mut y: f32;
        loop {
            x = (rng.gen::<f32>() * 1.5 * WINDOW_D.0) - WINDOW_D.0;
            y = (rng.gen::<f32>() * 1.5 * WINDOW_D.1) - WINDOW_D.1;

            if x < 0.0 || x > WINDOW_D.0 || y < 0.0 || y > WINDOW_D.1 {
                break;
            }
        }

        let speed = MAX_SPEED * rng_min(&mut rng, 0.25);
        let radius = MAX_RADIUS * rng_min(&mut rng, 0.25);
        let velocity = (Vector2::new(rng.gen::<f32>() * WINDOW_D.0, rng.gen::<f32>() * WINDOW_D.1) - Vector2::new(x,y)).normalized() * speed;

        let n_points = rng.gen_range(8..14);
        let mut points: Vec<Vector2> = Vec::new();

        // generate shape
        for i in 0..n_points {
            let magnitude = rng_min(&mut rng, 0.5);
            let theta = (PI * 2.0 / n_points as f32) * i as f32;
            let dir: Vector2 = Vector2::new(theta.cos(), theta.sin());

            points.push(dir * (radius * magnitude));
        }
        points.push(points[0]);

        state.asteroids.push(Asteroid {
            radius: radius,
            points: points,
            velocity: velocity,
            pos: Vector2::new(x, y)
        });
    }
}

fn main_loop(state : &mut State) {
    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;

        if state.asteroids.len() < 10 {
            generate_asteroids(state, 6);
        }

        if state.player.exploding && state.player.segments[0].speed < 1e-6 {
            generate_explosion(state);
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
    let mut ship_lines : Vec<Vector2> = Vec::new();

    let mut d = state.rl_handle.begin_drawing(&state.thread);

    d.clear_background(Color::BLACK);
    if state.player.exploding {
        for segment in &state.player.segments {
            let theta = segment.dir;
            // let displacement = state.player.pos + (Vector2::new(theta.cos(), theta.sin()) * segment.speed);
            let displacement = state.player.pos;

            let mut p1 = segment.p1.clone();
            p1.rotate(segment.angle);
            p1.scale(25.0);
            let mut p2 = segment.p2.clone();
            p2.rotate(segment.angle);
            p2.scale(25.0);

            p1 = to_draw_vector(p1 + displacement + segment.ds);
            p2 = to_draw_vector(p2 + displacement + segment.ds);
            d.draw_line_v(p1, p2, Color::WHITE);
        }
    } else {
        for point in &state.player.points {
            let mut p = point.clone();
            p.rotate(state.player.angle_r);
            p.scale(25.0);
            p += state.player.pos;
            ship_lines.push(to_draw_vector(p));
        }
    }

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
    if state.player.exploding {
        // update animation
        state.player.velocity.scale(1.0 - DRAG);

        for segment in &mut state.player.segments {
            segment.ds += Vector2::new(segment.dir.cos(), segment.dir.sin()) * segment.speed * state.delta as f32;
        }

        return
    }

    const SPEED: f64 = 0.25;

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

    for asteroid in &state.asteroids {
        // check for collisions
        let hit_x : bool = state.player.pos.x <= asteroid.pos.x + asteroid.radius
            && state.player.pos.x >= asteroid.pos.x - asteroid.radius;
        let hit_y : bool = state.player.pos.y <= asteroid.pos.y + asteroid.radius
            && state.player.pos.y >= asteroid.pos.y - asteroid.radius;
        if hit_x && hit_y {
            // state.player.pos = Vector2::zero();
            // state.player.velocity = Vector2::zero();

            state.player.exploding = true;
        }
    }
}
