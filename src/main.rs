use raylib::prelude::*;
use std::cmp;

const PI: f32 = 3.14159265359;
const WINDOW_D: (i32, i32) = (640, 580);

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
    velocity: f64,
    accel: f64,
    displacement: Vector2
}

struct State {
    rl_handle: RaylibHandle,
    thread: RaylibThread,
    player: Player,
    delta: f64,
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
            angle_r: 0.0,
            ammo: 5,
            velocity: 0.0,
            accel: 0.0,
            displacement: Vector2::zero()
        },
        delta: 0.0,
        keys: [false; 31]
    };

    main_loop(&mut state);
}

fn main_loop(state : &mut State) {

    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;
        // state.player.angle_r += state.delta as f32;

        update_inputs(state);
        update_player(state);

        let ship_lines = serialize_player(state);

        let mut d = state.rl_handle.begin_drawing(&state.thread);
        d.clear_background(Color::BLACK);
        d.draw_line_strip(&ship_lines, Color::WHITE);

        println!("Angle: {}", state.player.angle_r);
        println!("Accel: {}", state.player.accel);
        println!("Vel: {}", state.player.velocity);
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

fn update_player(state : &mut State) {
    // rotate at constant velocity?
    // unless moving
    if state.keys[26] { // right
        state.player.angle_r += 0.0012;
    }
    if state.keys[27] { // left
        state.player.angle_r -= 0.0012;
    }
    if state.keys[28] {
        if state.player.accel < 0.01 {
            state.player.accel += 0.001;
        }
    } else if state.player.accel > 0.0 {
        state.player.accel = (state.player.accel - 0.001).max(0.0);
    }

    state.player.velocity += state.player.accel * state.delta;

    // how to find vector from angle?
    // x = cos theta * hyp
    // y = sin theta * hyp
    // hyp = 1 cause unit vector
    let theta: f32 = state.player.angle_r - (PI * 3.0/2.0);
    let mut displacement_delta : Vector2 = Vector2::new(theta.cos(), theta.sin()); 
    displacement_delta.scale(state.player.velocity as f32);

    state.player.displacement += displacement_delta;
    // find displacement vector
    // (1) find unit direction vector
    // (2) move along it by velocity
    // (3) add to current displacement vector
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
        *i += Vector2::new(640.0/2.0, 480.0/2.0);
        *i += state.player.displacement;
    }

    ship_lines
}
