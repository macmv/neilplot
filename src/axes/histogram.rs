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

  pub(crate) fn draw(&self, render: &mut Render, transform: vello::kurbo::Affine) {}
}
