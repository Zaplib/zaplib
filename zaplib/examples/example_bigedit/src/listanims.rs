use zaplib::*;

pub const LIST_ANIMS_COLOR_BG_EVEN: Vec4 = vec4(0.16, 0.16, 0.16, 1.0);
pub const LIST_ANIMS_COLOR_BG_ODD: Vec4 = vec4(0.15, 0.15, 0.15, 1.0);
pub const LIST_ANIMS_ANIM_EVEN: Anim = Anim {
    duration: 0.01,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(1.0, LIST_ANIMS_COLOR_BG_EVEN)], ease: Ease::DEFAULT }],
};
pub const LIST_ANIMS_ANIM_ODD: Anim = Anim {
    duration: 0.01,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(1.0, LIST_ANIMS_COLOR_BG_ODD)], ease: Ease::DEFAULT }],
};
pub const LIST_ANIMS_ANIM_MARKED: Anim = Anim {
    duration: 0.01,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.07, 0.27, 0.43, 1.0))], ease: Ease::DEFAULT }],
};
pub const LIST_ANIMS_ANIM_EVEN_OVER: Anim = Anim {
    duration: 0.02,
    chain: false,
    tracks: &[Track::Vec4 { key_frames: &[(0.0, vec4(0.24, 0.24, 0.24, 1.0))], ease: Ease::DEFAULT }],
};
pub const LIST_ANIMS_ANIM_ODD_OVER: Anim = Anim {
    duration: 0.02,
    chain: false,
    tracks: &[Track::Vec4 { key_frames: &[(0.0, vec4(0.22, 0.22, 0.22, 1.0))], ease: Ease::DEFAULT }],
};
pub const LIST_ANIMS_ANIM_MARKED_OVER: Anim = Anim { duration: 0.02, chain: false, tracks: LIST_ANIMS_ANIM_MARKED.tracks };
