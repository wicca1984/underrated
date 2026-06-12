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

/// Returns true if the high-level input event triggers a window redraw.
pub fn input_event_triggers_redraw(ev: &InputEvent) -> bool {
    match ev {
        InputEvent::Click { .. } | InputEvent::Key { .. } => true,
    }
}

/// Represents the visual rendering geometry of the text insertion caret.
/// spec: S-34, t0176
#[derive(Debug, Clone, PartialEq)]
pub struct CaretGeometry {
    /// The NodeId of the focused input element.
    pub node_id: crate::infra::NodeId,
    /// The character index of the caret within the text.
    pub char_index: usize,
    /// The computed horizontal coordinate of the caret.
    pub x: f64,
    /// The computed vertical coordinate of the caret.
    pub y: f64,
    /// The computed width of the caret.
    pub width: f64,
    /// The computed height of the caret.
    pub height: f64,
}

/// Helper function to insert a character at a specific character index (UTF-8 safe).
fn insert_char_at(s: &mut String, char_idx: usize, ch: char) -> usize {
    let mut chars: Vec<char> = s.chars().collect();
    let clamped_idx = char_idx.min(chars.len());
    chars.insert(clamped_idx, ch);
    *s = chars.into_iter().collect();
    clamped_idx + 1
}

/// Helper function to delete a character before a specific character index (UTF-8 safe).
fn delete_char_before(s: &mut String, char_idx: usize) -> Option<usize> {
    if char_idx == 0 {
        return None;
    }
    let mut chars: Vec<char> = s.chars().collect();
    let clamped_idx = char_idx.min(chars.len());
    chars.remove(clamped_idx - 1);
    *s = chars.into_iter().collect();
    Some(clamped_idx - 1)
}

/// Tracks focused text-input elements, routes keyboard inputs, and computes caret geometry.
/// spec: S-34, t0176
#[derive(Debug, Default, Clone)]
pub struct ShellInputManager {
    focused_element: Option<crate::infra::NodeId>,
    text_buffers: std::collections::HashMap<crate::infra::NodeId, String>,
    caret_positions: std::collections::HashMap<crate::infra::NodeId, usize>,
}

impl ShellInputManager {
    /// Creates a new ShellInputManager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently focused NodeId, if any.
    pub fn focused_element(&self) -> Option<crate::infra::NodeId> {
        self.focused_element
    }

    /// Focuses the given node.
    pub fn focus(&mut self, node_id: crate::infra::NodeId) {
        self.focused_element = Some(node_id);
        // Initialize text buffer and caret position if not present
        self.text_buffers.entry(node_id).or_default();
        self.caret_positions.entry(node_id).or_insert(0);
    }

    /// Blurs the currently focused node.
    pub fn blur(&mut self) {
        self.focused_element = None;
    }

    /// Handles a click event. If a focus target (hit-test result) is provided, it is focused.
    /// Otherwise, focus is cleared.
    pub fn handle_click(&mut self, _x: f64, _y: f64, hit_test: Option<crate::infra::NodeId>) {
        if let Some(node_id) = hit_test {
            self.focus(node_id);
        } else {
            self.blur();
        }
    }

