use kurbo::{Affine, BezPath, Point, Stroke};
use peniko::{Brush, Color};
use polars::prelude::*;

use crate::{
  ResultExt,
  bounds::{DataBounds, DataRange, ViewportTransform},
  render::Render,
};

pub struct LineAxes<'a> {
  x:       &'a Column,
  y:       &'a Column,
  options: LineOptions,
}

#[derive(Clone)]
pub struct LineOptions {
  pub width: f64,
  pub color: Brush,
  pub dash:  Option<Vec<f64>>,
}

impl Default for LineOptions {
  fn default() -> Self {
    LineOptions { width: 2.0, color: Brush::Solid(Color::from_rgb8(117, 158, 208)), dash: None }
  }
}

impl<'a> LineAxes<'a> {
  pub(crate) fn new(x: &'a Column, y: &'a Column) -> Self {
    LineAxes { x, y, options: LineOptions::default() }
  }

  pub(crate) fn data_bounds(&self) -> PolarsResult<DataBounds<'_>> {
    Ok(DataBounds { x: DataRange::from_column(self.x)?, y: DataRange::from_column(self.y)? })
  }

  fn iter<'b>(&'b self) -> impl Iterator<Item = PolarsResult<Point>> + 'b {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i)?.try_extract::<f64>()?;
      let y = self.y.get(i)?.try_extract::<f64>()?;

      Ok(Point::new(x, y))
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: &ViewportTransform) {
    let mut shape = BezPath::new();

    for (i, point) in self.iter().filter_map(|p| p.log_err()).map(|p| transform * p).enumerate() {
      if i == 0 {
        shape.move_to(point);
      } else {
        shape.line_to(point);
      }
    }

    render.stroke(&shape, Affine::IDENTITY, &self.options.color, &self.options.stroke());
  }
}

impl LineOptions {
  pub(crate) fn stroke(&self) -> Stroke {
    let mut stroke = Stroke::new(self.width);
    if let Some(dash) = &self.dash {
      stroke = stroke.with_dashes(0.0, dash.clone());
    }
    stroke
  }
}
