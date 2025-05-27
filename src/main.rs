use raylib::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const SCALE: f32 = 1.65;

const PI: f32 = 3.14159265359;
const WINDOW_D: (f32, f32) = (800.0 * SCALE, 580.0 * SCALE);
const DRAG: f32 = 1.5 * 1e-4;
const MAX_SPEED: f32 = 200.0;
const THICKNESS: f32 = 1.25 * SCALE;
const SPEED: f32 = 0.25 * SCALE;
const ROT_SPEED: f32 = 0.0018;
const SHIP_SCALE: f32 = 25.0 * SCALE;

const MAX_PARTICLE_DIST: f32 = 20.0 * SCALE;

const SEGMENT_SPEED: f32 = 20.0 * SCALE;
const PARTICLE_SPEED: f32 = 35.0 * SCALE;
const LASER_SPEED: f32 = 250.0 * SCALE;

const LASER_RADIUS: f32 = 1.5 * SCALE;
const PARTICLE_RADIUS: f32 = 1.0 * SCALE;

const LIFE_SIZE: f32 = 25.0 * SCALE;
const SCORE_SIZE: i32 = 34 * SCALE as i32;

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
    pos: Vector2,
    hit: bool
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

impl Distribution<AsteroidSize> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng : &mut R) -> AsteroidSize {
        match rng.gen_range(0..=4) {
            0 => AsteroidSize::Tiny,
            1 => AsteroidSize::Small,
            2 => AsteroidSize::Medium,
            3 => AsteroidSize::Large,
            _ => AsteroidSize::Huge
        }
    }
}

impl AsteroidSize {
    fn radius(&self) -> f32 {
        match self {
            Self::Tiny => 12.0 * SCALE,
            Self::Small => 20.0 * SCALE,
            Self::Medium => 35.0 * SCALE,
            Self::Large => 40.0 * SCALE,
            _ => 50.0 * SCALE,
        }
    }

    fn score(&self) -> i32 {
        match self {
            Self::Tiny => 100,
            Self::Small => 70,
            Self::Medium => 50,
            Self::Large => 30,
            _ => 10,
        }
    }

    fn split_size(&self) -> AsteroidSize {
        match self {
            Self::Small => Self::Tiny,
            Self::Medium => Self::Small,
            Self::Large => Self::Medium,
            Self::Huge => Self::Large,
            _ => Self::Tiny,
        }
    }
}

struct Particle {
    pos: Vector2,
    dir: Vector2,
    speed: f32,
    lifetime: f32
}

struct Asteroid {
    size: AsteroidSize,
    radius: f32,
    points: Vec<Vector2>,
    velocity: Vector2,
    pos: Vector2,
    particles: Vec<Particle>,
    destroyed: bool,
    stale : bool
}

struct State {
    rl_handle: RaylibHandle,
    audio: Option<RaylibAudio>,
    thread: RaylibThread,
    player: Player,
    delta: f32,
    asteroids: Vec<Asteroid>,
    score: i32,
}

// force the stale vector move
fn clean_vec<T>(v : &mut Vec<T>, mut stale : Vec<usize>) {
    // descending order => index shifting doesn't matter
    stale.reverse();
    for i in stale {
        // swap_remove is O(1), doesn't affect future removals because
        // it swaps with the back
        v.swap_remove(i);
    }
}

fn to_draw_vector(point : Vector2) -> Vector2 {
    Vector2::new(point.x+WINDOW_D.0/2.0, -point.y+WINDOW_D.1/2.0)
}

fn in_bounds(point : Vector2) -> bool {
    point.x >= -WINDOW_D.0/2.0
    && point.x <= WINDOW_D.0/2.0
    && point.y >= -WINDOW_D.1/2.0
    && point.y <= WINDOW_D.1/2.0
}

fn rng_min(min : f32) -> f32 {
    let mut x = 0.0;
    while x < min {
        x = rand::random::<f32>();
    }
    x
}

