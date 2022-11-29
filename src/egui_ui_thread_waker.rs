use std::ops::{Deref, DerefMut};

use egui::Context;

use ui_thread_waker::UiThreadWaker;

#[repr(transparent)]
pub struct EguiUiThreadWaker(pub Context);

impl From<Context> for EguiUiThreadWaker {
    fn from(ctx: Context) -> Self {
        Self(ctx)
    }
}

impl Deref for EguiUiThreadWaker {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EguiUiThreadWaker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl UiThreadWaker for EguiUiThreadWaker {
    fn wake_ui_thread(&self) {
        self.0.request_repaint();
    }
}
