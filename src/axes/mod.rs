mod histogram;
mod line;
mod scatter;

pub use histogram::HistogramAxes;
pub use line::LineAxes;
pub use scatter::ScatterAxes;

use crate::Plot;
use polars::prelude::*;

pub enum Axes<'a> {
  Scatter(ScatterAxes<'a>),
  Line(LineAxes<'a>),
  Histogram(HistogramAxes<'a>),
}

impl Axes<'_> {
  pub fn data_bounds(&self) -> Option<crate::Bounds> {
    match self {
      Axes::Scatter(a) => Some(a.data_bounds()),
      Axes::Line(a) => Some(a.data_bounds()),
      Axes::Histogram(a) => Some(a.data_bounds()),
    }
  }

  pub fn draw(&self, render: &mut crate::render::Render, transform: vello::kurbo::Affine) {
    match self {
      Axes::Scatter(a) => a.draw(render, transform),
      Axes::Line(a) => a.draw(render, transform),
      Axes::Histogram(a) => a.draw(render, transform),
    }
  }
}

impl<'a> Plot<'a> {
  pub fn scatter(&mut self, x: &'a Column, y: &'a Column) -> &mut ScatterAxes<'a> {
    self.axes.push(Axes::Scatter(ScatterAxes::new(x, y)));
    match self.axes.last_mut().unwrap() {
      Axes::Scatter(sa) => sa,
      _ => unreachable!(),
    }
  }

  pub fn line(&mut self, x: &'a Column, y: &'a Column) -> &mut LineAxes<'a> {
    self.axes.push(Axes::Line(LineAxes::new(x, y)));
    match self.axes.last_mut().unwrap() {
      Axes::Line(sa) => sa,
      _ => unreachable!(),
    }
  }

  pub fn histogram(&mut self, values: &'a Column, bins: usize) -> &mut HistogramAxes<'a> {
    self.axes.push(Axes::Histogram(HistogramAxes::new(values, bins)));
    match self.axes.last_mut().unwrap() {
      Axes::Histogram(sa) => sa,
      _ => unreachable!(),
    }
  }

  pub fn histogram_counted(&mut self, counts: &'a Column) -> &mut HistogramAxes<'a> {
    self.axes.push(Axes::Histogram(HistogramAxes::new_counted(counts)));
    match self.axes.last_mut().unwrap() {
      Axes::Histogram(sa) => sa,
      _ => unreachable!(),
    }
  }
}