fn main() {
    let (rl, thread) = raylib::init()
        .size(WINDOW_D.0 as i32, WINDOW_D.1 as i32)
        .title("Rusteroids")
        .build();

    let mut audio : Option<RaylibAudio> = None;
    match RaylibAudio::init_audio_device() {
        Ok(a) => audio = Some(a),
        Err(e) => println!("Error initializing audio: {}", e),
    };

    if audio.is_some() {
        println!("Got audio device");
    }

    let mut state = State {
        rl_handle: rl,
        audio: audio,
        thread: thread,
        player: Player::new(),
        delta: 0.0,
        asteroids: Vec::new(),
        score: 0,
    };

    game_loop(state); // move state into game loop, transfer ownership
}

fn game_loop(mut state : State) {
    while !state.rl_handle.window_should_close() {
        state.delta = state.rl_handle.get_frame_time();

        update(&mut state);
        render(&mut state);
    }
}

fn render(state : &mut State) {
    let mut d = state.rl_handle.begin_drawing(&state.thread);
    d.clear_background(Color::BLACK);

    // draw ship
    state.player.render(&mut d);

    // draw lasers
    for laser in &state.player.lasers {
        d.draw_circle_v(
            to_draw_vector(laser.pos),
            LASER_RADIUS,
            Color::WHITE);
    }

    // draw asteroids
    for asteroid in &state.asteroids {
        asteroid.render(&mut d);
    }

    // draw score and lives
    d.draw_text(&state.score.to_string(), 20, 20, SCORE_SIZE, Color::WHITE);
    for i in 1..state.player.lives+1 {
        d.draw_line_strip(&state.player.points
            .iter()
            .map(|p| (p.rotated(PI) * LIFE_SIZE) + Vector2::new(100.0*SCALE + (i as f32 * LIFE_SIZE), 45.0))
            .collect::<Vec<_>>(),
            Color::WHITE);
    }
}

fn update(state : &mut State) {
    if state.player.lives == 0 {
        state.score = 0;
        state.player.lives = 3;
    }

    state.player.update(state.delta, &state.rl_handle, &mut state.asteroids);

    if state.asteroids.len() < 20 {
        for _ in 0..20-state.asteroids.len() {
            state.asteroids.push(Asteroid::generate(None, None));
        }
    }

    let mut new: Vec<Asteroid> = Vec::new();
    let mut stale: Vec<usize> = Vec::new();

    for (idx, asteroid) in state.asteroids.iter_mut().enumerate() {
        asteroid.update(
            &mut state.score,
            state.delta,
            &mut state.player.lasers,
            &mut new);

        if asteroid.stale {
            stale.push(idx);
        }
    }

    clean_vec(&mut state.asteroids, stale);

    for a in new {
        state.asteroids.push(a);
    }
}

impl Player {
    fn new() -> Self {
        let points : [Vector2; 4] = [Vector2::new(-0.35, -0.5),
                                     Vector2::new(0.0, 0.5),
                                     Vector2::new(0.35, -0.5),
                                     Vector2::new(-0.35, -0.5)];

        let mut segments : Vec<ShipSegment> = Vec::new();
        for i in 0..points.len() {
            segments.push(ShipSegment {
                speed: 0.0,
                ds: Vector2::zero(),
                dir: 0.0,
                angle: 0.0,
                p1: points[i],
                p2: points[(i+1)%points.len()]
            });
        }

        Self {
            lives: 3,
            angle_r: 0.0,
            velocity: Vector2::zero(),
            pos: Vector2::zero(),
            exploding: false,
            explosion_delta: 0.0,
            points,
            segments,
            laser_cooldown: 0.0,
            lasers: Vec::new(),
        }
    }

