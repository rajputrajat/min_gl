use glfw::WindowEvent;
use min_gl::{gl, Display, Options};

fn main() {
    // Assume this is some application state.
    let mut event_count = 0u32;
    // Just create and done!
    // All library initialization and window created is handled.
    // They panic if an error occurs.
    let mut disp = Display::new(
        // No defaults; you cannot miss anything!
        Options {
            width: 1280,
            height: 720,
            title: "Display Test".into(),
            fullscreen: false,
            decorated: true,
            samples: 16,
            vsync: true,
        },
        |event| {
            event_count += 1; // Closure can modify state (FnMut).
            match event {
                WindowEvent::Key(k, _, a, _) => println!("Key `{:?}` is {:?}ed!", k, a),
                e => println!("Some {:?} happened!", e),
            }
        },
    );
    // Of course, you can go with more complicated main loops.
    while !disp.get_window().should_close() {
        disp.update();
        /* drawing start */
        gl::ClearColor(0.7, 0.5, 0.6, 1.0);
        /* drawing end */
        disp.render();
    }
    println!("In total {} window events happened!", event_count);
}
