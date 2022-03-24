use crate::util::PrettyPrintedFloat;
use std::f32::consts::PI;
use std::f32::EPSILON;
use std::fmt;
use std::ops;

pub fn f32_from_lerp(from: f32, to: f32, t: f32) -> f32 {
    from * (1.0 - t) + to * t
}

/// 4x4 matrix; very common in graphics programming.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
#[repr(C)]
pub struct Mat4 {
    pub v: [f32; 16],
}

/// A common type of transformation, that includes a rotation ([`Transform::orientation`])
/// and translation ([`Transform::position`]).
///
/// TODO(JP): Maybe rename orientation/position to rotation/translation?
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Transform {
    pub orientation: Quat,
    pub position: Vec3,
}

impl Transform {
    pub fn to_mat4(&self) -> Mat4 {
        let q = self.orientation;
        let t = self.position;
        Mat4 {
            v: [
                (1.0 - 2.0 * q.b * q.b - 2.0 * q.c * q.c),
                (2.0 * q.a * q.b - 2.0 * q.c * q.d),
                (2.0 * q.a * q.c + 2.0 * q.b * q.d),
                0.0,
                (2.0 * q.a * q.b + 2.0 * q.c * q.d),
                (1.0 - 2.0 * q.a * q.a - 2.0 * q.c * q.c),
                (2.0 * q.b * q.c - 2.0 * q.a * q.d),
                0.0,
                (2.0 * q.a * q.c - 2.0 * q.b * q.d),
                (2.0 * q.b * q.c + 2.0 * q.a * q.d),
                (1.0 - 2.0 * q.a * q.a - 2.0 * q.b * q.b),
                0.0,
                t.x,
                t.y,
                t.z,
                1.0,
            ],
        }
    }

    pub fn from_lerp(a: Transform, b: Transform, f: f32) -> Self {
        Transform {
            orientation: Quat::from_slerp(a.orientation, b.orientation, f),
            position: Vec3::from_lerp(a.position, b.position, f),
        }
    }

    pub fn from_slerp_orientation(a: Transform, b: Transform, f: f32) -> Self {
        Transform { orientation: Quat::from_slerp(a.orientation, b.orientation, f), position: b.position }
    }
}

/// Convenience function for making a [`Vec2`].
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}
/// Convenience function for making a [`Vec3`].
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}
/// Convenience function for making a [`Vec4`].
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 { x, y, z, w }
}

/// Vector (as in linear algebra, not as in [`Vec`]!) with two elements.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn from_lerp(a: Self, b: Self, f: f32) -> Self {
        Self { x: f32_from_lerp(a.x, b.x, f), y: f32_from_lerp(a.y, b.y, f) }
    }

    pub const fn all(x: f32) -> Vec2 {
        Vec2 { x, y: x }
    }

    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance(&self, other: &Vec2) -> f32 {
        (*other - *self).length()
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn min(&self, other: &Vec2) -> Vec2 {
        vec2(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(&self, other: &Vec2) -> Vec2 {
        vec2(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn clamp(&self, min: &Vec2, max: &Vec2) -> Vec2 {
        vec2(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3 { x: self.x, y: self.y, z: 0.0 }
    }

    pub fn as_array(&self) -> &[f32; 2] {
        unsafe { &*(self as *const _ as *const [f32; 2]) }
    }

    pub fn as_mut_array(&mut self) -> &mut [f32; 2] {
        unsafe { &mut *(self as *mut _ as *mut [f32; 2]) }
    }

    pub fn normalize(&self) -> Vec2 {
        let sz = self.x * self.x + self.y * self.y;
        if sz > 0.0 {
            let sr = 1.0 / sz.sqrt();
            return Vec2 { x: self.x * sr, y: self.y * sr };
        }
        Vec2::default()
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vec2({}, {})", PrettyPrintedFloat(self.x), PrettyPrintedFloat(self.y),)
    }
}

const TORAD: f32 = 0.017_453_292;
const TODEG: f32 = 57.295_78;

/// Vector (as in linear algebra, not as in [`Vec`]!) with three elements.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn from_lerp(a: Self, b: Self, f: f32) -> Self {
        Self { x: f32_from_lerp(a.x, b.x, f), y: f32_from_lerp(a.y, b.y, f), z: f32_from_lerp(a.z, b.z, f) }
    }

    pub const fn all(x: f32) -> Vec3 {
        Vec3 { x, y: x, z: x }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
    }

    pub fn scale(&self, f: f32) -> Vec3 {
        Vec3 { x: self.x * f, y: self.y * f, z: self.z * f }
    }

    pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
        Vec3 { x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x }
    }

    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalize(&self) -> Vec3 {
        let sz = self.x * self.x + self.y * self.y + self.z * self.z;
        if sz > 0.0 {
            let sr = 1.0 / sz.sqrt();
            return Vec3 { x: self.x * sr, y: self.y * sr, z: self.z * sr };
        }
        Vec3::default()
    }

    pub fn distance(&self, other: &Vec3) -> f32 {
        (*other - *self).length()
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn clamp(&self, min: &Vec3, max: &Vec3) -> Vec3 {
        vec3(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z))
    }

    pub fn as_array(&self) -> &[f32; 3] {
        unsafe { &*(self as *const _ as *const [f32; 3]) }
    }

    pub fn as_mut_array(&mut self) -> &mut [f32; 3] {
        unsafe { &mut *(self as *mut _ as *mut [f32; 3]) }
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vec3({}, {}, {})", PrettyPrintedFloat(self.x), PrettyPrintedFloat(self.y), PrettyPrintedFloat(self.z),)
    }
}

/// Vector (as in linear algebra, not as in [`Vec`]!) with four elements.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "vec4({}, {}, {}, {})",
            PrettyPrintedFloat(self.x),
            PrettyPrintedFloat(self.y),
            PrettyPrintedFloat(self.z),
            PrettyPrintedFloat(self.w),
        )
    }
}

