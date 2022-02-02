use crate::*;

/// Indicates when to wrap the current line to a new line. See also [`Direction`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LineWrap {
    /// Never wrap to a new line.
    None,

    /// Wrap to a new line when the available width is exhausted.
    Overflow,
}
impl LineWrap {
    /// TODO(JP): Replace these with LineWrap::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: LineWrap = LineWrap::None;
}
impl Default for LineWrap {
    fn default() -> Self {
        LineWrap::DEFAULT
    }
}

/// Configure how a [`CxLayoutBox`] is going to walk, typically bounded by the
/// dimensions of a parent [`CxLayoutBox`].
#[derive(Copy, Clone, Debug)]
pub(crate) struct Layout {
    /// See [`LayoutSize`].
    pub layout_size: LayoutSize,
    /// See [`Padding`].
    pub padding: Padding,
    /// See [`Direction`].
    pub direction: Direction,
    /// See [`LineWrap`].
    pub line_wrap: LineWrap,
    /// Absolutely position by overriding the [`CxLayoutBox::origin`] with (0,0) instead of using the parent's
    /// current position.
    pub absolute: bool,
    /// Override the maximum size of the [`Window`]/[`Pass`]. Should typically
    /// not be used; instead set [`CxLayoutBox::width`] and [`CxLayoutBox::height`]
    /// through [`Layout::layout_size`].
    pub abs_size: Option<Vec2>,
}

impl Layout {
    /// TODO(JP): Replace these with Layout::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Layout = Layout {
        layout_size: LayoutSize::DEFAULT,
        padding: Padding::DEFAULT,
        direction: Direction::DEFAULT,
        line_wrap: LineWrap::DEFAULT,
        absolute: false,
        abs_size: None,
    };
}

impl Default for Layout {
    fn default() -> Self {
        Layout::DEFAULT
    }
}

/// Determines how a [`CxLayoutBox`] should walk. Can be applied to a new [`CxLayoutBox`]
/// through [`Layout::layout_size`], or directly to move an existing [`CxLayoutBox`] by
/// using [`Cx::add_box`].
#[derive(Copy, Clone, Debug)]
pub struct LayoutSize {
    pub width: Width,
    pub height: Height,
}

impl LayoutSize {
    /// TODO(JP): Replace these with Align::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: LayoutSize = LayoutSize { width: Width::DEFAULT, height: Height::DEFAULT };
    pub const FILL: LayoutSize = LayoutSize { width: Width::Fill, height: Height::Fill };

    pub const fn new(w: Width, h: Height) -> Self {
        Self { width: w, height: h }
    }
}
impl Default for LayoutSize {
    fn default() -> Self {
        LayoutSize::DEFAULT
    }
}

/// The direction in which the [`CxLayoutBox`] should walk. It will typically walk
/// in a straight line in this direction. E.g. when walking to [`Direction::Right`],
/// it will only walk horizontally, not vertically, until it hits the [`CxLayoutBox::width`],
/// at which point it will wrap around using [`LineWrap`], based on the maximum
/// height of widgets that have been drawn so far, which is registered in
/// [`CxLayoutBox::biggest`].
///
/// TODO(JP): This line wrapping behavior makes sense for [`Direction::Right`],
/// but not so much for [`Direction::Down`].. Maybe we should split [`CxLayoutBox`]
/// into different kinds of behavior?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Down,
}
impl Direction {
    /// TODO(JP): Replace these with Direction::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Direction = Direction::Right;
}
impl Default for Direction {
    fn default() -> Self {
        Direction::DEFAULT
    }
}

/// Different ways in which a [`LayoutSize`] can get a width.
///
/// TODO(JP): See [`Height::DEFAULT`] for a related TODO.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Width {
    /// Fill up as much of the available space as possible.
    Fill,
    /// Use a fixed width.
    Fix(f32),
    /// Will defer computation of [`CxLayoutBox::width`] by setting it to [`f32::NAN`],
    /// and only properly computing it later on.
    ///
    /// TODO(JP): This can also be passed into [`Cx::add_box`] but there it
    /// makes no sense!
    Compute,
    /// Fill up as much of the available space as possible up to provided width
    FillUntil(f32),
}
impl Width {
    /// TODO(JP): Replace these with Width::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Width = Width::Fill;
}
impl Default for Width {
    fn default() -> Self {
        Width::Fill
    }
}

/// Different ways in which a [`LayoutSize`] can get a height.
///
/// See [`Width`] for more documentation, since it's analogous.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Height {
    /// See [`Width::Fill`].
    Fill,
    /// See [`Width::Fix`].
    Fix(f32),
    /// See [`Width::Compute`].
    Compute,
    /// See [`Width::FillUntil`],
    FillUntil(f32),
}
impl Height {
    /// TODO(JP): [`Height::Fill`] might be a bad default, because if you use
    /// [`Direction::Down`] it will push out everything out it below.
    /// HTML/CSS uses something more like [`Height::Compute`] by default for height,
    /// and only [`Height::Fill`] for width (for block-layout elements).
    ///
    /// TODO(JP): Replace these with Height::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Height = Height::Fill;
}
impl Default for Height {
    fn default() -> Self {
        Height::Fill
    }
}

/// Defines how elements on [`Cx::layout_box_align_list`] should be moved horizontally
pub(crate) struct AlignX(pub f32);

impl AlignX {
    // Note: LEFT is the default so not needed as explicit option
    pub(crate) const CENTER: AlignX = AlignX(0.5);
    #[allow(dead_code)]
    pub(crate) const RIGHT: AlignX = AlignX(1.0);
}

/// Defines how elements on [`Cx::layout_box_align_list`] should be moved vertically
pub(crate) struct AlignY(pub f32);

impl AlignY {
    // Note: TOP is the default so not needed as explicit option
    pub(crate) const CENTER: AlignY = AlignY(0.5);
    #[allow(dead_code)]
    pub(crate) const BOTTOM: AlignY = AlignY(1.0);
}
