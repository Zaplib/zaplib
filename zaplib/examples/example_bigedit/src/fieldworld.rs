// a bunch o buttons to select the world
use zaplib::*;
use zaplib_components::*;

#[derive(Default)]
pub struct FieldWorld {
    pub area: Area,
}

impl FieldWorld {
    pub fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {
        // lets see.
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        SkyBox::draw(cx, vec3(0., 0., 0.));
    }
}
