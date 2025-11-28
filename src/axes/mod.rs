mod line;
mod scatter;

pub use line::LineAxes;
pub use scatter::ScatterAxes;

use crate::Plot;
use polars::prelude::*;

pub enum Axes<'a> {
  Scatter(ScatterAxes<'a>),
  Line(LineAxes<'a>),
}

impl Axes<'_> {
  pub fn data_bounds(&self) -> crate::Bounds {
    match self {
      Axes::Scatter(sa) => sa.data_bounds(),
      Axes::Line(la) => la.data_bounds(),
    }
  }

  pub fn draw(&self, render: &mut crate::render::Render, transform: vello::kurbo::Affine) {
    match self {
      Axes::Scatter(sa) => sa.draw(render, transform),
      Axes::Line(la) => la.draw(render, transform),
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
}
