use raylib::prelude::*;

const PI: f32 = 3.14159265359;


struct Player {
    lives: i32,
    angle_r: f32, // radians
    ammo: i32,
    velocity: f64,
    accel: f64,
}

struct State {
    rl_handle: RaylibHandle,
    thread: RaylibThread,
    player: Player,
    delta: f64
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
        },
        delta: 0.0
    };

    main_loop(&mut state);
}

fn main_loop(state : &mut State) {
    let mut ship_lines: [Vector2; 4] = [Vector2::new(-0.35, 0.0),
                                        Vector2::new(0.0, 1.0),
                                        Vector2::new(0.35, 0.0),
                                        Vector2::new(-0.35, 0.0)];
    for i in &mut ship_lines {
        i.scale(25.0);
        *i += Vector2::new(640.0/2.0, 480.0/2.0);
    }

    let mut prev_time: f64 = 0.0;
    while !state.rl_handle.window_should_close() {
        let mut cur_time = state.rl_handle.get_time();
        state.delta = cur_time - prev_time;
        let mut d = state.rl_handle.begin_drawing(&state.thread);

        d.draw_line_strip(&ship_lines, Color::WHITE);

        d.clear_background(Color::BLACK);
        prev_time = cur_time;

        println!("Delta: {}", state.delta);
    }
}