    /// Gets the text buffer for the given node, or an empty string if none exists.
    pub fn text_buffer(&self, node_id: crate::infra::NodeId) -> &str {
        self.text_buffers
            .get(&node_id)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Sets the text buffer for the given node.
    pub fn set_text_buffer(&mut self, node_id: crate::infra::NodeId, text: String) {
        let len = text.chars().count();
        self.text_buffers.insert(node_id, text);
        if let Some(caret) = self.caret_positions.get_mut(&node_id) {
            if *caret > len {
                *caret = len;
            }
        } else {
            self.caret_positions.insert(node_id, len);
        }
    }

    /// Gets the caret position (as a character count) for the given node.
    pub fn caret_position(&self, node_id: crate::infra::NodeId) -> usize {
        *self.caret_positions.get(&node_id).unwrap_or(&0)
    }

    /// Sets the caret position for the given node.
    pub fn set_caret_position(&mut self, node_id: crate::infra::NodeId, pos: usize) {
        let text_len = self.text_buffer(node_id).chars().count();
        self.caret_positions.insert(node_id, pos.min(text_len));
    }

    /// Handles a keyboard key press. Returns `true` if the key was consumed by the active text input,
    /// or `false` if it should be passed through (e.g. for page scrolling or form submission).
    pub fn handle_key(&mut self, key: &str) -> bool {
        let Some(focused) = self.focused_element else {
            return false;
        };

        match key {
            "Backspace" => {
                let mut text = self.text_buffer(focused).to_string();
                let caret = self.caret_position(focused);
                if let Some(new_caret) = delete_char_before(&mut text, caret) {
                    self.text_buffers.insert(focused, text);
                    self.caret_positions.insert(focused, new_caret);
                }
                true
            }
            "ArrowLeft" => {
                let caret = self.caret_position(focused);
                if caret > 0 {
                    self.caret_positions.insert(focused, caret - 1);
                }
                true
            }
            "ArrowRight" => {
                let caret = self.caret_position(focused);
                let text_len = self.text_buffer(focused).chars().count();
                if caret < text_len {
                    self.caret_positions.insert(focused, caret + 1);
                }
                true
            }
            "Space" => {
                let mut text = self.text_buffer(focused).to_string();
                let caret = self.caret_position(focused);
                let new_caret = insert_char_at(&mut text, caret, ' ');
                self.text_buffers.insert(focused, text);
                self.caret_positions.insert(focused, new_caret);
                true
            }
            // Non-text control keys are passed through
            "ArrowUp" | "ArrowDown" | "PageUp" | "PageDown" | "Enter" | "Escape" | "Tab" => false,
            other => {
                let mut chars = other.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) if !ch.is_control() => {
                        let mut text = self.text_buffer(focused).to_string();
                        let caret = self.caret_position(focused);
                        let new_caret = insert_char_at(&mut text, caret, ch);
                        self.text_buffers.insert(focused, text);
                        self.caret_positions.insert(focused, new_caret);
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    /// Computes caret geometry relative to the input field bounding box.
    pub fn calculate_caret_geometry(
        &self,
        node_id: crate::infra::NodeId,
        input_x: f64,
        input_y: f64,
        input_height: f64,
        text_offset: f64,
    ) -> Option<CaretGeometry> {
        if self.focused_element != Some(node_id) {
            return None;
        }
        let char_index = self.caret_position(node_id);
        Some(CaretGeometry {
            node_id,
            char_index,
            x: input_x + text_offset,
            y: input_y,
            width: 1.5,
            height: input_height,
        })
    }
}

/// A port for windowing operations.
/// spec: S-16
pub trait Window {
    /// Presents the given canvas to the window.
    fn present(&mut self, canvas: &Canvas);

    /// Gets the current vertical scroll offset.
    fn scroll_offset_y(&self) -> f64 {
        0.0
    }

    /// Sets the vertical scroll offset, clamping appropriately.
    fn set_scroll_offset_y(&mut self, _offset: f64) {}

    /// Gets the total content height.
    fn content_height(&self) -> f64 {
        0.0
    }

    /// Sets the total content height.
    fn set_content_height(&mut self, _height: f64) {}

    /// Scrolls vertically by the given delta (positive scrolls down, negative scrolls up).
    fn scroll_by(&mut self, delta: f64) {
        let current = self.scroll_offset_y();
        self.set_scroll_offset_y(current + delta);
    }
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

    /// Runs `draw` on a dedicated 16 MiB-stack thread so deep (but bounded) DOM/CSS/JS
    /// work cannot overflow the small Windows main-thread stack. Falls back to running
    /// inline if the OS cannot spawn the thread (I-6: never panic).
    fn render_on_big_stack<F>(draw: &mut F) -> Canvas
    where
        F: FnMut() -> Canvas + Send,
    {
        let produced = std::thread::scope(|scope| {
            match std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, &mut *draw)
            {
                Ok(handle) => handle.join().ok(), // Some(canvas) on success; None if the render thread panicked
                Err(_) => None,                   // OS refused to spawn the thread
            }
        });
        match produced {
            Some(canvas) => canvas,
            None => draw(), // graceful degrade on spawn failure; L2 caps already prevent panics
        }
    }

    /// An adapter that uses winit and softbuffer to display a window.
    /// spec: S-16
    pub struct WinitWindow {
        title: String,
        width: u32,
        height: u32,
        scroll_offset_y: f64,
        content_height: f64,
    }

    impl WinitWindow {
        /// Creates a new WinitWindow.
        /// spec: S-16
        pub fn new(title: &str, width: u32, height: u32) -> Self {
            Self {
                title: title.to_string(),
                width,
                height,
                scroll_offset_y: 0.0,
                content_height: 0.0,
            }
        }

        /// Gets the current vertical scroll offset.
        pub fn scroll_offset_y(&self) -> f64 {
            self.scroll_offset_y
        }

        /// Sets the vertical scroll offset, clamped to the valid scroll range.
        pub fn set_scroll_offset_y(&mut self, offset: f64) {
            let max_scroll = (self.content_height - self.height as f64).max(0.0);
            self.scroll_offset_y = offset.clamp(0.0, max_scroll);
        }

        /// Gets the total content height.
        pub fn content_height(&self) -> f64 {
            self.content_height
        }

        /// Sets the total content height and re-clamps the scroll offset.
        pub fn set_content_height(&mut self, height: f64) {
            self.content_height = height;
            self.scroll_offset_y = self
                .scroll_offset_y
                .clamp(0.0, (self.content_height - self.height as f64).max(0.0));
        }

        /// Runs the event loop, calling `draw` to get the next canvas to present.
        /// spec: S-16
        pub fn run<F>(self, mut draw: F)
        where
            F: FnMut() -> Canvas + Send,
        {
            let event_loop = match EventLoop::new() {
                Ok(el) => el,
                Err(_) => return, // spec: I-6: no panic
            };

            let draw = move || render_on_big_stack(&mut draw);

            let mut app = App {
                window_attrs: Some((self.title, self.width, self.height)),
                state: None,
                draw,
                viewport_width: self.width,
                viewport_height: self.height,
                scroll_offset_y: self.scroll_offset_y,
                content_height: self.content_height,
            };

            let _ = event_loop.run_app(&mut app);
        }

        /// Runs the event loop with input support, calling `draw` to get the next canvas to present,
        /// and `on_event` when a mapped input event occurs.
        /// spec: S-34
        pub fn run_with_input<F, G>(self, mut draw: F, on_event: G)
        where
            F: FnMut() -> Canvas + Send,
            G: FnMut(InputEvent),
        {
            let event_loop = match EventLoop::new() {
                Ok(el) => el,
                Err(_) => return, // spec: I-6: no panic
            };

            let draw = move || render_on_big_stack(&mut draw);

            let mut app = AppWithInput {
                window_attrs: Some((self.title, self.width, self.height)),
                state: None,
                draw,
                on_event,
                cursor_pos: (0.0, 0.0),
                viewport_width: self.width,
                viewport_height: self.height,
                scroll_offset_y: self.scroll_offset_y,
                content_height: self.content_height,
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
        viewport_width: u32,
        viewport_height: u32,
        scroll_offset_y: f64,
        content_height: f64,
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
            if let Some(state) = &self.state {
                state.window.request_redraw();
            }
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
                WindowEvent::MouseWheel { delta, .. } => {
                    let d = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => -(y as f64) * 40.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => -pos.y,
                    };

                    let max_scroll = (self.content_height - self.viewport_height as f64).max(0.0);
                    self.scroll_offset_y = (self.scroll_offset_y + d).clamp(0.0, max_scroll);
                    if let Some(state) = &self.state {
                        state.window.request_redraw();
                    }
                }
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    if key_event.state == winit::event::ElementState::Pressed {
                        let delta = match &key_event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                                Some(40.0)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                                Some(-40.0)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown) => {
                                Some(self.viewport_height as f64 * 0.9)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp) => {
                                Some(-(self.viewport_height as f64 * 0.9))
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                                Some(self.viewport_height as f64 * 0.9)
                            }
                            _ => None,
                        };

                        if let Some(d) = delta {
                            let max_scroll =
                                (self.content_height - self.viewport_height as f64).max(0.0);
                            self.scroll_offset_y =
                                (self.scroll_offset_y + d).clamp(0.0, max_scroll);
                            if let Some(state) = &self.state {
                                state.window.request_redraw();
                            }
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(state) = &mut self.state {
                        let canvas = (self.draw)();

                        // Set total content height and re-clamp scroll
                        self.content_height = canvas.height as f64;
                        let max_scroll =
                            (self.content_height - self.viewport_height as f64).max(0.0);
                        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll);

                        let (width, height) = match (
                            NonZeroU32::new(self.viewport_width),
                            NonZeroU32::new(self.viewport_height),
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

                        let start_y = self.scroll_offset_y.round() as i32;
                        for vy in 0..self.viewport_height {
                            let cy = start_y + vy as i32;
                            for vx in 0..self.viewport_width {
                                let dest_index =
                                    (vy as usize) * (self.viewport_width as usize) + (vx as usize);
                                if cy >= 0 && cy < canvas.height as i32 && vx < canvas.width {
                                    let src_index =
                                        (cy as usize) * (canvas.width as usize) + (vx as usize);
                                    buffer[dest_index] = canvas.pixels[src_index];
                                } else {
                                    buffer[dest_index] = 0xFFFFFFFF; // default white background
                                }
                            }
                        }

                        let _ = buffer.present();
                    }
                }
                _ => (),
            }
        }

        fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
    }

    struct AppWithInput<F, G> {
        window_attrs: Option<(String, u32, u32)>,
        state: Option<AppState>,
        draw: F,
        on_event: G,
        cursor_pos: (f64, f64),
        viewport_width: u32,
        viewport_height: u32,
        scroll_offset_y: f64,
        content_height: f64,
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
            if let Some(state) = &self.state {
                state.window.request_redraw();
            }
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
                let adjusted_event = match input_event {
                    InputEvent::Click { x, y } => InputEvent::Click {
                        x,
                        y: y + self.scroll_offset_y,
                    },
                    other => other,
                };
                let triggers_redraw = input_event_triggers_redraw(&adjusted_event);
                (self.on_event)(adjusted_event);
                if triggers_redraw && let Some(state) = &self.state {
                    state.window.request_redraw();
                }
            }

            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let d = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => -(y as f64) * 40.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => -pos.y,
                    };

