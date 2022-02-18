use std::sync::mpsc::Receiver;

use glfw::{Context, Glfw, Monitor, SwapInterval, Window, WindowEvent};

/// OpenGL loading code, which is generated using glad v2.0.
///
/// # Safety
///
/// Only modification that was done on the output of glad was the removal of
/// the unsafe keyword from the macro `func!` that defined the functions.
/// Thus, all OpenGL calls can be tought of as unsafe!
pub mod gl;

/// Options for creating a display.
pub struct Options {
    /// With of the window in pixels.
    pub width: u32,
    /// Height of the window in pixels.
    /// Consider the aspect ratio (width/height) as 16/9.
    pub height: u32,
    /// Title of the window.
    pub title: String,
    /// Whether the window occupies all the monitor.
    pub fullscreen: bool,
    /// Whether the window has frame.
    /// Meaningfull when not in fullscreen mode.
    pub decorated: bool,
    /// Amount of fragments per pixel.
    /// Gives high quality smoother edges, but costly.
    /// Must be powers of 2 (I guess).
    /// Consider `Some(4)` for good quality/performance.
    /// Disabled if `None`.
    pub msaa: Option<u32>,
    /// Whether a monitor refresh must be waited between frames.
    /// Can decrease the frame rate a lot when struggling around the refresh rate.
    /// If not setted the frame rate is unbounded, which can lead to tearing.
    pub vsync: bool,
}

impl Options {
    fn config(&self, glfw: &mut Glfw) {
        use glfw::WindowHint::*;
        glfw.default_window_hints();
        glfw.window_hint(Resizable(false));
        glfw.window_hint(Decorated(self.decorated));
        glfw.window_hint(Samples(self.msaa));
        glfw.window_hint(ContextVersion(4, 6));
        glfw.window_hint(OpenGlForwardCompat(true));
        glfw.window_hint(OpenGlProfile(glfw::OpenGlProfileHint::Core));
        #[cfg(debug_assertions)]
        glfw.window_hint(OpenGlDebugContext(true));
    }

    fn create(&self, glfw: &mut Glfw, monitor: &Monitor) -> (Window, Receiver<(f64, WindowEvent)>) {
        let (mut window, events) = glfw
            .create_window(
                self.width,
                self.height,
                self.title.as_str(),
                if self.fullscreen {
                    glfw::WindowMode::FullScreen(monitor)
                } else {
                    glfw::WindowMode::Windowed
                },
            )
            .expect("Could not create the window!");
        let vidmode = monitor
            .get_video_mode()
            .expect("Could not get the vidmode of the monitor!");
        window.set_pos(
            (vidmode.width - self.width) as i32 / 2,
            (vidmode.height - self.height) as i32 / 2,
        );
        window.set_cursor_pos(self.width as f64 / 2.0, self.height as f64 / 2.0);
        (window, events)
    }

    fn config_context(&self, glfw: &mut Glfw) {
        glfw.set_swap_interval(SwapInterval::Sync(self.vsync as u32));
        gl::Viewport(0, 0, self.width as i32, self.height as i32);
        match self.msaa {
            Some(_) => gl::Enable(gl::MULTISAMPLE),
            None => gl::Disable(gl::MULTISAMPLE),
        }
    }
}

/// [GLFW](glfw) window with valid OpenGL 4.6 CORE context loaded by [GLAD](gl).
pub struct Display<T>
where
    T: FnMut(WindowEvent) -> (),
{
    window: Window,
    handler: T,
    events: Receiver<(f64, WindowEvent)>,
}

impl<T> Display<T>
where
    T: FnMut(WindowEvent) -> (),
{
    /// Creates and sets up a new [glfw::Window].
    /// Calls the given [window event](glfw::WindowEvent) handler after polling.
    /// Must be initialized and used on the same thread all the OpenGL calls are done.
    ///
    /// # Panics
    ///
    /// - On GLFW Errors.
    /// - If cannot initialize GLFW.
    /// - If cannot get the primary monitor.
    /// - If cannot create the window.
    /// - If cannot get the primary monitor's video mode.
    pub fn new(opt: Options, handler: T) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Could not initialize the GLFW!");
        let (mut window, events) = glfw.with_primary_monitor(|glfw, monitor| {
            if let Some(monitor) = monitor {
                opt.config(glfw);
                opt.create(glfw, monitor)
            } else {
                panic!("Could not get the primary monitor!");
            }
        });
        window.set_all_polling(true);
        window.make_current();
        gl::load(|proc| glfw.get_proc_address_raw(proc));
        opt.config_context(&mut glfw);
        Self {
            window,
            handler,
            events,
        }
    }

    /// Renders the drawn contents and clears the color buffer for next frame.
    /// Will wait for a monitor refresh with VSync enabled.
    pub fn render(&mut self) {
        self.window.swap_buffers();
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    /// Polls the [window events](glfw::WindowEvent) and calls the handler.
    pub fn update(&mut self) {
        self.glfw().poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            (self.handler)(event);
        }
    }

    /// Returns the [glfw::Window].
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    /// Returns the [glfw::Glfw].
    pub fn glfw(&mut self) -> &mut Glfw {
        &mut self.window().glfw
    }
}

#[cfg(test)]
mod tests {
    use glfw::WindowEvent;

    use crate::{gl, Display, Options};

    #[test]
    #[ignore]
    fn window_with_event_logging() {
        // Assume this is some application state.
        let mut event_count = 0u32;

        // Just create and done!
        // All library initialization and window creation is handled.
        // They panic if an error occurs.
        let mut disp = Display::new(
            // No defaults; you cannot miss anything!
            Options {
                width: 1280,
                height: 720,
                title: "Display Test".into(),
                fullscreen: false,
                decorated: true,
                msaa: Some(16),
                vsync: true,
            },
            // WindowEvent handling...
            |event| {
                event_count += 1; // Closure can modify state (FnMut).

                match event {
                    WindowEvent::Key(k, _, a, _) => println!("Key `{:?}` is {:?}ed!", k, a),
                    e => println!("Some {:?} happened!", e),
                }
            },
        );

        // Of course, you can go with more complicated main loops.
        while !disp.window().should_close() {
            // All window events...
            disp.update();

            /* ~~~~ drawing start ~~~~ */

            gl::ClearColor(0.7, 0.5, 0.6, 1.0);

            /* ~~~~ drawing end ~~~~ */

            disp.render();
        }

        // No clean up; thanks to idomatic glfw-rs!
        // OpenGL calls are not valid after `disp` is dropped!
        println!("In total {} window events happened!", event_count);
    }
}
