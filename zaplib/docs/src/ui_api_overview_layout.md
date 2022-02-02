# Layout

## Introduction

Layouting is a mechanism in Zaplib to position components on the screen.

A core philosophy of the Zaplib layouting model is its simplicity and speed, by having only a single pass
to do layouting. Contrast this with systems like [CSS Flexbox](https://en.wikipedia.org/wiki/CSS_Flexible_Box_Layout),
which use a constraint satisfaction system to lay out your widgets. Instead, we make a single
pass, but do sometimes shift over individual elements after the fact. When doing this we can regard it as a "1.5-pass" rendering. 

 - The core concept is a "box" - entity having a current draw position and a "sandbox" where it can draw in.
 - Boxes can be nested, so we have a stack of boxes. 
 - The boxes are defined imperatively: new boxes are pushed to the stack using various `begin_*` methods and popped using corresponding `end_*` methods. 
 - The last box in the stack is the "active" one and all rendering calls are added to that box's sandbox. 
 - When the active box is ended it's sandbox content is pushed into its parent's sandbox (potentially shifted when using alignments) and then parent draw position is moved accordingly. 

For the examples on how to use the Layout API, follow the [UI Layout](./tutorial_ui_layout.md) tutorial.

## API Overview

<!-- Define styles that would be used in example below -->
<style>
.box {
  background-color: rgba(0,0,255,.2);
  border: 3px solid #00f;
  margin: 2px;
}

.content_box {
  background-color: #fff;
  border: 3px solid #00f;
}

.outer {
  border: 3px solid #000;
  padding: 5px;
  display: flex;
}
.caption {
  border: 0px;
  margin: 2px;
}
.row {
  display: flex;
  margin: 5px;
}
</style>

### [`begin_row`](/target/doc/zaplib/struct.Cx.html#method.begin_row) / [`end_row`](/target/doc/zaplib/struct.Cx.html#method.end_row)

Defines a box that has its nested elements layed out horizontally (as a row).


<div class="outer" style="flex-direction: row; width: 250px">
  <div class="box" style="width: 75px; height: 75px">Box 1</div>
  <div class="box" style="width: 100px; height: 100px">Box 2</div>
  <div class="box" style="width: 50px; height: 50px">Box 3</div>
</div>

### [`begin_column`](/target/doc/zaplib/struct.Cx.html#method.begin_column) / [`end_column`](/target/doc/zaplib/struct.Cx.html#method.end_column)

Defines a box that has it nested elements layed out vertically (as a column).

<div class="outer" style="flex-direction: column; width: 110px">
  <div class="box" style="width: 75px; height: 75px">Box 1</div>
  <div class="box" style="width: 100px; height: 100px">Box 2</div>
  <div class="box" style="width: 50px; height: 50px">Box 3</div>
</div>


### [`begin_absolute_box`](/target/doc/zaplib/struct.Cx.html#method.begin_absolute_box) / [`end_absolute_box`](/target/doc/zaplib/struct.Cx.html#method.end_absolute_box)


Defines a box that is absolutely positioned starting at (0, 0) coordinate. Normally the new box starts at the current draw position of the active box. This method allows to bypass that and use absolute coordinates on the screen.


### [`begin_padding_box`](/target/doc/zaplib/struct.Cx.html#method.begin_padding_box) / [`end_padding_box`](/target/doc/zaplib/struct.Cx.html#method.end_padding_box)

Defines a box that has padding inside. 

<div class="outer" style="flex-direction: column; width: 250px; height: 160px">
    <div class="row" style="flex-direction: row; width: 250px; height: 30px">
        <div  style="width: 70px; height: 30px"></div>
        <div cstyle="width: 100px; height: 30px">Top Padding</div>
    </div>
    <div class="row" style="flex-direction: row; width: 250px; height: 50px">
        <div  style="width: 70px; height: 50px"> Left Padding</div>
        <div class="box" style="width: 100px; height: 50px">Content</div>
        <div  style="width: 50px; height: 50px"> Right Padding</div>
    </div>
    <div class="row" style="flex-direction: row; width: 250px; height: 50px">
        <div style="width: 70px; height: 50px"> </div>
        <div style="width: 100px; height: 50px">Bottom Padding</div>
    </div>
</div>



### [`begin_wrapping_box`](/target/doc/zaplib/struct.Cx.html#method.begin_wrapping_box) / [`end_wrapping_box`](/target/doc/zaplib/struct.Cx.html#method.end_wrapping_box)

Defines a box that is wrapping its content inside. This is only supported for horizontal direction. This is defined in terms of child boxes, meaning that if any of the immediately nested boxes goes beyond the bounds, it would be wrapped to new line). Text has its own wrapping mechanism via [`TextInsProps::wrapping`](/target/doc/zaplib/struct.TextInsProps.html#structfield.wrapping).


### [`begin_right_box`](/target/doc/zaplib/struct.Cx.html#method.begin_right_box) / [`end_right_box`](/target/doc/zaplib/struct.Cx.html#method.end_right_box)

Defines a box that will be aligned to the right by x axis within last box.

<div class="outer" style="flex-direction: row-reverse; width: 250px">
  <div class="box" style="width: 50px; height: 50px">Right Box</div>
</div>

### [`begin_bottom_box`](/target/doc/zaplib/struct.Cx.html#method.begin_bottom_box) / [`end_bottom_box`](/target/doc/zaplib/struct.Cx.html#method.end_bottom_box)

Defines a box that will be aligned to the bottom by y axis within last box.

<div class="outer" style="flex-direction: column-reverse; width: 70px; height: 250px">
  <div class="box" style="width: 60px; height: 60px">Bottom Box</div>
</div>


### [`begin_center_x_align`](/target/doc/zaplib/struct.Cx.html#method.begin_center_x_align) / [`end_center_x_align`](/target/doc/zaplib/struct.Cx.html#method.end_center_x_align)

Defines a box that fills up all remaining space by x axis and will be aligned to center by x axis within last box.

<div class="outer" style="flex-direction: row; justify-content: center; width: 150px; height: 150px">
  <div class="box" style="width: 50px; height: 50px">Box</div>
</div>

### [`begin_center_y_align`](/target/doc/zaplib/struct.Cx.html#method.begin_center_y_align) / [`end_center_y_align`](/target/doc/zaplib/struct.Cx.html#method.end_center_y_align)

Defines a box that fills up all remaining space by y axis and will be aligned to center by y axis within last box.

<div class="outer" style="flex-direction: column; justify-content: center; width: 150px; height: 150px">
  <div class="box" style="width: 50px; height: 50px">Box</div>
</div>


### [`begin_center_x_and_y_align`](/target/doc/zaplib/struct.Cx.html#method.begin_center_x_and_y_align) / [`end_center_x_and_y_align`](/target/doc/zaplib/struct.Cx.html#method.end_center_x_and_y_align)

Defines a box that fills up all remaining space by x and y axises and will be aligned to center by both x and y axises within last box.

<div class="outer" style="flex-direction: column; align-items: center; justify-content: center; width: 150px; height: 150px">
  <div class="box" style="width: 50px; height: 50px">Box</div>
</div>


### [`get_box_rect`](/target/doc/zaplib/struct.Cx.html#method.get_box_rect)

Returns the full rect corresponding to current box. It counts all available_width/height plus padding.

### [`get_width_left`](/target/doc/zaplib/struct.Cx.html#method.get_width_left) / [`get_height_left`](/target/doc/zaplib/struct.Cx.html#method.get_height_left)

Returns the amount of width / height left within the current box. 


### [`get_width_total`](/target/doc/zaplib/struct.Cx.html#method.get_width_total) / [`get_height_total`](/target/doc/zaplib/struct.Cx.html#method.get_height_total)


Get some notion of the total width / height of the active box. If the width/ height is well defined, then we return it. If it's computed, then we return the bound (including padding) of how much we've drawn so far. And if we haven't drawn anything, we return 0.


### [`get_box_bounds`](/target/doc/zaplib/struct.Cx.html#method.get_box_bounds)

Get the bounds of what the box has actually drawn, including any padding that the layout of the active box specifies.

### [`get_box_origin`](/target/doc/zaplib/struct.Cx.html#method.get_box_origin)

Returns the starting position of the active box in absolute coordinates.

### [`get_draw_pos`](/target/doc/zaplib/struct.Cx.html#method.get_draw_pos)

Returns the current draw position of the active box in absolute coordinates.

### [`add_box`](/target/doc/zaplib/struct.Cx.html#method.add_box)

Moves the current draw position of the active box as if the new box with provided dimensions was drawn inside.

### [`move_draw_pos`](/target/doc/zaplib/struct.Cx.html#method.move_draw_pos)

Manually move the current draw position of the active box. Warning! Does not update bounds like `add_box` does; might result in unexpected behavior.

### [`set_draw_pos`](/target/doc/zaplib/struct.Cx.html#method.set_draw_pos)

Manually set the current draw position of the active box. Warning! Does not update bounds like `add_box` does; might result in unexpected behavior.