                    let max_scroll = (self.content_height - self.viewport_height as f64).max(0.0);
                    self.scroll_offset_y = (self.scroll_offset_y + d).clamp(0.0, max_scroll);
                    if let Some(state) = &self.state {
                        state.window.request_redraw();
                    }
                }
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    if key_event.state == winit::event::ElementState::Pressed {
                        let delta = match &key_event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                                Some(40.0)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                                Some(-40.0)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown) => {
                                Some(self.viewport_height as f64 * 0.9)
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp) => {
                                Some(-(self.viewport_height as f64 * 0.9))
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                                Some(self.viewport_height as f64 * 0.9)
                            }
                            _ => None,
                        };

                        if let Some(d) = delta {
                            let max_scroll =
                                (self.content_height - self.viewport_height as f64).max(0.0);
                            self.scroll_offset_y =
                                (self.scroll_offset_y + d).clamp(0.0, max_scroll);
                            if let Some(state) = &self.state {
                                state.window.request_redraw();
                            }
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(state) = &mut self.state {
                        let canvas = (self.draw)();

                        // Set total content height and re-clamp scroll
                        self.content_height = canvas.height as f64;
                        let max_scroll =
                            (self.content_height - self.viewport_height as f64).max(0.0);
                        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll);

                        let (width, height) = match (
                            NonZeroU32::new(self.viewport_width),
                            NonZeroU32::new(self.viewport_height),
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

                        let start_y = self.scroll_offset_y.round() as i32;
                        for vy in 0..self.viewport_height {
                            let cy = start_y + vy as i32;
                            for vx in 0..self.viewport_width {
                                let dest_index =
                                    (vy as usize) * (self.viewport_width as usize) + (vx as usize);
                                if cy >= 0 && cy < canvas.height as i32 && vx < canvas.width {
                                    let src_index =
                                        (cy as usize) * (canvas.width as usize) + (vx as usize);
                                    buffer[dest_index] = canvas.pixels[src_index];
                                } else {
                                    buffer[dest_index] = 0xFFFFFFFF; // default white background
                                }
                            }
                        }

                        let _ = buffer.present();
                    }
                }
                _ => (),
            }
        }

        fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
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
        viewport_width: u32,
        viewport_height: u32,
        scroll_offset_y: f64,
        content_height: f64,
    }

    impl Window for MockWindow {
        fn present(&mut self, canvas: &Canvas) {
            if self.content_height == 0.0 {
                self.content_height = canvas.height as f64;
            }
            let max_scroll = (self.content_height - self.viewport_height as f64).max(0.0);
            self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll);

            let mut viewport_canvas = Canvas::new(self.viewport_width, self.viewport_height);
            let start_y = self.scroll_offset_y.round() as i32;

            for vy in 0..self.viewport_height {
                let cy = start_y + vy as i32;
                for vx in 0..self.viewport_width {
                    let dest_index = (vy as usize) * (self.viewport_width as usize) + (vx as usize);
                    if cy >= 0 && cy < canvas.height as i32 && vx < canvas.width {
                        let src_index = (cy as usize) * (canvas.width as usize) + (vx as usize);
                        viewport_canvas.pixels[dest_index] = canvas.pixels[src_index];
                    } else {
                        viewport_canvas.pixels[dest_index] = 0xFFFFFFFF; // background default
                    }
                }
            }
            self.last_canvas = Some(viewport_canvas);
        }

        fn scroll_offset_y(&self) -> f64 {
            self.scroll_offset_y
        }

        fn set_scroll_offset_y(&mut self, offset: f64) {
            let max_scroll = (self.content_height - self.viewport_height as f64).max(0.0);
            self.scroll_offset_y = offset.clamp(0.0, max_scroll);
        }

        fn content_height(&self) -> f64 {
            self.content_height
        }

        fn set_content_height(&mut self, height: f64) {
            self.content_height = height;
            let max_scroll = (self.content_height - self.viewport_height as f64).max(0.0);
            self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll);
        }
    }

    #[test]
    fn test_mock_window() {
        let mut mock = MockWindow {
            last_canvas: None,
            viewport_width: 10,
            viewport_height: 10,
            scroll_offset_y: 0.0,
            content_height: 10.0,
        };
        let mut canvas = Canvas::new(10, 10);
        canvas.pixels[0] = 0xFF112233;

        mock.present(&canvas);

        let presented = mock.last_canvas.as_ref().unwrap();
        assert_eq!(presented.width, 10);
        assert_eq!(presented.height, 10);
        assert_eq!(presented.pixels[0], 0xFF112233);
    }

    #[test]
    fn test_scroll_logic() {
        let mut mock = MockWindow {
            last_canvas: None,
            viewport_width: 10,
            viewport_height: 10,
            scroll_offset_y: 0.0,
            content_height: 25.0,
        };

        // Create a tall canvas
        let mut canvas = Canvas::new(10, 25);
        // Put a marked pixel at y=15 (which is off-screen initially)
        canvas.pixels[15 * 10] = 0xFF998877;

        // Present with scroll = 0
        mock.present(&canvas);
        let presented_1 = mock.last_canvas.as_ref().unwrap();
        // The pixel at presented_1[0] should NOT be 0xFF998877 because we are scrolled to 0.
        assert_ne!(presented_1.pixels[0], 0xFF998877);

        // Now scroll down by 15 pixels
        mock.scroll_by(15.0);
        assert_eq!(mock.scroll_offset_y(), 15.0);

        // Present again
        mock.present(&canvas);
        let presented_2 = mock.last_canvas.as_ref().unwrap();
        // Now the pixel that was at y=15 in the canvas should be at y=0 in the presented viewport!
        assert_eq!(presented_2.pixels[0], 0xFF998877);

        // Test clamping: scrolling past content should clamp to (content_height - viewport_height) = 15.0
        mock.scroll_by(20.0); // total scroll would be 35
        assert_eq!(mock.scroll_offset_y(), 15.0); // should clamp to 15.0
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

    #[test]
    fn test_shell_input_manager_focus_and_click() {
        let mut manager = ShellInputManager::new();
        let mut arena = crate::infra::Arena::new();
        let node_id_1 = arena.insert("input1");
        let node_id_2 = arena.insert("input2");

        // Set focus directly
        manager.focus(node_id_1);
        assert_eq!(manager.focused_element(), Some(node_id_1));

        // Blur
        manager.blur();
        assert_eq!(manager.focused_element(), None);

        // Click on an element focuses it
        manager.handle_click(10.0, 20.0, Some(node_id_2));
        assert_eq!(manager.focused_element(), Some(node_id_2));

        // Click outside blurs
        manager.handle_click(10.0, 20.0, None);
        assert_eq!(manager.focused_element(), None);
    }

    #[test]
    fn test_shell_input_manager_key_routing_and_backspace() {
        let mut manager = ShellInputManager::new();
        let mut arena = crate::infra::Arena::new();
        let node_id = arena.insert("input");

        // Focus the input
        manager.focus(node_id);
        assert_eq!(manager.text_buffer(node_id), "");
        assert_eq!(manager.caret_position(node_id), 0);

        // Type 'H'
        let consumed = manager.handle_key("H");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "H");
        assert_eq!(manager.caret_position(node_id), 1);

        // Type 'i'
        let consumed = manager.handle_key("i");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "Hi");
        assert_eq!(manager.caret_position(node_id), 2);

        // Backspace
        let consumed = manager.handle_key("Backspace");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "H");
        assert_eq!(manager.caret_position(node_id), 1);

        // Backspace again
        let consumed = manager.handle_key("Backspace");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "");
        assert_eq!(manager.caret_position(node_id), 0);

        // Backspace on empty shouldn't panic, should return true (consumed)
        let consumed = manager.handle_key("Backspace");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "");
        assert_eq!(manager.caret_position(node_id), 0);
    }

    #[test]
    fn test_shell_input_manager_arrow_navigation() {
        let mut manager = ShellInputManager::new();
        let mut arena = crate::infra::Arena::new();
        let node_id = arena.insert("input");

        manager.focus(node_id);
        manager.handle_key("a");
        manager.handle_key("b");
        manager.handle_key("c");
        assert_eq!(manager.text_buffer(node_id), "abc");
        assert_eq!(manager.caret_position(node_id), 3);

        // ArrowLeft moves caret left
        let consumed = manager.handle_key("ArrowLeft");
        assert!(consumed);
        assert_eq!(manager.caret_position(node_id), 2);

        // Insert 'x' at caret position 2 -> "abxc"
        let consumed = manager.handle_key("x");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), "abxc");
        assert_eq!(manager.caret_position(node_id), 3);

        // ArrowRight moves caret right
        let consumed = manager.handle_key("ArrowRight");
        assert!(consumed);
        assert_eq!(manager.caret_position(node_id), 4);

        // ArrowRight at end does nothing
        let consumed = manager.handle_key("ArrowRight");
        assert!(consumed);
        assert_eq!(manager.caret_position(node_id), 4);

        // ArrowLeft multiple times
        manager.handle_key("ArrowLeft");
        manager.handle_key("ArrowLeft");
        manager.handle_key("ArrowLeft");
        manager.handle_key("ArrowLeft");
        assert_eq!(manager.caret_position(node_id), 0);

        // ArrowLeft at start does nothing
        let consumed = manager.handle_key("ArrowLeft");
        assert!(consumed);
        assert_eq!(manager.caret_position(node_id), 0);
    }

    #[test]
    fn test_shell_input_manager_control_and_space_keys() {
        let mut manager = ShellInputManager::new();
        let mut arena = crate::infra::Arena::new();
        let node_id = arena.insert("input");

        manager.focus(node_id);

        // "Space" named key
        let consumed = manager.handle_key("Space");
        assert!(consumed);
        assert_eq!(manager.text_buffer(node_id), " ");

        // Control / submit keys should not be consumed by text input routing
        let consumed = manager.handle_key("Enter");
        assert!(!consumed);

        let consumed = manager.handle_key("Escape");
        assert!(!consumed);

        let consumed = manager.handle_key("Tab");
        assert!(!consumed);

        let consumed = manager.handle_key("ArrowUp");
        assert!(!consumed);

        assert_eq!(manager.text_buffer(node_id), " ");
    }

    #[test]
    fn test_shell_input_manager_unfocused_routing() {
        let mut manager = ShellInputManager::new();
        // Routing when nothing is focused returns false
        let consumed = manager.handle_key("a");
        assert!(!consumed);
    }

    #[test]
    fn test_shell_input_manager_caret_geometry() {
        let mut manager = ShellInputManager::new();
        let mut arena = crate::infra::Arena::new();
        let node_id_1 = arena.insert("input1");
        let node_id_2 = arena.insert("input2");

        manager.focus(node_id_1);
        manager.handle_key("H");
        manager.handle_key("i");

        // Correct node caret geometry
        let geom = manager.calculate_caret_geometry(node_id_1, 100.0, 50.0, 20.0, 15.5);
        assert!(geom.is_some());
        let g = geom.unwrap();
        assert_eq!(g.node_id, node_id_1);
        assert_eq!(g.char_index, 2);
        assert_eq!(g.x, 115.5);
        assert_eq!(g.y, 50.0);
        assert_eq!(g.width, 1.5);
        assert_eq!(g.height, 20.0);

        // Unfocused node caret geometry should be None
        let geom_unfocused = manager.calculate_caret_geometry(node_id_2, 100.0, 50.0, 20.0, 15.5);
        assert!(geom_unfocused.is_none());
    }

    #[test]
    fn test_input_event_triggers_redraw() {
        let click_event = InputEvent::Click { x: 10.0, y: 20.0 };
        assert!(input_event_triggers_redraw(&click_event));

        let key_event = InputEvent::Key {
            key: "a".to_string(),
        };
        assert!(input_event_triggers_redraw(&key_event));
    }
}
