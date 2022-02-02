use zaplib_cef::CefColor;
use zaplib_shader_compiler::math::Vec4;

pub fn vec4_to_cef_color(color: &Vec4) -> CefColor {
    fn normalize_to_u8(value: f32) -> u8 {
        (value * 255.0) as u8
    }
    CefColor::from_argb(normalize_to_u8(color.w), normalize_to_u8(color.x), normalize_to_u8(color.y), normalize_to_u8(color.z))
}
