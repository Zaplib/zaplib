use zaplib_shader_compiler::math::Rect;

/// Enum to encapsulate various events that happens during draw call
#[derive(Clone)]
pub enum DebugLog {
    /// For cases when cx.end_box() is getting called
    EndBox { rect: Rect },
}