impl Vec4 {
    pub fn from_lerp(a: Self, b: Self, f: f32) -> Self {
        Self { x: f32_from_lerp(a.x, b.x, f), y: f32_from_lerp(a.y, b.y, f), z: f32_from_lerp(a.z, b.z, f), w: f32_from_lerp(a.w, b.w, f) }
    }

    pub const fn all(v: f32) -> Self {
        Self { x: v, y: v, z: v, w: v }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3 { x: self.x, y: self.y, z: self.z }
    }

    pub fn dot(&self, other: Vec4) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn mix(a: Vec4, b: Vec4, f: f32) -> Vec4 {
        let nf = 1.0 - f;
        Vec4 { x: nf * a.x + f * b.x, y: nf * a.y + f * b.y, z: nf * a.z + f * b.z, w: nf * a.w + f * b.w }
    }

    pub fn is_equal_enough(&self, other: &Vec4) -> bool {
        (self.x - other.x).abs() < 0.0001
            && (self.y - other.y).abs() < 0.0001
            && (self.z - other.z).abs() < 0.0001
            && (self.w - other.w).abs() < 0.0001
    }

    pub fn from_hsva(hsv: Vec4) -> Vec4 {
        fn mix(x: f32, y: f32, t: f32) -> f32 {
            x + (y - x) * t
        }
        fn clamp(x: f32, mi: f32, ma: f32) -> f32 {
            if x < mi {
                mi
            } else if x > ma {
                ma
            } else {
                x
            }
        }
        fn fract(x: f32) -> f32 {
            x.fract()
        }
        fn abs(x: f32) -> f32 {
            x.abs()
        }
        Vec4 {
            x: hsv.z * mix(1.0, clamp(abs(fract(hsv.x + 1.0) * 6.0 - 3.0) - 1.0, 0.0, 1.0), hsv.y),
            y: hsv.z * mix(1.0, clamp(abs(fract(hsv.x + 2.0 / 3.0) * 6.0 - 3.0) - 1.0, 0.0, 1.0), hsv.y),
            z: hsv.z * mix(1.0, clamp(abs(fract(hsv.x + 1.0 / 3.0) * 6.0 - 3.0) - 1.0, 0.0, 1.0), hsv.y),
            w: 1.0,
        }
    }

    pub fn to_hsva(&self) -> Vec4 {
        let pc = self.y < self.z; //step(c[2],c[1])
        let p0 = if pc { self.z } else { self.y }; //mix(c[2],c[1],pc)
        let p1 = if pc { self.y } else { self.z }; //mix(c[1],c[2],pc)
        let p2 = if pc { -1.0 } else { 0.0 }; //mix(-1,0,pc)
        let p3 = if pc { 2.0 / 3.0 } else { -1.0 / 3.0 }; //mix(2/3,-1/3,pc)

        let qc = self.x < p0; //step(p0, c[0])
        let q0 = if qc { p0 } else { self.x }; //mix(p0, c[0], qc)
        let q1 = p1;
        let q2 = if qc { p3 } else { p2 }; //mix(p3, p2, qc)
        let q3 = if qc { self.x } else { p0 }; //mix(c[0], p0, qc)

        let d = q0 - q3.min(q1);
        let e = 1.0e-10;
        Vec4 { x: (q2 + (q3 - q1) / (6.0 * d + e)).abs(), y: d / (q0 + e), z: q0, w: self.w }
    }

    pub fn from_u32(val: u32) -> Vec4 {
        Vec4 {
            x: ((val >> 24) & 0xff) as f32 / 255.0,
            y: ((val >> 16) & 0xff) as f32 / 255.0,
            z: ((val >> 8) & 0xff) as f32 / 255.0,
            w: ((val >> 0) & 0xff) as f32 / 255.0,
        }
    }

    pub fn to_hex_string(&self) -> String {
        fn int_to_hex(d: u8) -> char {
            if d >= 10 {
                return (d + 55) as char;
            }
            (d + 48) as char
        }

        let r = (self.x * 255.0) as u8;
        let g = (self.y * 255.0) as u8;
        let b = (self.z * 255.0) as u8;
        let mut out = String::new();
        out.push(int_to_hex((r >> 4) & 0xf));
        out.push(int_to_hex((r) & 0xf));
        out.push(int_to_hex((g >> 4) & 0xf));
        out.push(int_to_hex((g) & 0xf));
        out.push(int_to_hex((b >> 4) & 0xf));
        out.push(int_to_hex((b) & 0xf));
        out
    }

