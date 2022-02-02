pub mod color;
mod point;
mod range;
mod rect;
mod size;
pub mod string;

pub use color::*;
pub use point::*;
pub use range::*;
pub use rect::*;
pub use size::*;

pub type LogSeverity = zaplib_cef_sys::cef_log_severity_t;
pub type PaintElementType = zaplib_cef_sys::cef_paint_element_type_t;
pub type TextInputMode = zaplib_cef_sys::cef_text_input_mode_t;
pub type DragOperationsMask = zaplib_cef_sys::cef_drag_operations_mask_t;
pub type ThreadId = zaplib_cef_sys::cef_thread_id_t;
pub type ProcessId = zaplib_cef_sys::cef_process_id_t;
