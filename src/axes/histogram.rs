use std::borrow::Cow;

use kurbo::{Affine, BezPath, Point};
use polars::prelude::*;

use crate::{Range, render::Render};

pub struct HistogramAxes<'a> {
  range:  Range,
  counts: Cow<'a, Column>,
}

impl<'a> HistogramAxes<'a> {
  pub(crate) fn new(values: &'a Column, bins: usize) -> Self {
    let series = values.as_materialized_series();
    let min = series.min::<f64>().unwrap().unwrap();
    let max = series.max::<f64>().unwrap().unwrap();
    let range = Range::new(min, max);

    let mut counts = vec![0; bins];

    for v in series.iter() {
      let Ok(v) = v.try_extract::<f64>() else { continue };

      let mut index = ((v - range.min) / range.size() * bins as f64) as usize;
      if index == bins {
        index -= 1;
      }
      counts[index] += 1;
    }

    HistogramAxes { range, counts: Cow::Owned(Column::new("counts".into(), counts)) }
  }

  pub(crate) fn new_counted(counts: &'a Column) -> Self {
    HistogramAxes { range: Range::new(0.0, counts.len() as f64), counts: Cow::Borrowed(counts) }
  }

  pub(crate) fn data_bounds(&self) -> crate::Bounds {
    crate::Bounds::new(
      self.range,
      crate::Range::new(
        0.0,
        self.counts.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      ),
    )
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut path = BezPath::new();
    path.move_to(Point::new(self.range.min, 0.0));

    for x in 0..self.counts.len() {
      let count = self.counts.get(x).unwrap().try_extract::<f64>().unwrap();

      let x = self.range.min + (x as f64 / self.counts.len() as f64) * self.range.size();

      path.line_to(Point::new(x, count));
      path.line_to(Point::new(x + self.range.size() / self.counts.len() as f64, count));
    }

    path.line_to(Point::new(self.range.max, 0.0));
    path.close_path();

    render.fill(&path, transform, crate::theme::ROCKET.sample(0.0));
  }
}
