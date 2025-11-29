use std::borrow::Cow;

use kurbo::{Affine, BezPath, Point, Stroke};
use peniko::Color;
use polars::prelude::*;

use crate::{
  Range, ResultExt,
  bounds::{DataBounds, DataRange},
  render::Render,
};

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
      let Some(v) = v.try_extract::<f64>().log_err() else { continue };

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

  pub(crate) fn data_bounds(&self) -> PolarsResult<DataBounds<'_>> {
    Ok(DataBounds {
      x: DataRange::Continuous { range: self.range, margin_min: false, margin_max: false },
      y: DataRange::Continuous {
        range:      Range::new(
          0.0,
          self.counts.max_reduce()?.into_value().try_extract::<i64>()? as f64,
        )
        .into(),
        margin_min: false,
        margin_max: true,
      },
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut outline = BezPath::new();
    let mut fill = BezPath::new();
    outline.move_to(Point::new(self.range.min, 0.0));
    fill.move_to(Point::new(self.range.min, 0.0));

    let mut prev = None;
    let mut start = None;
    for x in 0..self.counts.len() {
      let Some(count) = self.counts.get(x).and_then(|c| c.try_extract::<i64>()).log_err() else {
        continue;
      };

      let x = self.range.min + (x as f64 / self.counts.len() as f64) * self.range.size();

      if let Some((_, prev_count)) = prev {
        if count > prev_count || prev_count == 0 {
          outline.move_to(Point::new(x, 0.0));
          if count != 0 {
            outline.line_to(Point::new(x, count as f64));
          }
        } else {
          outline.line_to(Point::new(x, 0.0));
          if count != 0 {
            outline.move_to(Point::new(x, count as f64));
          }
        }
      } else {
        outline.line_to(Point::new(x, count as f64));
      }
      prev = Some((x, count));

      fill.line_to(Point::new(x, count as f64));
      fill.line_to(Point::new(x + self.range.size() / self.counts.len() as f64, count as f64));
      if count != 0 {
        outline.line_to(Point::new(x + self.range.size() / self.counts.len() as f64, count as f64));
        if start.is_none() {
          start = Some(x);
        }
      } else if let Some(start_x) = start.take() {
        outline.line_to(Point::new(start_x, 0.0));
      }
    }

    fill.line_to(Point::new(self.range.max, 0.0));
    fill.close_path();
    outline.line_to(Point::new(self.range.max, 0.0));
    if let Some(start_x) = start {
      outline.line_to(Point::new(start_x, 0.0));
    }

    render.fill(&fill, transform, crate::theme::ROCKET.sample(0.0));
    render.stroke(&(transform * outline), Affine::IDENTITY, Color::BLACK, &Stroke::new(2.0));
  }
}
