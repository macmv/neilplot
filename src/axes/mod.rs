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