    fn render(&self, handle : &mut RaylibDrawHandle) {
        if self.exploding {
            for segment in &self.segments {
                let p1 = to_draw_vector(
                    (segment.p1.clone().rotated(segment.angle) * SHIP_SCALE)
                    + self.pos + segment.ds);
                let p2 = to_draw_vector(
                    (segment.p2.clone().rotated(segment.angle) * SHIP_SCALE)
                    + self.pos + segment.ds);
                handle.draw_line_ex(p1, p2, THICKNESS, Color::WHITE);
            }
            return;
        }

        let transformed : Vec<Vector2> = self.points.iter()
            .map(|p|
                to_draw_vector(
                    (p.rotated(self.angle_r) * SHIP_SCALE)
                    + self.pos))
                .collect();

        for i in 0..transformed.len() {
            handle.draw_line_ex(
                transformed[i],
                transformed[(i+1)%transformed.len()],
                THICKNESS,
                Color::WHITE);
        }
    }

    fn explode(&mut self) {
        for segment in &mut self.segments {
            segment.speed = SEGMENT_SPEED * rng_min(0.5);
            segment.dir = 2.0 * PI * rand::random::<f32>();
            segment.angle = 2.0 * PI * rand::random::<f32>();
            segment.ds = Vector2::zero();
        }
    }

    fn update_lasers(&mut self, global_delta : f32) {
        self.laser_cooldown -= global_delta;

        let mut stale : Vec<usize> = Vec::new();
        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.pos += laser.dir * LASER_SPEED * global_delta;

            if ! in_bounds(laser.pos) || laser.hit {
                stale.push(i);
            }
        }

        clean_vec(&mut self.lasers, stale);
    }

    fn update_explosion(&mut self, global_delta : f32) {
        self.explosion_delta += global_delta;

        for segment in &mut self.segments {
            segment.speed *= 1.0 - DRAG;
            segment.ds += Vector2::new(segment.dir.cos(), segment.dir.sin()).normalized()
                * segment.speed
                * global_delta;
        }
    }

    fn reset(&mut self) {
        self.pos = Vector2::zero();
        self.angle_r = 0.0;
        self.velocity = Vector2::zero();
        self.explosion_delta = 0.0;
        self.exploding = false;
    }

    fn update(&mut self,
        global_delta : f32,
        rl_handle : &RaylibHandle,
        asteroids : &Vec<Asteroid>) {

        self.update_lasers(global_delta);

        if self.explosion_delta > 2.0 {
            self.reset();
        }

        if self.exploding {
            return self.update_explosion(global_delta);
        }

        let theta: f32 = self.angle_r - (PI * 3.0/2.0);
        let direction : Vector2 = Vector2::new(theta.cos(), theta.sin()).normalized(); 

        if rl_handle.is_key_down(KeyboardKey::KEY_RIGHT)
            || rl_handle.is_key_down(KeyboardKey::KEY_D) { // right
            self.angle_r -= ROT_SPEED;
        }
        if rl_handle.is_key_down(KeyboardKey::KEY_LEFT)
            || rl_handle.is_key_down(KeyboardKey::KEY_A) {
            self.angle_r += ROT_SPEED;
        }
        if rl_handle.is_key_down(KeyboardKey::KEY_UP)
            || rl_handle.is_key_down(KeyboardKey::KEY_W) {
            self.velocity += direction * SPEED * global_delta;
        }

        if rl_handle.is_key_down(KeyboardKey::KEY_SPACE)
            && self.laser_cooldown < 1e-6
            && self.lasers.len() < 5 {
            self.lasers.push(Laser {
                dir: direction,
                pos: self.pos + (direction * 0.5 * SHIP_SCALE),
                hit: false
            });
            self.laser_cooldown = 0.2;
        }

        self.velocity.scale(1.0 - DRAG);
        self.pos += self.velocity;

        if self.pos.x.abs() - (self.pos.x %(WINDOW_D.0/2.0)).abs() > 1e-6 {
            self.pos.x -= self.pos.x.signum() * WINDOW_D.0;
        }

        if self.pos.y.abs() - (self.pos.y % (WINDOW_D.1/2.0)).abs() > 1e-6 {
            self.pos.y -= self.pos.y.signum() * WINDOW_D.1;
        }

        for asteroid in asteroids {
            if self.pos.distance_to(asteroid.pos) < asteroid.radius && ! asteroid.destroyed {
                self.lives -= 1;
                self.exploding = true;
                self.explosion_delta = 0.0;

                self.explode();
                break;
            }
        }
    }
}