    pub fn color(value: &str) -> Vec4 {
        if let Ok(val) = Self::from_hex_str(value) {
            val
        } else {
            Vec4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 }
        }
    }

    fn from_hex_str(hex: &str) -> Result<Vec4, ()> {
        let bytes = hex.as_bytes();
        if bytes[0] == b'#' {
            Self::from_hex_bytes(&bytes[1..])
        } else {
            Self::from_hex_bytes(bytes)
        }
    }

    pub fn from_hex_bytes(bytes: &[u8]) -> Result<Vec4, ()> {
        fn hex_to_int(c: u32) -> Result<u32, ()> {
            if (48..=57).contains(&c) {
                return Ok(c - 48);
            }
            if (65..=70).contains(&c) {
                return Ok(c - 65 + 10);
            }
            if (97..=102).contains(&c) {
                return Ok(c - 97 + 10);
            }
            Err(())
        }

        match bytes.len() {
            1 => {
                // #w
                let val = hex_to_int(bytes[0] as u32)? as f32 / 15.0;
                return Ok(vec4(val, val, val, 1.0));
            }
            2 => {
                //#ww
                let w = ((hex_to_int(bytes[0] as u32)? << 4) + hex_to_int(bytes[1] as u32)?) as f32 / 255.0;
                return Ok(vec4(w, w, w, 1.0));
            }
            3 => {
                // #rgb
                let r = hex_to_int(bytes[0] as u32)? as f32 / 15.0;
                let g = hex_to_int(bytes[1] as u32)? as f32 / 15.0;
                let b = hex_to_int(bytes[2] as u32)? as f32 / 15.0;
                return Ok(vec4(r, g, b, 1.0));
            }
            4 => {
                // #rgba
                let r = hex_to_int(bytes[0] as u32)? as f32 / 15.0;
                let g = hex_to_int(bytes[1] as u32)? as f32 / 15.0;
                let b = hex_to_int(bytes[2] as u32)? as f32 / 15.0;
                let a = hex_to_int(bytes[3] as u32)? as f32 / 15.0;
                return Ok(vec4(r, g, b, a));
            }
            6 => {
                // #rrggbb
                let r = ((hex_to_int(bytes[0] as u32)? << 4) + hex_to_int(bytes[1] as u32)?) as f32 / 255.0;
                let g = ((hex_to_int(bytes[2] as u32)? << 4) + hex_to_int(bytes[3] as u32)?) as f32 / 255.0;
                let b = ((hex_to_int(bytes[4] as u32)? << 4) + hex_to_int(bytes[5] as u32)?) as f32 / 255.0;
                return Ok(vec4(r, g, b, 1.0));
            }
            8 => {
                // #rrggbbaa
                let r = ((hex_to_int(bytes[0] as u32)? << 4) + hex_to_int(bytes[1] as u32)?) as f32 / 255.0;
                let g = ((hex_to_int(bytes[2] as u32)? << 4) + hex_to_int(bytes[3] as u32)?) as f32 / 255.0;
                let b = ((hex_to_int(bytes[4] as u32)? << 4) + hex_to_int(bytes[5] as u32)?) as f32 / 255.0;
                let a = ((hex_to_int(bytes[6] as u32)? << 4) + hex_to_int(bytes[7] as u32)?) as f32 / 255.0;
                return Ok(vec4(r, g, b, a));
            }
            _ => (),
        }
        Err(())
    }

    pub fn clamp(&self, min: &Vec4, max: &Vec4) -> Vec4 {
        vec4(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z), self.w.clamp(min.w, max.w))
    }

    pub fn as_array(&self) -> &[f32; 4] {
        unsafe { &*(self as *const _ as *const [f32; 4]) }
    }

    pub fn as_mut_array(&mut self) -> &mut [f32; 4] {
        unsafe { &mut *(self as *mut _ as *mut [f32; 4]) }
    }
}

/// Represents an (axis-aligned) rectangle. Axis-aligned means that you can't
/// rotate it.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn translate(self, pos: Vec2) -> Rect {
        Rect { pos: self.pos + pos, size: self.size }
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        pos.x >= self.pos.x && pos.x <= self.pos.x + self.size.x && pos.y >= self.pos.y && pos.y <= self.pos.y + self.size.y
    }

    pub fn intersects(&self, r: Rect) -> bool {
        !(r.pos.x > self.pos.x + self.size.x
            || r.pos.x + r.size.x < self.pos.x
            || r.pos.y > self.pos.y + self.size.y
            || r.pos.y + r.size.y < self.pos.y)
    }

    /// This returns the [`Rect`] for if you'd add padding all around the given [`Rect`].
    ///
    /// This means that the `pos` will move according to the left/top padding, and the size will be adjusted
    /// based on the sum of the vertical/horizontal paddings.
    ///
    /// If you just want to adjust the size while keeping `pos` the same, you can simply add the desired
    /// dimensions to `size`.
    pub fn add_padding(self, padding: Padding) -> Self {
        Self {
            pos: Vec2 { x: self.pos.x - padding.l, y: self.pos.y - padding.t },
            size: Vec2 { x: self.size.x + padding.l + padding.r, y: self.size.y + padding.t + padding.b },
        }
    }
}

