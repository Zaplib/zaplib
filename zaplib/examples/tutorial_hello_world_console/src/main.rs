#![rustfmt::skip]

//ANCHOR: main
use zaplib::*;                          // import all Zaplib functions globally

struct App {}                           // normally this would hold state 

impl App {
    fn new(_cx: &mut Cx) -> App {                         
        App {}                          // initialize & return an empty App                     
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {  
        match event {                   
            Event::Construct => {       // match on a Construct event (the first event)
                log!("Hello, world!");  
            }
            _ => {}                     // ignore all other events
        }
    }
    fn draw(&mut self, _cx: &mut Cx) {}
}

main_app!(App);                        
//ANCHOR_END: main
