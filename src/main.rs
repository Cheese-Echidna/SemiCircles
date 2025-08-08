mod sketch;
mod easing;

use async_std::task::block_on;
use nannou::App;
use sketch::{run_app, Model};

fn main() {
    block_on(async {
        run_app(1024, 768).await;
    });
}


// fn main() {
//     nannou::app(model).update(update).fullscreen().run();
// }
