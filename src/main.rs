use raylib::prelude::*;

const PI: f32 = 3.142;

struct Player {
    lives: i32,
    angle_r: f32, // radians
    ammo: i32,
    velocity: f32,
    accel: f32,
}

struct State {
    rl_handle: RaylibHandle,
    thread: RaylibThread,
    player: Player,
}


fn main() {
    let (rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello world!")
        .build();

    let mut state = State {
        rl_handle: rl,
        thread: thread,
        player: Player {
            lives: 3,
            angle_r: PI,
            ammo: 5,
            velocity: 0.0,
            accel: 0.0
        }
    };

    main_loop(&mut state);
}

fn main_loop(state : &mut State) {
    while !state.rl_handle.window_should_close() {
        let mut d = state.rl_handle.begin_drawing(&state.thread);

        d.clear_background(Color::BLACK);
    }
}

