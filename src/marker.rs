use kurbo::{BezPath, Circle, Point, Rect, Shape};

pub enum Marker {
  Circle,
  Plus,
  Cross,
  Star,

  Square,
  Triangle,
  Diamond,
  Hexagon,
  Octagon,
}

impl Marker {
  pub(crate) fn to_path(&self, tolerance: f64) -> BezPath {
    match self {
      Marker::Circle => Circle::new(Point::new(0.0, 0.0), 0.5).to_path(tolerance),
      Marker::Plus => {
        const INSET: f64 = 0.15;

        let mut path = BezPath::new();
        path.move_to(Point::new(-INSET, -0.5));
        path.line_to(Point::new(INSET, -0.5));
        path.line_to(Point::new(INSET, -INSET));
        path.line_to(Point::new(0.5, -INSET));
        path.line_to(Point::new(0.5, INSET));
        path.line_to(Point::new(INSET, INSET));
        path.line_to(Point::new(INSET, 0.5));
        path.line_to(Point::new(-INSET, 0.5));
        path.line_to(Point::new(-INSET, 0.5));
        path.line_to(Point::new(-INSET, INSET));
        path.line_to(Point::new(-0.5, INSET));
        path.line_to(Point::new(-0.5, -INSET));
        path.line_to(Point::new(-INSET, -INSET));
        path.line_to(Point::new(-INSET, -0.5));
        path.close_path();
        path
      }
      Marker::Cross => {
        const INSET: f64 = 0.15;

        let mut path = BezPath::new();
        path.move_to(Point::new(-0.5 + INSET, -0.5));
        path.line_to(Point::new(0.0, -INSET));
        path.line_to(Point::new(0.5 - INSET, -0.5));
        path.line_to(Point::new(0.5, -0.5 + INSET));
        path.line_to(Point::new(INSET, 0.0));
        path.line_to(Point::new(0.5, 0.5 - INSET));
        path.line_to(Point::new(0.5 - INSET, 0.5));
        path.line_to(Point::new(0.0, INSET));
        path.line_to(Point::new(-0.5 + INSET, 0.5));
        path.line_to(Point::new(-0.5, 0.5 - INSET));
        path.line_to(Point::new(-INSET, 0.0));
        path.line_to(Point::new(-0.5, -0.5 + INSET));
        path.close_path();
        path
      }
      Marker::Square => Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(tolerance),
      Marker::Triangle => {
        // sqrt(3) / 4.0, using the unstable SQRT_3 constant.
        const Y: f64 = 1.732050807568877293527446341505872367_f64 / 4.0;

        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, -Y));
        path.line_to(Point::new(0.5, Y));
        path.line_to(Point::new(-0.5, Y));
        path.close_path();
        path
      }
      Marker::Diamond => {
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, -0.5));
        path.line_to(Point::new(0.5, 0.0));
        path.line_to(Point::new(0.0, 0.5));
        path.line_to(Point::new(-0.5, 0.0));
        path.close_path();
        path
      }
      Marker::Hexagon => {
        // sqrt(3) / 4.0, using the unstable SQRT_3 constant.
        const Y: f64 = 1.732050807568877293527446341505872367_f64 / 4.0;

        let mut path = BezPath::new();
        path.move_to(Point::new(-0.25, -Y));
        path.line_to(Point::new(0.25, -Y));
        path.line_to(Point::new(0.5, 0.0));
        path.line_to(Point::new(0.25, Y));
        path.line_to(Point::new(-0.25, Y));
        path.line_to(Point::new(-0.5, 0.0));
        path.close_path();
        path
      }
      _ => todo!(),
    }
  }
}