/// Inner padding dimensions that should be applied on top of a [`Rect`] or other
/// object that defines dimensions.
///
/// TODO(JP): these values can be negative, which can be quite confusing, but we
/// seem to actually honor that in the layout boxes code. Might be good to look into that
/// and see if we should forbid that or not (we seem to never actually do that yet).
#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}
impl Padding {
    pub const ZERO: Padding = Padding { l: 0.0, t: 0.0, r: 0.0, b: 0.0 };

    /// TODO(JP): Replace these with Padding::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Padding = Padding::ZERO;

    pub const fn all(v: f32) -> Padding {
        Padding { l: v, t: v, r: v, b: v }
    }

    pub const fn left(v: f32) -> Padding {
        Padding { l: v, ..Padding::ZERO }
    }

    pub const fn top(v: f32) -> Padding {
        Padding { t: v, ..Padding::ZERO }
    }

    pub const fn right(v: f32) -> Padding {
        Padding { r: v, ..Padding::ZERO }
    }

    pub const fn bottom(v: f32) -> Padding {
        Padding { b: v, ..Padding::ZERO }
    }

    /// Helper function to set vertical and horizontal padding
    /// This is a common case when top=bottom left=right
    pub const fn vh(v: f32, h: f32) -> Padding {
        Padding { l: h, r: h, t: v, b: v }
    }
}
impl Default for Padding {
    fn default() -> Self {
        Padding::DEFAULT
    }
}

/// [Quaternion](https://en.wikipedia.org/wiki/Quaternion); used for rotations.
///
/// Let's give it up for [Hamilton](https://www.youtube.com/watch?v=SZXHoWwBcDc).
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Quat {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Quat {
    pub fn dot(&self, other: Quat) -> f32 {
        self.a * other.a + self.b * other.b + self.c * other.c + self.d * other.d
    }

    pub fn neg(&self) -> Quat {
        Quat { a: -self.a, b: -self.b, c: -self.c, d: -self.d }
    }

    pub fn get_angle_with(&self, other: Quat) -> f32 {
        let dot = self.dot(other);
        (2.0 * dot * dot - 1.0).acos() * TODEG
    }

    pub fn from_slerp(n: Quat, mut m: Quat, t: f32) -> Quat {
        // calc cosine
        let mut cosom = n.dot(m);
        // adjust signs (if necessary)
        if cosom < 0.0 {
            cosom = -cosom;
            m = m.neg();
        }
        // calculate coefficients
        let (scale0, scale1) = if 1.0 - cosom > 0.000001 {
            // standard case (slerp)
            let omega = cosom.acos();
            let sinom = omega.sin();
            (((1.0 - t) * omega).sin() / sinom, (t * omega).sin() / sinom)
        } else {
            (1.0 - t, t)
        };
        // calculate final values
        (Quat {
            a: scale0 * n.a + scale1 * m.a,
            b: scale0 * n.b + scale1 * m.b,
            c: scale0 * n.c + scale1 * m.c,
            d: scale0 * m.d + scale1 * m.d,
        })
        .normalized()
    }

    /// Creates a [`Quat`] from a given rotation axis and angle (in radians)
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let theta = 0.5 * angle;
        let sin_theta = theta.sin();
        Quat { a: sin_theta * axis.x, b: sin_theta * axis.y, c: sin_theta * axis.z, d: theta.cos() }
    }

    /// Creates a [`Quat`] representing the shortest rotation from one
    /// [`Vec3`] to another.
    pub fn rotation_to(a: Vec3, b: Vec3) -> Quat {
        let dot = a.dot(b);
        if dot < -(1.0 - EPSILON) {
            // Input vectors are pointing in opposite directions
            // Rotate using an arbitrary vector
            const UNIT_X: Vec3 = vec3(1.0, 0.0, 0.0);
            const UNIT_Y: Vec3 = vec3(0.0, 1.0, 0.0);
            let mut axis = Vec3::cross(UNIT_X, a);
            if axis.length() < EPSILON {
                axis = Vec3::cross(UNIT_Y, a);
            }
            axis.normalize();
            Quat::from_axis_angle(axis, PI)
        } else if dot > (1.0 - EPSILON) {
            // Input vectors have the same orientation
            Quat { a: 0., b: 0., c: 0., d: 1. }
        } else {
            let axis = Vec3::cross(a, b);
            Quat { a: axis.x, b: axis.y, c: axis.z, d: (1. + dot) }.normalized()
        }
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&mut self) -> Quat {
        let len = self.length();
        Quat { a: self.a / len, b: self.b / len, c: self.c / len, d: self.d / len }
    }

    /// Transforms a [`Vec3`] by rotating it
    pub fn rotate_vec(&self, a: Vec3) -> Vec3 {
        let qv = vec3(self.a, self.b, self.c);
        let mut u = Vec3::cross(qv, a);
        let mut v = Vec3::cross(qv, u);
        u *= 2.0 * self.d;
        v *= 2.0;
        a + u + v
    }

    /// Rotates a [`Quat`] by the given angle (in radians) in the Y-axis
    pub fn rotate_y(&self, angle: f32) -> Quat {
        let theta = 0.5 * angle;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        Quat {
            a: self.a * cos_theta - self.c * sin_theta,
            b: self.b * cos_theta + self.d * sin_theta,
            c: self.c * cos_theta + self.a * sin_theta,
            d: self.d * cos_theta - self.b * sin_theta,
        }
    }
}

