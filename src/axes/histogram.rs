use kurbo::{Affine, BezPath, Point};
use polars::prelude::*;

use crate::render::Render;

pub struct HistogramAxes<'a> {
  counts: &'a Column,
}

impl<'a> HistogramAxes<'a> {
  pub(crate) fn new(counts: &'a Column) -> Self { HistogramAxes { counts } }

  pub(crate) fn data_bounds(&self) -> crate::Bounds {
    crate::Bounds::new(
      crate::Range::new(0.0, self.counts.len() as f64),
      crate::Range::new(
        0.0,
        self.counts.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      ),
    )
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut path = BezPath::new();
    path.move_to(Point::new(0.0, 0.0));

    for x in 0..self.counts.len() {
      let count = self.counts.get(x).unwrap().try_extract::<f64>().unwrap();

      path.line_to(Point::new(x as f64, count));
      path.line_to(Point::new(x as f64 + 1.0, count));
    }

    path.line_to(Point::new(self.counts.len() as f64, 0.0));
    path.close_path();

    render.fill(&path, transform, crate::theme::ROCKET.sample(0.0));
  }
}
