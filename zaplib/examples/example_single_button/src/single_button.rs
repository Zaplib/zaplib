use zaplib::*;
use zaplib_components::*;

#[derive(Default)]
pub struct SingleButton {
    button: Button,
    clicks: u32,
}

impl SingleButton {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.button.handle(cx, event) {
            self.clicks += 1;
            cx.request_draw();
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.button.draw(cx, &format!("I've been clicked {} times", self.clicks));
    }
}

/*

Equivalent React component-style:

class SingleButton {
    state = {
        clicks: 0,
    },

    onButtonClick = () => {
        this.setState({ clicks: this.state.clicks + 1 });
    }

    render() {
        return (
            <button onClick={this.onButtonClick}>
              I've been clicked {this.state.clicks} times
            </button>
        );
    }
}

Equivalent React functional-style:

function SingleButton() {
    const [clicks, setClicks] = useState(false);
    return (
        <button onClick={() => setClicks(clicks + 1)}>
            I've been clicked {clicks} times
        </button>
    );
}

*/