impl Mat4 {
    pub fn identity() -> Mat4 {
        Mat4 { v: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] }
    }

    pub fn txyz_s_ry_rx_txyz(t1: Vec3, s: f32, ry: f32, rx: f32, t2: Vec3) -> Mat4 {
        let cx = f32::cos(rx * TORAD);
        let cy = f32::cos(ry * TORAD);
        //let cz = f32::cos(r.z * TORAD);
        let sx = f32::sin(rx * TORAD);
        let sy = f32::sin(ry * TORAD);
        //let sz = f32::sin(r.z * TORAD);
        // y first, then x, then z

        // Y
        // |  cy,  0,  sy  |
        // |  0,   1,  0  |
        // | -sy,  0,  cy  |

        // X:
        // |  1,  0,  0  |
        // |  0,  cx, -sx  |
        // |  0,  sx,  cx  |

        // Z:
        // |  cz, -sz,  0  |
        // |  sz,  cz,  0  |
        // |  0,    0,  1  |

        // X * Y
        // | cy,           0,    sy |
        // | -sx*-sy,     cx,   -sx*cy  |
        // | -sy * cx,    sx,  cx*cy  |

        // Z * X * Y
        // | cz * cy + -sz * -sx *-sy,   -sz * cx,    sy *cz + -sz * -sx * cy |
        // | sz * cy + -sx*-sy * cz,     sz * cx,   sy * sz + cz * -sz * cy  |
        // | -sy * cx,    sx,  cx*cy  |

        // Y * X * Z
        // | c*c,  c, s*s   |
        // |   0,  c,  -s   |
        // |  -s,  c*s, c*c |

        /*
        let m0 = s * (cz * cy + (-sz) * (-sx) *(-sy));
        let m1 = s * (-sz * cx);
        let m2 = s * (sy *cz + (-sz) * (-sx) * cy);

        let m4 = s * (sz * cy + (-sx)*(-sy) * cz);
        let m5 = s * (sz * cx);
        let m6 = s * (sy * sz + cz * (-sx) * cy);

        let m8 = s * (-sy*cx);
        let m9 = s * (sx);
        let m10 = s * (cx * cy);
        */

        let m0 = s * (cy);
        let m1 = s * (0.0);
        let m2 = s * (sy);

        let m4 = s * (-sx * -sy);
        let m5 = s * (cx);
        let m6 = s * (-sx * cy);

        let m8 = s * (-sy * cx);
        let m9 = s * (sx);
        let m10 = s * (cx * cy);

        /*
        let m0 = s * (cy * cz + sx * sy * sz);
        let m1 = s * (-sz * cy + cz * sx * sy);
        let m2 = s * (sy * cx);

        let m4 = s * (sz * cx);
        let m5 = s * (cx * cz);
        let m6 = s * (-sx);

        let m8 = s * (-sy * cz + cy * sx * sz);
        let m9 = s * (sy * sz + cy * sx * cz);
        let m10 = s * (cx * cy);
        */
        Mat4 {
            v: [
                m0,
                m4,
                m8,
                0.0,
                m1,
                m5,
                m9,
                0.0,
                m2,
                m6,
                m10,
                0.0,
                t2.x + (m0 * t1.x + m1 * t1.y + m2 * t1.z),
                t2.y + (m4 * t1.x + m5 * t1.y + m6 * t1.z),
                t2.z + (m8 * t1.x + m9 * t1.y + m10 * t1.z),
                1.0,
            ],
        }
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / f32::tan(fov_y * TORAD / 2.0);
        let nf = 1.0 / (near - far);
        Mat4 {
            v: [
                f / aspect,
                0.0,
                0.0,
                0.0,
                0.0,
                f,
                0.0,
                0.0,
                0.0,
                0.0,
                (far + near) * nf,
                -1.0,
                0.0,
                0.0,
                (2.0 * far * near) * nf,
                0.0,
            ],
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Mat4 {
        Mat4 { v: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0] }
    }

    pub fn scaled_translation(s: f32, x: f32, y: f32, z: f32) -> Mat4 {
        Mat4 { v: [s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, s, 0.0, x, y, z, 1.0] }
    }

    pub fn rotation(rx: f32, ry: f32, rz: f32) -> Mat4 {
        const TORAD: f32 = 0.017_453_292;
        let cx = f32::cos(rx * TORAD);
        let cy = f32::cos(ry * TORAD);
        let cz = f32::cos(rz * TORAD);
        let sx = f32::sin(rx * TORAD);
        let sy = f32::sin(ry * TORAD);
        let sz = f32::sin(rz * TORAD);
        let m0 = cy * cz + sx * sy * sz;
        let m1 = -sz * cy + cz * sx * sy;
        let m2 = sy * cx;
        let m4 = sz * cx;
        let m5 = cx * cz;
        let m6 = -sx;
        let m8 = -sy * cz + cy * sx * sz;
        let m9 = sy * sz + cy * sx * cz;
        let m10 = cx * cy;
        Mat4 { v: [m0, m4, m8, 0.0, m1, m5, m9, 0.0, m2, m6, m10, 0.0, 0.0, 0.0, 0.0, 1.0] }
    }

    pub fn ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32, scalex: f32, scaley: f32) -> Mat4 {
        let lr = 1.0 / (left - right);
        let bt = 1.0 / (bottom - top);
        let nf = 1.0 / (near - far);
        /*return Mat4{v:[
            -2.0 * lr * scalex, 0.0, 0.0, (left+right) * lr,
            0.0, -2.0 * bt * scaley, 0.0, (top+bottom) * bt,
            0.0, 0.0, 2.0 * nf, (far + near) * nf,
            0.0, 0.0, 0.0, 1.0
        ]}*/
        Mat4 {
            v: [
                -2.0 * lr * scalex,
                0.0,
                0.0,
                0.0,
                0.0,
                -2.0 * bt * scaley,
                0.0,
                0.0,
                0.0,
                0.0,
                -1.0 * nf,
                0.0,
                (left + right) * lr,
                (top + bottom) * bt,
                0.5 + (far + near) * nf,
                1.0,
            ],
        }
    }

    pub fn transform_vec4(&self, v: Vec4) -> Vec4 {
        let m = &self.v;
        Vec4 {
            x: m[0] * v.x + m[4] * v.y + m[8] * v.z + m[12] * v.w,
            y: m[1] * v.x + m[5] * v.y + m[9] * v.z + m[13] * v.w,
            z: m[2] * v.x + m[6] * v.y + m[10] * v.z + m[14] * v.w,
            w: m[3] * v.x + m[7] * v.y + m[11] * v.z + m[15] * v.w,
        }
    }

    pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
        // this is probably stupid. Programmed JS for too long.
        let a = &a.v;
        let b = &b.v;
        fn d(i: &[f32; 16], x: usize, y: usize) -> f32 {
            i[x + 4 * y]
        }
        Mat4 {
            v: [
                d(a, 0, 0) * d(b, 0, 0) + d(a, 1, 0) * d(b, 0, 1) + d(a, 2, 0) * d(b, 0, 2) + d(a, 3, 0) * d(b, 0, 3),
                d(a, 0, 0) * d(b, 1, 0) + d(a, 1, 0) * d(b, 1, 1) + d(a, 2, 0) * d(b, 1, 2) + d(a, 3, 0) * d(b, 1, 3),
                d(a, 0, 0) * d(b, 2, 0) + d(a, 1, 0) * d(b, 2, 1) + d(a, 2, 0) * d(b, 2, 2) + d(a, 3, 0) * d(b, 2, 3),
                d(a, 0, 0) * d(b, 3, 0) + d(a, 1, 0) * d(b, 3, 1) + d(a, 2, 0) * d(b, 3, 2) + d(a, 3, 0) * d(b, 3, 3),
                d(a, 0, 1) * d(b, 0, 0) + d(a, 1, 1) * d(b, 0, 1) + d(a, 2, 1) * d(b, 0, 2) + d(a, 3, 1) * d(b, 0, 3),
                d(a, 0, 1) * d(b, 1, 0) + d(a, 1, 1) * d(b, 1, 1) + d(a, 2, 1) * d(b, 1, 2) + d(a, 3, 1) * d(b, 1, 3),
                d(a, 0, 1) * d(b, 2, 0) + d(a, 1, 1) * d(b, 2, 1) + d(a, 2, 1) * d(b, 2, 2) + d(a, 3, 1) * d(b, 2, 3),
                d(a, 0, 1) * d(b, 3, 0) + d(a, 1, 1) * d(b, 3, 1) + d(a, 2, 1) * d(b, 3, 2) + d(a, 3, 1) * d(b, 3, 3),
                d(a, 0, 2) * d(b, 0, 0) + d(a, 1, 2) * d(b, 0, 1) + d(a, 2, 2) * d(b, 0, 2) + d(a, 3, 2) * d(b, 0, 3),
                d(a, 0, 2) * d(b, 1, 0) + d(a, 1, 2) * d(b, 1, 1) + d(a, 2, 2) * d(b, 1, 2) + d(a, 3, 2) * d(b, 1, 3),
                d(a, 0, 2) * d(b, 2, 0) + d(a, 1, 2) * d(b, 2, 1) + d(a, 2, 2) * d(b, 2, 2) + d(a, 3, 2) * d(b, 2, 3),
                d(a, 0, 2) * d(b, 3, 0) + d(a, 1, 2) * d(b, 3, 1) + d(a, 2, 2) * d(b, 3, 2) + d(a, 3, 2) * d(b, 3, 3),
                d(a, 0, 3) * d(b, 0, 0) + d(a, 1, 3) * d(b, 0, 1) + d(a, 2, 3) * d(b, 0, 2) + d(a, 3, 3) * d(b, 0, 3),
                d(a, 0, 3) * d(b, 1, 0) + d(a, 1, 3) * d(b, 1, 1) + d(a, 2, 3) * d(b, 1, 2) + d(a, 3, 3) * d(b, 1, 3),
                d(a, 0, 3) * d(b, 2, 0) + d(a, 1, 3) * d(b, 2, 1) + d(a, 2, 3) * d(b, 2, 2) + d(a, 3, 3) * d(b, 2, 3),
                d(a, 0, 3) * d(b, 3, 0) + d(a, 1, 3) * d(b, 3, 1) + d(a, 2, 3) * d(b, 3, 2) + d(a, 3, 3) * d(b, 3, 3),
            ],
        }
    }

    pub fn invert(&self) -> Mat4 {
        let a = &self.v;
        let a00 = a[0];
        let a01 = a[1];
        let a02 = a[2];
        let a03 = a[3];
        let a10 = a[4];
        let a11 = a[5];
        let a12 = a[6];
        let a13 = a[7];
        let a20 = a[8];
        let a21 = a[9];
        let a22 = a[10];
        let a23 = a[11];
        let a30 = a[12];
        let a31 = a[13];
        let a32 = a[14];
        let a33 = a[15];

        let b00 = a00 * a11 - a01 * a10;
        let b01 = a00 * a12 - a02 * a10;
        let b02 = a00 * a13 - a03 * a10;
        let b03 = a01 * a12 - a02 * a11;
        let b04 = a01 * a13 - a03 * a11;
        let b05 = a02 * a13 - a03 * a12;
        let b06 = a20 * a31 - a21 * a30;
        let b07 = a20 * a32 - a22 * a30;
        let b08 = a20 * a33 - a23 * a30;
        let b09 = a21 * a32 - a22 * a31;
        let b10 = a21 * a33 - a23 * a31;
        let b11 = a22 * a33 - a23 * a32;

        // Calculate the determinant
        let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

        if det == 0.0 {
            return Mat4::identity();
        }

        let idet = 1.0 / det;
        Mat4 {
            v: [
                (a11 * b11 - a12 * b10 + a13 * b09) * idet,
                (a02 * b10 - a01 * b11 - a03 * b09) * idet,
                (a31 * b05 - a32 * b04 + a33 * b03) * idet,
                (a22 * b04 - a21 * b05 - a23 * b03) * idet,
                (a12 * b08 - a10 * b11 - a13 * b07) * idet,
                (a00 * b11 - a02 * b08 + a03 * b07) * idet,
                (a32 * b02 - a30 * b05 - a33 * b01) * idet,
                (a20 * b05 - a22 * b02 + a23 * b01) * idet,
                (a10 * b10 - a11 * b08 + a13 * b06) * idet,
                (a01 * b08 - a00 * b10 - a03 * b06) * idet,
                (a30 * b04 - a31 * b02 + a33 * b00) * idet,
                (a21 * b02 - a20 * b04 - a23 * b00) * idet,
                (a11 * b07 - a10 * b09 - a12 * b06) * idet,
                (a00 * b09 - a01 * b07 + a02 * b06) * idet,
                (a31 * b01 - a30 * b03 - a32 * b00) * idet,
                (a20 * b03 - a21 * b01 + a22 * b00) * idet,
            ],
        }
    }

    /// Transpose a matrix
    pub fn transpose(&self) -> Mat4 {
        Mat4 {
            v: [
                self.v[0], self.v[4], self.v[8], self.v[12], self.v[1], self.v[5], self.v[9], self.v[13], self.v[2], self.v[6],
                self.v[10], self.v[14], self.v[3], self.v[7], self.v[11], self.v[15],
            ],
        }
    }

    /// Extracts just the rotation values from a transformation matrix.
    pub fn as_rotation(&self) -> Mat4 {
        Mat4 {
            v: [
                self.v[0], self.v[1], self.v[2], 0., self.v[4], self.v[5], self.v[6], 0., self.v[8], self.v[9], self.v[10], 0.,
                0., 0., 0., 1.,
            ],
        }
    }
}

