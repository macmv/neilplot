use polars::prelude::*;

use crate::render::Render;

pub struct HistogramAxes<'a> {
  counts: &'a Column,
}

impl<'a> HistogramAxes<'a> {
  pub(crate) fn new(counts: &'a Column) -> Self { HistogramAxes { counts } }

  pub(crate) fn draw(&self, render: &mut Render, transform: vello::kurbo::Affine) {}
}
