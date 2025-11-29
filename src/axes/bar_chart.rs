use kurbo::{Affine, BezPath, Point};
use polars::prelude::*;

use crate::{
  Range,
  bounds::{DataBounds, DataRange},
  render::Render,
};

pub struct BarChartAxes<'a> {
  labels: &'a Column,
  values: &'a Column,
}

impl<'a> BarChartAxes<'a> {
  pub(crate) fn new(labels: &'a Column, values: &'a Column) -> Self {
    BarChartAxes { labels, values }
  }

  pub(crate) fn data_bounds(&self) -> DataBounds<'_> {
    DataBounds {
      x: DataRange::Categorical(self.labels),
      y: DataRange::Continuous {
        range:      Range::new(
          0.0,
          self.values.max_reduce().unwrap().into_value().try_extract::<i64>().unwrap() as f64,
        )
        .into(),
        margin_min: false,
        margin_max: true,
      },
    }
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut fill = BezPath::new();

    for x in 0..self.labels.len() {
      const WIDTH: f64 = 0.3;
      let value = self.values.get(x).unwrap().try_extract::<f64>().unwrap();

      fill.move_to(Point::new(x as f64 - WIDTH, 0.0));
      fill.line_to(Point::new(x as f64 - WIDTH, value));
      fill.line_to(Point::new(x as f64 + WIDTH, value));
      fill.line_to(Point::new(x as f64 + WIDTH, 0.0));
      fill.line_to(Point::new(x as f64 - WIDTH, 0.0));
    }

    render.fill(&fill, transform, crate::theme::ROCKET.sample(0.0));
  }
}
