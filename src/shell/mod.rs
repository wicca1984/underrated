//! Shell module for windowing and input.
//! spec: S-16, S-34

use crate::raster::Canvas;

/// High-level input event mapped from window input.
/// spec: S-34
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A mouse click event.
    Click { x: f64, y: f64 },
    /// A keyboard key press event.
    Key { key: String },
}

/// A port for windowing operations.
/// spec: S-16
pub trait Window {
    /// Presents the given canvas to the window.
    fn present(&mut self, canvas: &Canvas);
}

/// Maps winit input to click coordinates.
/// spec: S-34
pub fn mouse_click_at(x: f64, y: f64) -> (f64, f64) {
    (x, y)
}

/// Maps winit input to click coordinates.
/// spec: S-34
pub fn map_mouse_click(x: f64, y: f64) -> (f64, f64) {
    (x, y)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod winit_adapter {
    use super::*;
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window as NativeWindow, WindowId};

    /// Maps a winit `WindowEvent` and current cursor position to an optional high-level input intent.
    /// spec: S-34
    pub fn map_window_event(event: &WindowEvent, cursor_pos: (f64, f64)) -> Option<InputEvent> {
        match event {
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                let (cx, cy) = mouse_click_at(cursor_pos.0, cursor_pos.1);
                Some(InputEvent::Click { x: cx, y: cy })
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    let key_str = match key_event.logical_key.as_ref() {
                        winit::keyboard::Key::Character(s) => s.to_string(),
                        winit::keyboard::Key::Named(named_key) => format!("{:?}", named_key),
                        winit::keyboard::Key::Unidentified(_) => "Unidentified".to_string(),
                        winit::keyboard::Key::Dead(dead_key) => format!("Dead({:?})", dead_key),
                    };
                    Some(InputEvent::Key { key: key_str })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

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

        /// Runs the event loop with input support, calling `draw` to get the next canvas to present,
        /// and `on_event` when a mapped input event occurs.
        /// spec: S-34
        pub fn run_with_input<F, G>(self, draw: F, on_event: G)
        where
            F: FnMut() -> Canvas,
            G: FnMut(InputEvent),
        {
            let event_loop = match EventLoop::new() {
                Ok(el) => el,
                Err(_) => return, // spec: I-6: no panic
            };

            let mut app = AppWithInput {
                window_attrs: Some((self.title, self.width, self.height)),
                state: None,
                draw,
                on_event,
                cursor_pos: (0.0, 0.0),
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

    struct AppWithInput<F, G> {
        window_attrs: Option<(String, u32, u32)>,
        state: Option<AppState>,
        draw: F,
        on_event: G,
        cursor_pos: (f64, f64),
    }

    impl<F, G> ApplicationHandler for AppWithInput<F, G>
    where
        F: FnMut() -> Canvas,
        G: FnMut(InputEvent),
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
            if let WindowEvent::CursorMoved { position, .. } = &event {
                self.cursor_pos = (position.x, position.y);
            }

            if let Some(input_event) = map_window_event(&event, self.cursor_pos) {
                (self.on_event)(input_event);
            }

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
pub use winit_adapter::{WinitWindow, map_window_event};

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

    #[test]
    fn test_mouse_click_conversion() {
        assert_eq!(mouse_click_at(12.3, 45.6), (12.3, 45.6));
        assert_eq!(map_mouse_click(100.0, 200.0), (100.0, 200.0));
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[test]
    fn test_map_window_event_click() {
        let device_id = winit::event::DeviceId::dummy();
        let event = winit::event::WindowEvent::MouseInput {
            device_id,
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left,
        };
        let input_evt = map_window_event(&event, (50.0, 100.0));
        assert_eq!(input_evt, Some(InputEvent::Click { x: 50.0, y: 100.0 }));

        // Non-left buttons or releases shouldn't produce a click
        let event_right = winit::event::WindowEvent::MouseInput {
            device_id,
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Right,
        };
        assert_eq!(map_window_event(&event_right, (50.0, 100.0)), None);

        let event_released = winit::event::WindowEvent::MouseInput {
            device_id,
            state: winit::event::ElementState::Released,
            button: winit::event::MouseButton::Left,
        };
        assert_eq!(map_window_event(&event_released, (50.0, 100.0)), None);
    }
}
