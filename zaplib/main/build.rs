use vergen::{vergen, Config};

fn main() {
    // If we can't find .git (e.g. on Heroku), then just skip.
    vergen(Config::default()).unwrap_or_default()
}