//------ Vec2 operators

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl ops::Div<Vec2> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x / rhs.x, y: self.y / rhs.y }
    }
}

impl ops::Add<Vec2> for f32 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self + rhs.x, y: self + rhs.y }
    }
}

impl ops::Sub<Vec2> for f32 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self - rhs.x, y: self - rhs.y }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self * rhs.x, y: self * rhs.y }
    }
}

impl ops::Div<Vec2> for f32 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self / rhs.x, y: self / rhs.y }
    }
}

impl ops::Add<f32> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: f32) -> Vec2 {
        Vec2 { x: self.x + rhs, y: self.y + rhs }
    }
}

impl ops::Sub<f32> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: f32) -> Vec2 {
        Vec2 { x: self.x - rhs, y: self.y - rhs }
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 { x: self.x * rhs, y: self.y * rhs }
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2 { x: self.x / rhs, y: self.y / rhs }
    }
}

impl ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl ops::SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}

impl ops::MulAssign<Vec2> for Vec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x = self.x * rhs.x;
        self.y = self.y * rhs.y;
    }
}

impl ops::DivAssign<Vec2> for Vec2 {
    fn div_assign(&mut self, rhs: Vec2) {
        self.x = self.x / rhs.x;
        self.y = self.y / rhs.y;
    }
}

