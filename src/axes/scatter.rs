use std::collections::HashMap;

use kurbo::{Affine, Point};
use peniko::{Brush, Color};
use polars::prelude::*;

use crate::{Marker, Range, bounds::DataBounds, render::Render};

pub struct ScatterAxes<'a> {
  x:       &'a Column,
  y:       &'a Column,
  options: ScatterOptions,

  hue_column: Option<&'a Column>,
  hue_keys:   Option<Vec<AnyValue<'a>>>,
}

pub struct ScatterOptions {
  pub size:   f64,
  pub marker: Marker,
  pub color:  Brush,
}

impl Default for ScatterOptions {
  fn default() -> Self {
    ScatterOptions {
      size:   12.0,
      marker: Marker::Circle,
      color:  Brush::Solid(Color::from_rgb8(117, 158, 208)),
    }
  }
}

impl<'a> ScatterAxes<'a> {
  pub(crate) fn new(x: &'a Column, y: &'a Column) -> Self {
    ScatterAxes { x, y, options: ScatterOptions::default(), hue_column: None, hue_keys: None }
  }

  pub(crate) fn data_bounds(&self) -> DataBounds {
    DataBounds {
      x: Range::new(
        self.x.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
        self.x.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      )
      .into(),
      y: Range::new(
        self.y.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
        self.y.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      )
      .into(),
    }
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

  pub fn marker_size(&mut self, size: f64) -> &mut Self {
    self.options.size = size;
    self
  }

  pub fn marker(&mut self, marker: Marker) -> &mut Self {
    self.options.marker = marker;
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

    let shape = self.options.marker.to_path(0.1);

    for (i, point) in self.iter().map(|p| transform * p).enumerate() {
      let color = if let Some(ref hues) = hues {
        let v = self.hue_column.as_ref().unwrap().get(i).unwrap();

        // TODO: Themes
        let index = hues.get(&v).copied().unwrap_or(0) as f32 / (hues.len() as f32);
        crate::theme::ROCKET.sample(index).into()
      } else {
        self.options.color.clone()
      };

      render.fill(&shape, Affine::scale(self.options.size).then_translate(point.to_vec2()), &color);
    }
  }
}
