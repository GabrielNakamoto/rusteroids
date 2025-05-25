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

struct Laser {
    dir: Vector2,
    pos: Vector2
}

struct Player {
    lives: i32,
    angle_r: f32, // radians
    velocity: Vector2,
    pos: Vector2,
    exploding: bool,
    explosion_delta: f32,
    points: [Vector2; 4],
    segments: Vec<ShipSegment>,
    laser_cooldown: f32,
    lasers: Vec<Laser>
}

#[derive(PartialEq)]
enum AsteroidSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge
}

struct Asteroid {
    size: AsteroidSize,
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
            angle_r: 0.0,
            velocity: Vector2::zero(),
            pos: Vector2::zero(),
            exploding: false,
            explosion_delta: 0.0,
            points: [Vector2::new(-0.35, 0.0),
                     Vector2::new(0.0, 1.0),
                     Vector2::new(0.35, 0.0),
                     Vector2::new(-0.35, 0.0)],
            segments: Vec::new(),
            laser_cooldown: 0.0,
            lasers: Vec::new()
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
        segment.speed = 15.0 * rng_min(&mut rng, 0.5);
        segment.dir = 2.0 * PI * rng.gen::<f32>();
        segment.angle = 2.0 * PI * rng.gen::<f32>();
    }
}

fn generate_asteroid(size : Option<AsteroidSize>, pos : Option<Vector2>) -> Asteroid {
    let mut rng = rand::thread_rng();

    // TODO: fix this crap
    let position : Vector2 = pos.unwrap_or_else(|| {
        let mut x: f32;
        let mut y: f32;
        loop {
            x = (rng.gen::<f32>() * 1.5 * WINDOW_D.0) - WINDOW_D.0;
            y = (rng.gen::<f32>() * 1.5 * WINDOW_D.1) - WINDOW_D.1;

            if x > WINDOW_D.0/2.0 || x < -WINDOW_D.0/2.0 || y < -WINDOW_D.1/2.0 || y > WINDOW_D.1/2.0 {
                break Vector2::new(x, y);
            }
        }
    });

    let size : AsteroidSize = size.unwrap_or_else(|| match rng.gen_range(1..5) {
        1 => AsteroidSize::Tiny,
        2 => AsteroidSize::Small,
        3 => AsteroidSize::Medium,
        4 => AsteroidSize::Large,
        5 => AsteroidSize::Huge,
        _ => AsteroidSize::Medium
    });

    let speed = MAX_SPEED * rng_min(&mut rng, 0.25);
    let radius = match size {
        AsteroidSize::Tiny => 12.0,
        AsteroidSize::Small => 20.0,
        AsteroidSize::Medium => 35.0,
        AsteroidSize::Large => 40.0,
        AsteroidSize::Huge => 50.0
    };
    let velocity = (Vector2::new(rng.gen::<f32>() * WINDOW_D.0, rng.gen::<f32>() * WINDOW_D.1) - position).normalized() * speed;

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

    Asteroid {
        size: size,
        radius: radius,
        points: points,
        velocity: velocity,
        pos: position
    }
}

fn main_loop(state : &mut State) {
    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;

        if state.asteroids.len() < 10 {
            for _ in 0..10-state.asteroids.len() {
                state.asteroids.push(generate_asteroid(None, None));
            }
        }

        if state.player.exploding && state.player.explosion_delta < 1e-6 {
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
    // big ones break up into smaller ones

    let mut stale: Vec<usize> = Vec::new();
    let mut animate: Vec<usize> = Vec::new();
    let mut new: Vec<Asteroid> = Vec::new();

    let mut idx = 0;
    for asteroid in &mut state.asteroids {
        asteroid.pos += asteroid.velocity * (state.delta as f32);

        if (!in_bounds(asteroid.pos)) && asteroid.velocity.dot(-asteroid.pos) < 0.0 {
            stale.push(idx);
            continue;
        }

        for laser in &state.player.lasers {
            if asteroid.pos.distance_to(laser.pos) > asteroid.radius {
                continue;
            }
            if asteroid.size == AsteroidSize::Tiny {
                animate.push(idx);
            } else {
                stale.push(idx);
                for _ in 0..2 {
                    new.push(generate_asteroid(Some(match asteroid.size {
                        _ => AsteroidSize::Tiny,
                        AsteroidSize::Small => AsteroidSize::Tiny,
                        AsteroidSize::Medium => AsteroidSize::Small,
                        AsteroidSize::Large => AsteroidSize::Medium,
                        AsteroidSize::Huge => AsteroidSize::Large,
                    }), Some(asteroid.pos)));
                }
            }
        }
        
        idx += 1;
    }

    let mut x = 0;
    for i in stale {
        state.asteroids.remove(i-x);
        x+=1;
    }
    for a in new {
        state.asteroids.push(a);
    }
}

fn render_screen(state : &mut State) {
    let mut ship_lines : Vec<Vector2> = Vec::new();

    let mut d = state.rl_handle.begin_drawing(&state.thread);

    d.clear_background(Color::BLACK);
    if state.player.exploding {
        for segment in &state.player.segments {
            let theta = segment.dir;
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

    for laser in &state.player.lasers {
        d.draw_circle_v(to_draw_vector(laser.pos), 1.5, Color::WHITE);
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
    state.player.laser_cooldown -= state.delta as f32;

    for laser in &mut state.player.lasers {
        laser.pos += laser.dir * 250.0 * state.delta as f32;
    }

    if state.player.explosion_delta > 2.0 {
        state.player.pos = Vector2::zero();
        state.player.angle_r = 0.0;
        state.player.velocity = Vector2::zero();
        state.player.explosion_delta = 0.0;
        state.player.exploding = false;
    }

    if state.player.exploding {
        state.player.explosion_delta += state.delta as f32;
        state.player.velocity.scale(1.0 - DRAG);

        for segment in &mut state.player.segments {
            segment.ds += Vector2::new(segment.dir.cos(), segment.dir.sin()).normalized()
                * segment.speed
                * state.delta as f32;
        }
        return
    }

    const SPEED: f64 = 0.25;

    let theta: f32 = state.player.angle_r - (PI * 3.0/2.0);
    let direction : Vector2 = Vector2::new(theta.cos(), theta.sin()).normalized(); 

    if state.rl_handle.is_key_down(KeyboardKey::KEY_RIGHT) { // right
        state.player.angle_r -= 0.0012;
    }
    if state.rl_handle.is_key_down(KeyboardKey::KEY_LEFT) {
        state.player.angle_r += 0.0012;
    }
    if state.rl_handle.is_key_down(KeyboardKey::KEY_UP) {
        state.player.velocity += direction * (SPEED * state.delta) as f32;
    }

    if state.rl_handle.is_key_down(KeyboardKey::KEY_SPACE) && state.player.laser_cooldown < 1e-6 {
        state.player.lasers.push(Laser {
            dir: direction,
            pos: state.player.pos
        });
        state.player.laser_cooldown = 0.2;
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
        if state.player.pos.distance_to(asteroid.pos) < asteroid.radius {
            state.player.exploding = true;
            state.player.explosion_delta = 0.0;
        }
    }
}
