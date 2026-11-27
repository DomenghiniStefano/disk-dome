//! A Slint platform with no window at all: Slint's software renderer draws into a buffer in memory.
//! Used by the measurements and the tests, so nothing appears on screen.

use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{LogicalPosition, PhysicalSize, PlatformError, SharedString};

struct HeadlessPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for HeadlessPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}

/// The off-screen window and the buffer it is drawn into.
pub struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    width: usize,
    height: usize,
    pixels: Vec<PremultipliedRgbaColor>,
}

impl Headless {
    /// Installs the headless platform for this thread. Must run before any component is created.
    pub fn install(width: u32, height: u32) -> Result<Self, PlatformError> {
        // Every frame is drawn in full: the worst case, and what a fast scroll causes anyway.
        let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        slint::platform::set_platform(Box::new(HeadlessPlatform {
            window: window.clone(),
        }))
        .map_err(|error| PlatformError::Other(error.to_string()))?;
        window.set_size(PhysicalSize::new(width, height));
        Ok(Self {
            window,
            width: width as usize,
            height: height as usize,
            pixels: vec![PremultipliedRgbaColor::default(); width as usize * height as usize],
        })
    }

    /// Runs timers, then lays out and draws a frame if anything changed. Returns how long the layout and
    /// drawing took (`None` when nothing needed drawing).
    pub fn frame(&mut self) -> Option<Duration> {
        slint::platform::update_timers_and_animations();
        let start = Instant::now();
        let width = self.width;
        let pixels = &mut self.pixels;
        let drawn = self.window.draw_if_needed(|renderer| {
            renderer.render(pixels, width);
        });
        drawn.then(|| start.elapsed())
    }

    /// Forces the next `frame()` to draw even if nothing changed.
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn press_key(&self, key: slint::platform::Key) {
        let text = SharedString::from(key);
        self.window
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        self.window
            .dispatch_event(WindowEvent::KeyReleased { text });
    }

    pub fn click(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let button = slint::platform::PointerEventButton::Left;
        self.window
            .dispatch_event(WindowEvent::PointerMoved { position });
        self.window
            .dispatch_event(WindowEvent::PointerPressed { position, button });
        self.window
            .dispatch_event(WindowEvent::PointerReleased { position, button });
    }

    /// Presses the left button at a point and keeps it held: a drag or a press that repeats, ended by `release`.
    pub fn press(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let button = slint::platform::PointerEventButton::Left;
        self.window
            .dispatch_event(WindowEvent::PointerMoved { position });
        self.window
            .dispatch_event(WindowEvent::PointerPressed { position, button });
    }

    /// Lets the left button go at a point.
    pub fn release(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let button = slint::platform::PointerEventButton::Left;
        self.window
            .dispatch_event(WindowEvent::PointerReleased { position, button });
    }

    /// A click while a modifier is held (Shift+click).
    pub fn click_with(&self, modifier: slint::platform::Key, x: f32, y: f32) {
        let held = SharedString::from(modifier);
        self.window
            .dispatch_event(WindowEvent::KeyPressed { text: held.clone() });
        self.click(x, y);
        self.window
            .dispatch_event(WindowEvent::KeyReleased { text: held });
    }

    /// A key pressed while a modifier is held (Shift+F10).
    pub fn press_key_with(&self, modifier: slint::platform::Key, key: slint::platform::Key) {
        let held = SharedString::from(modifier);
        self.window
            .dispatch_event(WindowEvent::KeyPressed { text: held.clone() });
        self.press_key(key);
        self.window
            .dispatch_event(WindowEvent::KeyReleased { text: held });
    }

    /// Types text, one key per character (a menu's type-ahead).
    pub fn type_text(&self, text: &str) {
        for c in text.chars() {
            let text = SharedString::from(c.to_string().as_str());
            self.window
                .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
            self.window
                .dispatch_event(WindowEvent::KeyReleased { text });
        }
    }

    /// Moves the pointer to a point, without pressing.
    pub fn hover(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        self.window
            .dispatch_event(WindowEvent::PointerMoved { position });
    }

    /// A right click at a point: what opens a context menu.
    pub fn right_click(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let button = slint::platform::PointerEventButton::Right;
        self.window
            .dispatch_event(WindowEvent::PointerMoved { position });
        self.window
            .dispatch_event(WindowEvent::PointerPressed { position, button });
        self.window
            .dispatch_event(WindowEvent::PointerReleased { position, button });
    }

    /// A mouse wheel or touchpad event at a point; a negative `delta_y` scrolls down.
    pub fn wheel(&self, x: f32, y: f32, delta_y: f32) {
        let position = LogicalPosition::new(x, y);
        self.window
            .dispatch_event(WindowEvent::PointerMoved { position });
        self.window.dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: 0.0,
            delta_y,
        });
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// One pixel of the last frame as red, green, blue and alpha (premultiplied); transparent outside the window.
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let pixel = if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            PremultipliedRgbaColor::default()
        };
        [pixel.red, pixel.green, pixel.blue, pixel.alpha]
    }

    /// The last frame as 24-bit BMP bytes (bottom-up rows, BGR), for a look at what was drawn.
    pub fn bmp(&self) -> Vec<u8> {
        let row_bytes = (self.width * 3).div_ceil(4) * 4;
        let image_bytes = row_bytes * self.height;
        let mut out = Vec::with_capacity(54 + image_bytes);
        let file_size = (54 + image_bytes) as u32;
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(self.width as i32).to_le_bytes());
        out.extend_from_slice(&(self.height as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&[0u8; 24]);
        for y in (0..self.height).rev() {
            let line = &self.pixels[y * self.width..(y + 1) * self.width];
            for pixel in line {
                // Composite over white: the buffer is premultiplied.
                let blend = |channel: u8| channel.saturating_add(255 - pixel.alpha);
                out.extend_from_slice(&[blend(pixel.blue), blend(pixel.green), blend(pixel.red)]);
            }
            out.resize(out.len() + row_bytes - self.width * 3, 0);
        }
        out
    }
}
