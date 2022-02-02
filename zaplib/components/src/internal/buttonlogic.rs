use crate::button::ButtonEvent;
use zaplib::*;

#[derive(Clone, PartialEq)]
pub(crate) enum ButtonLogicEvent {
    Over,
    Default,
    Down,
}

pub(crate) fn handle_button_logic<F>(cx: &mut Cx, hit_event: Event, mut cb: F) -> ButtonEvent
where
    F: FnMut(&mut Cx, ButtonLogicEvent),
{
    match hit_event {
        Event::PointerDown(_pe) => {
            cb(cx, ButtonLogicEvent::Down);
            return ButtonEvent::Down;
        }
        Event::PointerHover(pe) => {
            cx.set_hover_mouse_cursor(MouseCursor::Hand);
            match pe.hover_state {
                HoverState::In => {
                    if pe.any_down {
                        cb(cx, ButtonLogicEvent::Down);
                    } else {
                        cb(cx, ButtonLogicEvent::Over);
                    }
                }
                HoverState::Out => cb(cx, ButtonLogicEvent::Default),
                _ => (),
            }
        }
        Event::PointerUp(pe) => {
            if pe.is_over {
                if pe.input_type.has_hovers() {
                    cb(cx, ButtonLogicEvent::Over)
                } else {
                    cb(cx, ButtonLogicEvent::Default)
                }
                return ButtonEvent::Clicked;
            } else {
                cb(cx, ButtonLogicEvent::Default);
                return ButtonEvent::Up;
            }
        }
        _ => (),
    };
    ButtonEvent::None
}
