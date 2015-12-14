use super::Point;

pub trait Click {
    fn click(&mut self, point: Point);
    fn on_click(&mut self, func: Box<Fn(&mut Self, Point)>) -> &mut Self;
}
