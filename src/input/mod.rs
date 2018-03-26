pub mod consumer;
pub mod producer;

use cgmath::*;
pub use glutin::VirtualKeyCode as KeyCode;

/// These are represented as an enumeration to preserve ordering when stored
/// in a vector and read sequentially.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum InputFrame {
    // Represents keyboard events.
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),

    // Represents cursor events.
    CursorPressed(CursorButton, Vector2<f32>),
    CursorReleased(CursorButton, Vector2<f32>),
    CursorLeft,
    CursorEntered,
}

/// Describes the cursor button being manipulated.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum CursorButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Describes the new bounds the window has been resized to.
#[derive(Copy, Clone)]
pub struct ResizeMessage {
    pub width: u32,
    pub height: u32,
}
