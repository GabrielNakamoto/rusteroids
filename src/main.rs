use raylib::prelude::*;

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
    displacement: [f32; 2]
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
            angle_r: PI,
            ammo: 5,
            velocity: 0.0,
            accel: 0.0,
            displacement: [0.0; 2]
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

        update_inputs(state);
        update_player(state);

        let ship_lines = get_ship_lines(state);

        let mut d = state.rl_handle.begin_drawing(&state.thread);
        d.clear_background(Color::BLACK);
        d.draw_line_strip(&ship_lines, Color::WHITE);

        prev_time = cur_time;

        println!("Space: {}", state.keys[0]);
        println!("Delta: {}", state.delta);
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
    if state.keys[26] {
        state.player.displacement[0] += 0.1;
    }
    if state.keys[27] {
        state.player.displacement[0] -= 0.1;
    }
    if state.keys[28] {
        state.player.displacement[1] -= 0.1;
    }
    if state.keys[29] {
        state.player.displacement[1] += 0.1;
    }
}

fn get_ship_lines(state : &State) -> [Vector2; 4]{
    let mut ship_lines: [Vector2; 4] = [Vector2::new(-0.35, 0.0),
                                        Vector2::new(0.0, 1.0),
                                        Vector2::new(0.35, 0.0),
                                        Vector2::new(-0.35, 0.0)];
    for i in &mut ship_lines {
        i.scale(25.0);
        *i += Vector2::new(640.0/2.0, 480.0/2.0);
        *i += Vector2::new(state.player.displacement[0],
                           state.player.displacement[1]);
    }

    ship_lines
}
