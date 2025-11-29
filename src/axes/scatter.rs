use std::collections::HashMap;

use kurbo::{Affine, Line, Point, Stroke};
use peniko::{Brush, Color};
use polars::prelude::*;

use crate::{Marker, Range, ResultExt, bounds::DataBounds, render::Render};

pub struct ScatterAxes<'a> {
  x:       &'a Column,
  y:       &'a Column,
  options: ScatterOptions,

  hue_column: Option<&'a Column>,
  hue_keys:   Option<Vec<AnyValue<'a>>>,
}

pub struct ScatterOptions {
  pub size:      f64,
  pub marker:    Marker,
  pub color:     Brush,
  pub trendline: Option<TrendlineOptions>,
}

pub struct TrendlineOptions {
  pub kind:  TrendlineKind,
  pub color: Brush,
  pub width: f64,
}

pub enum TrendlineKind {
  Polynomial(usize),
}

impl TrendlineKind {
  pub const LINEAR: Self = TrendlineKind::Polynomial(1);
}

impl Default for ScatterOptions {
  fn default() -> Self {
    ScatterOptions {
      size:      12.0,
      marker:    Marker::Circle,
      color:     Brush::Solid(Color::from_rgb8(117, 158, 208)),
      trendline: None,
    }
  }
}

impl<'a> ScatterAxes<'a> {
  pub(crate) fn new(x: &'a Column, y: &'a Column) -> Self {
    ScatterAxes { x, y, options: ScatterOptions::default(), hue_column: None, hue_keys: None }
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

  pub fn trendline(&mut self, kind: TrendlineKind) -> &mut TrendlineOptions {
    self.options.trendline = Some(TrendlineOptions {
      kind,
      color: Brush::Solid(Color::from_rgb8(200, 50, 50)),
      width: 2.0,
    });
    self.options.trendline.as_mut().unwrap()
  }

  fn iter<'b>(&'b self) -> impl Iterator<Item = PolarsResult<Point>> + 'b {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i)?.try_extract::<f64>()?;
      let y = self.y.get(i)?.try_extract::<f64>()?;

      Ok(Point::new(x, y))
    })
  }

  pub(crate) fn draw(&self, render: &mut Render, transform: vello::kurbo::Affine) {
    let unique;

    let hues: Option<HashMap<AnyValue, usize>> = if let Some(order) = &self.hue_keys {
      Some(order.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect::<HashMap<_, _>>())
    } else if let Some(hue_column) = &self.hue_column {
      if let Some(u) = hue_column.unique_stable().log_err() {
        unique = u;
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
      }
    } else {
      None
    };

    let shape = self.options.marker.to_path(0.1);

    for (i, point) in self.iter().filter_map(|p| p.log_err()).map(|p| transform * p).enumerate() {
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

    if let Some(trendline) = &self.options.trendline {
      trendline.draw(self.x, self.y, render, transform).log_err();
    }
  }
}

impl TrendlineOptions {
  fn draw(
    &self,
    x: &Column,
    y: &Column,
    render: &mut Render,
    transform: Affine,
  ) -> PolarsResult<()> {
    let df =
      DataFrame::new(vec![x.clone().with_name("x".into()), y.clone().with_name("y".into())])?;
    let stats = df
      .lazy()
      .select([
        cov(col("x"), col("y"), 1).alias("cov_xy"),
        col("x").var(1).alias("var_x"),
        col("x").mean().alias("mean_x"),
        col("y").mean().alias("mean_y"),
      ])
      .collect()?;

    let s_cov = stats.column("cov_xy")?.f64()?.get(0).unwrap();
    let s_var = stats.column("var_x")?.f64()?.get(0).unwrap();
    let mean_x = stats.column("mean_x")?.f64()?.get(0).unwrap();
    let mean_y = stats.column("mean_y")?.f64()?.get(0).unwrap();

    let slope = s_cov / s_var;
    let intercept = mean_y - slope * mean_x;

    let p0 = Point::new(
      x.min_reduce()?.into_value().try_extract::<f64>()?,
      x.min_reduce()?.into_value().try_extract::<f64>()? * slope + intercept,
    );
    let p1 = Point::new(
      x.max_reduce()?.into_value().try_extract::<f64>()?,
      x.max_reduce()?.into_value().try_extract::<f64>()? * slope + intercept,
    );

    let line = Line::new(p0, p1);
    render.stroke(&(transform * line), Affine::IDENTITY, &self.color, &Stroke::new(self.width));

    Ok(())
  }
}
