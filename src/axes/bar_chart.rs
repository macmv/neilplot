use kurbo::{Affine, BezPath, Point};
use polars::prelude::*;

use crate::{
  Range, ResultExt,
  bounds::{DataBounds, DataRange, RangeUnit},
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

  pub(crate) fn data_bounds(&self) -> PolarsResult<DataBounds<'_>> {
    Ok(DataBounds {
      x: DataRange::Categorical(self.labels),
      y: DataRange::Continuous {
        range:      Range::new(
          0.0,
          self.values.max_reduce()?.into_value().try_extract::<i64>()? as f64,
        )
        .into(),
        unit:       RangeUnit::Absolute,
        margin_min: false,
        margin_max: true,
      },
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut fill = BezPath::new();

    for x in 0..self.labels.len() {
      const WIDTH: f64 = 0.3;
      let Some(value) = self.values.get(x).and_then(|v| v.try_extract::<f64>()).log_err() else {
        continue;
      };

      fill.move_to(Point::new(x as f64 - WIDTH, 0.0));
      fill.line_to(Point::new(x as f64 - WIDTH, value));
      fill.line_to(Point::new(x as f64 + WIDTH, value));
      fill.line_to(Point::new(x as f64 + WIDTH, 0.0));
      fill.line_to(Point::new(x as f64 - WIDTH, 0.0));
    }

    render.fill(&fill, transform, crate::theme::ROCKET.sample(0.0));
  }
}
