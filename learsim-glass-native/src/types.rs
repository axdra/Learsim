//! Small shared value types passed between the async tasks and the render loop.

use std::sync::{Arc, Mutex};

/// A decoded panel frame, ready to upload to an SDL texture (RGB24).
pub struct DecodedFrame {
    pub width: u32,
    pub height: u32,
    /// `width * height * 3` bytes, RGB order.
    pub rgb: Vec<u8>,
}

/// The most recent decoded frame from the glassout client. The render loop
/// takes it; the client overwrites it (latest wins, old frames dropped).
pub type LatestFrame = Arc<Mutex<Option<DecodedFrame>>>;

/// A tap/click, in the *source panel's* pixel coordinate space, to forward to
/// the engine as a `panel.input` message.
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub x: u32,
    pub y: u32,
}
