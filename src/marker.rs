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
      Marker::Star => {
        // (pi * 2 / 5).sin() / 2 and -(pi * 2 / 5).cos() / 2
        const OX_1: f64 = 0.475528258147576765590969216646044515;
        const OY_1: f64 = -0.154508497187473725631434717797674239;
        const OX_2: f64 = 0.293892626146236624062879627672373317;
        const OY_2: f64 = 0.404508497187473670120283486539847218;

        // 0.5 * (1 - cos(54 degrees)/cos(18 degrees))
        const INNER_RADIUS: f64 = 0.5 * 0.381966011250105097474261128809303045;

        // (pi / 5).sin() * inner and -(pi / 5).cos() * inner
        const IX_1: f64 = 0.112256994144896329879124152739677811;
        const IY_1: f64 = -0.154508497187473697875859102168760728;
        const IX_2: f64 = 0.181635632001340197039240820231498219;
        const IY_2: f64 = 0.059016994374947402690612108244749834;

        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, -0.5));
        path.line_to(Point::new(IX_1, IY_1));
        path.line_to(Point::new(OX_1, OY_1));
        path.line_to(Point::new(IX_2, IY_2));
        path.line_to(Point::new(OX_2, OY_2));
        path.line_to(Point::new(0.0, INNER_RADIUS));
        path.line_to(Point::new(-OX_2, OY_2));
        path.line_to(Point::new(-IX_2, IY_2));
        path.line_to(Point::new(-OX_1, OY_1));
        path.line_to(Point::new(-IX_1, IY_1));
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
      Marker::Octagon => {
        const OFFSET: f64 = (std::f64::consts::SQRT_2 - 1.0) / 2.0;

        let mut path = BezPath::new();
        path.move_to(Point::new(-OFFSET, -0.5));
        path.line_to(Point::new(OFFSET, -0.5));
        path.line_to(Point::new(0.5, -OFFSET));
        path.line_to(Point::new(0.5, OFFSET));
        path.line_to(Point::new(OFFSET, 0.5));
        path.line_to(Point::new(-OFFSET, 0.5));
        path.line_to(Point::new(-0.5, OFFSET));
        path.line_to(Point::new(-0.5, -OFFSET));
        path.close_path();
        path
      }
    }
  }
}
