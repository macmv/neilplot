use color::{Oklch, OpaqueColor};

pub struct LinearPalette {
  start: OpaqueColor<Oklch>,
  end:   OpaqueColor<Oklch>,
}

pub const ROCKET: LinearPalette =
  LinearPalette::new(OpaqueColor::new([0.7, 0.13, 50.0]), OpaqueColor::new([0.7, 0.13, 290.0]));

impl LinearPalette {
  pub const fn new(start: OpaqueColor<Oklch>, end: OpaqueColor<Oklch>) -> Self {
    Self { start, end }
  }

  pub fn sample(&self, t: f32) -> OpaqueColor<Oklch> {
    let t = t.clamp(0.0, 1.0);
    self.start.lerp(self.end, t, color::HueDirection::Shorter)
  }
}