impl Asteroid {
    fn generate(size : Option<AsteroidSize>, pos : Option<Vector2>) -> Self {
        // TODO: fix this crap
        let pos : Vector2 = match pos {
            None => {
                loop {
                    let x = (rand::random::<f32>() * 2.0 * WINDOW_D.0) - WINDOW_D.0;
                    let y = (rand::random::<f32>() * 2.0 * WINDOW_D.1) - WINDOW_D.1;

                    if ! in_bounds(Vector2::new(x, y)) {
                        break Vector2::new(x, y);
                    }
                }
            },
            Some(vec) => vec
        };

        let size : AsteroidSize = match size {
            None => rand::random(),
            Some(s) => s
        };

        let speed = MAX_SPEED * rng_min(0.25);
        let radius = size.radius();
        let velocity = (Vector2::new(
                (rand::random::<f32>() * WINDOW_D.0) - (WINDOW_D.0/2.0),
                (rand::random::<f32>() * WINDOW_D.1) - (WINDOW_D.1/2.0)) - pos)
            .normalized() * speed;

        let n_points = rand::thread_rng().gen_range(8..14);
        let mut points: Vec<Vector2> = Vec::new();

        // generate shape
        for i in 0..n_points {
            let magnitude = rng_min(0.5);
            let theta = (PI * 2.0 / n_points as f32) * i as f32;
            let dir = Vector2::new(theta.cos(), theta.sin());

            points.push(dir * (radius * magnitude));
        }
        points.push(points[0]);

        Self {
            size,
            radius,
            points,
            velocity,
            pos,
            particles: Vec::new(),
            destroyed: false,
            stale : false
        }
    }

    fn generate_particles(&mut self) {
        for _ in 0..6 {
            let theta = rand::random::<f32>() * PI * 2.0;
            let dir = Vector2::new(theta.cos(), theta.sin()).normalized();
            self.particles.push(Particle {
                pos: self.pos,
                dir: dir,
                speed: PARTICLE_SPEED * rng_min(0.35),
                lifetime: 0.0
            });
        }
    }

    fn update_particles(&mut self, global_delta : f32) {
        for particle in &mut self.particles {
            particle.speed *= 1.0 - DRAG;
            particle.pos += particle.dir * particle.speed * global_delta;

            if particle.pos.distance_to(self.pos) > MAX_PARTICLE_DIST {
                self.stale = true;
                break;
            }
        }
    }

    fn update(&mut self, score : &mut i32, global_delta : f32, lasers : &mut Vec<Laser>, new : &mut Vec<Asteroid>) {
        if self.destroyed {
            return self.update_particles(global_delta)
        }

        self.pos += self.velocity * global_delta;

        if (!in_bounds(self.pos - (self.velocity.normalized() * self.radius)))
            && self.velocity.dot(-self.pos) < 0.0 {
            self.stale = true;
            return;
        }

        for laser in lasers {
            if self.pos.distance_to(laser.pos) > self.radius {
                continue;
            }

            laser.hit = true;
            *score += self.size.score();

            if self.size == AsteroidSize::Tiny {
                self.destroyed = true;
                self.generate_particles();
            } else {
                self.stale = true;
                for _ in 0..rand::thread_rng().gen_range(2..3) {
                    new.push(Asteroid::generate(
                        Some(self.size.split_size()),
                        Some(self.pos)));
                }
            }
        }
    }

    fn render(&self, handle: &mut RaylibDrawHandle) {
        if self.destroyed {
            for particle in &self.particles {
                handle.draw_circle_v(
                    to_draw_vector(particle.pos),
                    PARTICLE_RADIUS,
                    Color::WHITE);
            }
                return;
            }

            let transformed : Vec<Vector2> = self.points.iter().map(|p|
                to_draw_vector(*p + self.pos)).collect();

            for i in 0..transformed.len() {
                handle.draw_line_ex(
                    transformed[i],
                    transformed[(i+1)%transformed.len()],
                    THICKNESS,
                    Color::WHITE);
        }
    }
}