impl ops::AddAssign<f32> for Vec2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x = self.x + rhs;
        self.y = self.y + rhs;
    }
}

impl ops::SubAssign<f32> for Vec2 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x = self.x - rhs;
        self.y = self.y - rhs;
    }
}

impl ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Self {
        Vec2 { x: -self.x, y: -self.y }
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self {
        Vec3 { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl ops::Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Self {
        Vec4 { x: -self.x, y: -self.y, z: -self.z, w: -self.w }
    }
}

//------ Vec3 operators

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z }
    }
}

impl ops::Add<Vec3> for f32 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self + rhs.x, y: self + rhs.y, z: self + rhs.z }
    }
}

impl ops::Sub<Vec3> for f32 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self - rhs.x, y: self - rhs.y, z: self - rhs.z }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z }
    }
}

impl ops::Div<Vec3> for f32 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z }
    }
}

impl ops::Add<f32> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: f32) -> Vec3 {
        Vec3 { x: self.x + rhs, y: self.y + rhs, z: self.z + rhs }
    }
}

impl ops::Sub<f32> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: f32) -> Vec3 {
        Vec3 { x: self.x - rhs, y: self.y - rhs, z: self.z - rhs }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3 { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;
    }
}

impl ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;
    }
}

impl ops::MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.x = self.x * rhs.x;
        self.y = self.y * rhs.y;
        self.z = self.z * rhs.z;
    }
}

