// Not great but we do these comparisons all over the place..
#![allow(clippy::float_cmp)]
// Clippy TODO
#![allow(clippy::all)]

mod builder;
mod buildmanager;
mod codeicon;
mod colorpicker;
mod fieldworld;
mod fileeditor;
mod filepanel;
mod filetree;
mod homepage;
mod itemdisplay;
mod jseditor;
mod keyboard;
mod listanims;
mod loglist;
mod makepadapp;
mod makepadstorage;
mod makepadwindow;
mod mprstokenizer;
mod plaineditor;
mod rusteditor;
mod searchindex;
mod searchresults;
mod treeworld;
mod worldview;

use crate::makepadapp::MakepadApp;
use zaplib::*;
main_app!(MakepadApp);
