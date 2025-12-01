mod bar_chart;
mod histogram;
mod line;
mod scatter;

pub use bar_chart::BarChartAxes;
pub use histogram::HistogramAxes;
pub use line::{LineAxes, LineOptions};
pub use scatter::{ScatterAxes, TrendlineKind};

use crate::{
  Plot,
  bounds::{DataBounds, ViewportTransform},
};
use polars::prelude::*;

pub enum Axes<'a> {
  Scatter(ScatterAxes<'a>),
  Line(LineAxes<'a>),
  Histogram(HistogramAxes<'a>),
  BarChart(BarChartAxes<'a>),
}

impl Axes<'_> {
  pub fn data_bounds(&self) -> PolarsResult<DataBounds<'_>> {
    match self {
      Axes::Scatter(a) => a.data_bounds(),
      Axes::Line(a) => a.data_bounds(),
      Axes::Histogram(a) => a.data_bounds(),
      Axes::BarChart(a) => a.data_bounds(),
    }
  }

  pub(crate) fn draw(&self, render: &mut crate::render::Render, transform: &ViewportTransform) {
    match self {
      Axes::Scatter(a) => a.draw(render, transform),
      Axes::Line(a) => a.draw(render, transform),
      Axes::Histogram(a) => a.draw(render, transform),
      Axes::BarChart(a) => a.draw(render, transform),
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
    self.x.ticks_fixed(bins + 1);
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

  pub fn bar_chart(&mut self, labels: &'a Column, values: &'a Column) -> &mut BarChartAxes<'a> {
    self.axes.push(Axes::BarChart(BarChartAxes::new(labels, values)));
    match self.axes.last_mut().unwrap() {
      Axes::BarChart(a) => a,
      _ => unreachable!(),
    }
  }
}
