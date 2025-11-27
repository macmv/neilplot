use std::collections::HashMap;

use kurbo::{Circle, Point};
use peniko::{Brush, Color};
use polars::prelude::*;

use crate::{Bounds, Range, render::Render};

pub struct ScatterAxes<'a> {
  x:       &'a Column,
  y:       &'a Column,
  options: ScatterOptions,

  hue_column: Option<&'a Column>,
  hue_keys:   Option<Vec<AnyValue<'a>>>,
}

pub struct ScatterOptions {
  pub size:  f64,
  pub color: Brush,
}

impl Default for ScatterOptions {
  fn default() -> Self {
    ScatterOptions { size: 5.0, color: Brush::Solid(Color::from_rgb8(117, 158, 208)) }
  }
}

impl<'a> ScatterAxes<'a> {
  pub(crate) fn new(x: &'a Column, y: &'a Column) -> Self {
    ScatterAxes { x, y, options: ScatterOptions::default(), hue_column: None, hue_keys: None }
  }

  pub(crate) fn data_bounds(&self) -> Bounds {
    Bounds::new(
      Range::new(
        self.x.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
        self.x.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      ),
      Range::new(
        self.y.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
        self.y.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      ),
    )
  }

  pub fn hue_from(&mut self, column: &'a Column) -> &mut Self {
    self.hue_column = Some(column);
    self.hue_keys = None;
    self
  }

  pub fn hue_from_keys<T: Into<AnyValue<'a>>>(
    &mut self,
    column: &'a Column,
    keys: impl IntoIterator<Item = T>,
  ) -> &mut Self {
    self.hue_column = Some(column);
    self.hue_keys = Some(keys.into_iter().map(Into::into).collect::<Vec<_>>());
    self
  }

  fn iter<'b>(&'b self) -> impl Iterator<Item = Point> + 'b {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i).unwrap().try_extract::<f64>().unwrap();
      let y = self.y.get(i).unwrap().try_extract::<f64>().unwrap();

      Point::new(x, y)
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: vello::kurbo::Affine) {
    let unique;

    let hues: Option<HashMap<AnyValue, usize>> = if let Some(order) = &self.hue_keys {
      Some(order.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect::<HashMap<_, _>>())
    } else if let Some(hue_column) = &self.hue_column {
      unique = hue_column.unique_stable().unwrap();

      Some(
        unique
          .as_materialized_series()
          .iter()
          .enumerate()
          .map(|(i, v)| (v, i))
          .collect::<HashMap<_, _>>(),
      )
    } else {
      None
    };

    for (i, point) in self.iter().map(|p| transform * p).enumerate() {
      let color = if let Some(ref hues) = hues {
        let v = self.hue_column.as_ref().unwrap().get(i).unwrap();

        // TODO: Themes
        let index = hues.get(&v).copied().unwrap_or(0) as u8;
        Brush::Solid(Color::from_rgb8(index * 16, 0, 0))
      } else {
        self.options.color.clone()
      };

      render.fill(&Circle::new(point, self.options.size), &color);
    }
  }
}
