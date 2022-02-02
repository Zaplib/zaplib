//! Mouse cursor styling.
//!
//! Be sure to keep this in sync with cursor_map.ts!

use crate::*;

/// The type of mouse cursor to show. Enjoy the ASCII art here.
#[derive(Clone, Debug, Hash, PartialEq)]
#[repr(u8)]
pub enum MouseCursor {
    /// Don't show the cursor.
    Hidden = 0,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *
    ///  *    *
    ///        *
    /// ```
    Default,

    /// ```text
    ///     |
    ///     |
    ///  ---+---
    ///     |
    ///     |
    /// ```
    Crosshair,

    /// ```text
    ///    *
    ///    *
    ///    * * * *
    /// *  * * * *
    /// *  *     *
    ///  * *     *
    ///  *      *
    /// ```
    Hand,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *
    ///  *    *
    ///        *
    /// ```
    Arrow,

    /// ```text
    ///     ^
    ///     |
    ///  <--+-->
    ///     |
    ///     v
    /// ```
    Move,

    /// ```text
    ///   --+--
    ///     |
    ///     |
    ///   __|__
    /// ```
    Text,

    /// ```text
    ///  |******|
    ///   \****/
    ///    \**/
    ///    /**\
    ///   /****\
    ///  |******|
    /// ```
    Wait,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *
    ///  *    *   ?
    ///        *
    /// ```
    Help,

    /// ```text
    ///    _____
    ///   / \   \
    ///  |   \  |
    ///   \___\/
    /// ```
    NotAllowed,

    /*
    /// ```
    ///  *
    ///  *  *
    ///  *    *
    ///  *      * |----|
    ///  *   *     \--/
    ///  *    *    /--\
    ///        *  |----|
    /// ```
    Progress,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *   |----|
    ///  *    *  |----|
    ///        * |----|
    /// ```
    ContextMenu,

    /// ```text
    ///     | |
    ///     | |
    ///  ---+ +---
    ///  ---+ +---
    ///     | |
    ///     | |
    /// ```
    Cell,

    /// ```text
    ///   |     |
    ///   |-----|
    ///   |     |
    /// ```
    VerticalText,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *    |  ^ |
    ///  *    *   | /  |
    ///        *
    /// ```
    Alias,

    /// ```text
    ///  *
    ///  *  *
    ///  *    *
    ///  *      *
    ///  *   *
    ///  *    *   |+|
    ///        *
    /// ```
    Copy,

    /// ```text
    ///    *
    ///    *
    ///    * * * *
    /// *  * * * *    _____
    /// *  *     *   / \   \
    ///  * *     *  |   \  |
    ///  *      *    \___\/
    /// ```
    NoDrop,

    /// ```text
    ///
    ///    * * * *
    ///    * * * *
    /// *  * * * *
    /// *  *     *
    ///  * *     *
    ///  *      *
    /// ```
    Grab,

    /// ```text
    ///
    ///
    ///    * * * *
    ///  * * * * *
    /// *  *     *
    ///  * *     *
    ///  *      *
    /// ```
    Grabbing,

    /// ```text
    ///     ^
    ///   < * >
    ///     v
    /// ```
    AllScroll,

    /// ```text
    ///   _____
    ///  /  |  \
    ///  | -+- |
    ///  \__|__/
    ///     |
    ///     |
    /// ```
    ZoomIn,

    /// ```text
    ///   _____
    ///  /     \
    ///  | --- |
    ///  \_____/
    ///     |
    ///     |
    /// ```
    ZoomOut,
    */
    /// ```text
    ///     ^
    ///     |
    /// ```
    NResize,

    /// ```text
    ///     ^
    ///    /
    /// ```
    NeResize,

    /// ```text
    ///    -->
    /// ```
    EResize,

    /// ```text
    ///    \
    ///     v
    /// ```
    SeResize,

    /// ```text
    ///     |
    ///     v
    /// ```
    SResize,

    /// ```text
    ///    /
    ///   v
    /// ```
    SwResize,

    /// ```text
    ///    <--
    /// ```
    WResize,

    /// ```text
    ///   ^
    ///    \
    /// ```
    NwResize,

    /// ```text
    ///     ^
    ///     |
    ///     v
    /// ```
    NsResize,

    /// ```text
    ///     ^
    ///    /
    ///   v
    /// ```
    NeswResize,

    /// ```text
    ///  <--->
    /// ```
    EwResize,

    /// ```text
    ///   ^
    ///    \
    ///     v
    /// ```
    NwseResize,

    /// ```text
    ///     ||
    ///   <-||->
    ///     ||
    /// ```
    ColResize,

    /// ```text
    ///     ^
    ///     |
    ///   =====
    ///     |
    ///     v
    /// ```
    RowResize,
}

impl Cx {
    pub fn set_down_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.down_mouse_cursor = Some(mouse_cursor);
    }
    pub fn set_hover_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.hover_mouse_cursor = Some(mouse_cursor);
    }
}

impl Eq for MouseCursor {}
impl Default for MouseCursor {
    fn default() -> MouseCursor {
        MouseCursor::Default
    }
}