impl ops::DivAssign<Vec3> for Vec3 {
    fn div_assign(&mut self, rhs: Vec3) {
        self.x = self.x / rhs.x;
        self.y = self.y / rhs.y;
        self.z = self.z / rhs.z;
    }
}

impl ops::AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        self.x = self.x + rhs;
        self.y = self.y + rhs;
        self.z = self.z + rhs;
    }
}

impl ops::SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x = self.x - rhs;
        self.y = self.y - rhs;
        self.z = self.z - rhs;
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
        self.z = self.z * rhs;
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
        self.z = self.z / rhs;
    }
}

//------ Vec4 operators

impl ops::Add<Vec4> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z, w: self.w + rhs.w }
    }
}

impl ops::Sub<Vec4> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z, w: self.w - rhs.w }
    }
}

impl ops::Mul<Vec4> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z, w: self.w * rhs.w }
    }
}

impl ops::Div<Vec4> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z, w: self.w / rhs.w }
    }
}

impl ops::Add<Vec4> for f32 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self + rhs.x, y: self + rhs.y, z: self + rhs.z, w: self + rhs.z }
    }
}

impl ops::Sub<Vec4> for f32 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self - rhs.x, y: self - rhs.y, z: self - rhs.z, w: self - rhs.z }
    }
}

impl ops::Mul<Vec4> for f32 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z, w: self * rhs.z }
    }
}

impl ops::Div<Vec4> for f32 {
    type Output = Vec4;
    fn div(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z, w: self / rhs.z }
    }
}

impl ops::Add<f32> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: f32) -> Vec4 {
        Vec4 { x: self.x + rhs, y: self.y + rhs, z: self.z + rhs, w: self.w + rhs }
    }
}

impl ops::Sub<f32> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: f32) -> Vec4 {
        Vec4 { x: self.x - rhs, y: self.y - rhs, z: self.z - rhs, w: self.w - rhs }
    }
}

impl ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs, w: self.w * rhs }
    }
}

impl ops::Div<f32> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f32) -> Vec4 {
        Vec4 { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs, w: self.w / rhs }
    }
}

impl ops::AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;
        self.w = self.w + rhs.w;
    }
}

impl ops::SubAssign<Vec4> for Vec4 {
    fn sub_assign(&mut self, rhs: Vec4) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;
        self.w = self.w - rhs.w;
    }
}

impl ops::MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, rhs: Vec4) {
        self.x = self.x * rhs.x;
        self.y = self.y * rhs.y;
        self.z = self.z * rhs.z;
        self.w = self.w * rhs.w;
    }
}

impl ops::DivAssign<Vec4> for Vec4 {
    fn div_assign(&mut self, rhs: Vec4) {
        self.x = self.x / rhs.x;
        self.y = self.y / rhs.y;
        self.z = self.z / rhs.z;
        self.w = self.w / rhs.w;
    }
}

impl ops::AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, rhs: f32) {
        self.x = self.x + rhs;
        self.y = self.y + rhs;
        self.z = self.z + rhs;
        self.w = self.w + rhs;
    }
}

impl ops::SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x = self.x - rhs;
        self.y = self.y - rhs;
        self.z = self.z - rhs;
        self.w = self.w - rhs;
    }
}

impl ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
        self.z = self.z * rhs;
        self.w = self.w * rhs;
    }
}

impl ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
        self.z = self.z / rhs;
        self.w = self.w / rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::math::*;

    #[test]
    fn test_vec4_from_lerp() {
        let a = vec4(1.0, 2.0, 3.0, 4.0);
        let b = vec4(5.0, 6.0, 7.0, 8.0);
        let t = 0.5;
        let c = Vec4::from_lerp(a, b, t);
        assert_eq!(c, vec4(3.0, 4.0, 5.0, 6.0));
    }
}
