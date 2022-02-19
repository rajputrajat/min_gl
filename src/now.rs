use glfw::WindowEvent;
use min_timer::{Now, Sec};

use crate::Display;

impl<T: FnMut(WindowEvent)> Now for Display<T> {
    fn now(&self) -> Sec {
        Sec::from(self.glfw().get_time())
    }
}
