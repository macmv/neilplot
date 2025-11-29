use kurbo::{Affine, BezPath, Point, Stroke};
use peniko::{Brush, Color};
use polars::prelude::*;

use crate::{Range, bounds::DataBounds, render::Render};

pub struct LineAxes<'a> {
  x:       &'a Column,
  y:       &'a Column,
  options: LineOptions,
}

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
    Ok(DataBounds {
      x: Range::new(
        self.x.min_reduce()?.into_value().try_extract::<f64>()?,
        self.x.max_reduce()?.into_value().try_extract::<f64>()?,
      )
      .into(),
      y: Range::new(
        self.y.min_reduce()?.into_value().try_extract::<f64>()?,
        self.y.max_reduce()?.into_value().try_extract::<f64>()?,
      )
      .into(),
    })
  }

  fn iter<'b>(&'b self) -> impl Iterator<Item = Point> + 'b {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i).unwrap().try_extract::<f64>().unwrap();
      let y = self.y.get(i).unwrap().try_extract::<f64>().unwrap();

      Point::new(x, y)
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: Affine) {
    let mut shape = BezPath::new();

    for (i, point) in self.iter().map(|p| transform * p).enumerate() {
      if i == 0 {
        shape.move_to(point);
      } else {
        shape.line_to(point);
      }
    }

    let mut stroke = Stroke::new(self.options.width);
    if let Some(dash) = &self.options.dash {
      stroke = stroke.with_dashes(0.0, dash.clone());
    }

    render.stroke(&shape, Affine::IDENTITY, &self.options.color, &stroke);
  }
}
