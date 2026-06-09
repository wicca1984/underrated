//! Shell module for windowing and input.
//! spec: S-16

use crate::raster::Canvas;

/// A port for windowing operations.
/// spec: S-16
pub trait Window {
    /// Presents the given canvas to the window.
    fn present(&mut self, canvas: &Canvas);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod winit_adapter {
    use super::*;
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window as NativeWindow, WindowId};

    /// An adapter that uses winit and softbuffer to display a window.
    /// spec: S-16
    pub struct WinitWindow {
        title: String,
        width: u32,
        height: u32,
    }

    impl WinitWindow {
        /// Creates a new WinitWindow.
        /// spec: S-16
        pub fn new(title: &str, width: u32, height: u32) -> Self {
            Self {
                title: title.to_string(),
                width,
                height,
            }
        }

        /// Runs the event loop, calling `draw` to get the next canvas to present.
        /// spec: S-16
        pub fn run<F>(self, draw: F)
        where
            F: FnMut() -> Canvas,
        {
            let event_loop = match EventLoop::new() {
                Ok(el) => el,
                Err(_) => return, // spec: I-6: no panic
            };

            let mut app = App {
                window_attrs: Some((self.title, self.width, self.height)),
                state: None,
                draw,
            };

            let _ = event_loop.run_app(&mut app);
        }
    }

    struct AppState {
        window: std::sync::Arc<NativeWindow>,
        _context: softbuffer::Context<std::sync::Arc<NativeWindow>>,
        surface: softbuffer::Surface<std::sync::Arc<NativeWindow>, std::sync::Arc<NativeWindow>>,
    }

    struct App<F> {
        window_attrs: Option<(String, u32, u32)>,
        state: Option<AppState>,
        draw: F,
    }

    impl<F> ApplicationHandler for App<F>
    where
        F: FnMut() -> Canvas,
    {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.state.is_some() {
                return;
            }

            let (title, width, height) = match self.window_attrs.take() {
                Some(attrs) => attrs,
                None => return,
            };

            let window_attributes = NativeWindow::default_attributes()
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(width, height));

            let window = match event_loop.create_window(window_attributes) {
                Ok(w) => std::sync::Arc::new(w),
                Err(_) => return, // spec: I-6: no panic
            };

            let _context = match softbuffer::Context::new(window.clone()) {
                Ok(c) => c,
                Err(_) => return,
            };

            let surface = match softbuffer::Surface::new(&_context, window.clone()) {
                Ok(s) => s,
                Err(_) => return,
            };

            self.state = Some(AppState {
                window,
                _context,
                surface,
            });
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if let Some(state) = &mut self.state {
                        let canvas = (self.draw)();
                        let (width, height) = match (
                            NonZeroU32::new(canvas.width),
                            NonZeroU32::new(canvas.height),
                        ) {
                            (Some(w), Some(h)) => (w, h),
                            _ => return,
                        };

                        if state.surface.resize(width, height).is_err() {
                            return;
                        }

                        let mut buffer = match state.surface.buffer_mut() {
                            Ok(b) => b,
                            Err(_) => return,
                        };

                        // Blit Canvas (0xAARRGGBB) to softbuffer (0x00RRGGBB)
                        // softbuffer ignores the top byte, so we can just copy if the formats match.
                        // SPEC says Canvas is 0xAARRGGBB.
                        for (dest, src) in buffer.iter_mut().zip(canvas.pixels.iter()) {
                            *dest = *src;
                        }

                        let _ = buffer.present();
                    }
                }
                _ => (),
            }
        }

        fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
            if let Some(state) = &self.state {
                state.window.request_redraw();
            }
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use winit_adapter::WinitWindow;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::Canvas;

    struct MockWindow {
        last_canvas: Option<Canvas>,
    }

    impl Window for MockWindow {
        fn present(&mut self, canvas: &Canvas) {
            let mut saved = Canvas::new(canvas.width, canvas.height);
            saved.pixels = canvas.pixels.clone();
            self.last_canvas = Some(saved);
        }
    }

    #[test]
    fn test_mock_window() {
        let mut mock = MockWindow { last_canvas: None };
        let mut canvas = Canvas::new(10, 10);
        canvas.pixels[0] = 0xFF112233;

        mock.present(&canvas);

        let presented = mock.last_canvas.as_ref().unwrap();
        assert_eq!(presented.width, 10);
        assert_eq!(presented.height, 10);
        assert_eq!(presented.pixels[0], 0xFF112233);
    }
}
