pub mod app_window;
pub mod engine;
pub mod state;

// Draw a 2d circle 
pub fn draw_circle(location: Point2D, radius: u128) {

}

pub fn init_window(screen_width: u128, screen_height: u128, title: &str) {

}

pub fn close_window() {

}

pub struct Point2D {
    x: i128,
    y: i128
}

pub enum Color {
    Red,
    Green,
    Blue
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
