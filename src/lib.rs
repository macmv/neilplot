use std::collections::HashMap;

use kurbo::{BezPath, Cap, Circle, Line, Point, Stroke};
use parley::FontWeight;
use peniko::{Brush, Color};
use polars::prelude::{AnyValue, Column};

use crate::render::{Align, DrawText, Render};

mod bounds;
mod render;

pub use bounds::{Bounds, Range};

pub struct Plot<'a> {
  pub x: Axis,
  pub y: Axis,

  border: Option<StrokeStyle>,
  grid:   Option<StrokeStyle>,
  title:  Option<String>,

  series: Vec<Series<'a>>,
}

pub struct StrokeStyle {
  stroke: Stroke,
  brush:  Option<Brush>,
}

#[derive(Default)]
pub struct Axis {
  title: Option<String>,
  min:   Option<f64>,
  max:   Option<f64>,
}

pub struct Series<'a> {
  x:      &'a Column,
  y:      &'a Column,
  line:   Option<SeriesLine>,
  points: Option<SeriesPoints>,

  hue_column: Option<&'a Column>,
  hue_keys:   Option<Vec<AnyValue<'a>>>,
}

pub struct SeriesLine {
  pub width: f64,
  pub color: Brush,
  pub dash:  Option<Vec<f64>>,
}

pub struct SeriesPoints {
  pub size:  f64,
  pub color: Brush,
}

impl Default for SeriesLine {
  fn default() -> Self {
    SeriesLine { width: 2.0, color: Brush::Solid(Color::from_rgb8(117, 158, 208)), dash: None }
  }
}

impl Default for SeriesPoints {
  fn default() -> Self {
    SeriesPoints { size: 5.0, color: Brush::Solid(Color::from_rgb8(117, 158, 208)) }
  }
}

impl<'a> Plot<'a> {
  pub fn new() -> Plot<'a> {
    Plot {
      x:      Axis::default(),
      y:      Axis::default(),
      border: Some(StrokeStyle::new(1.0)),
      grid:   None,
      title:  None,
      series: Vec::new(),
    }
  }

  pub fn title(&mut self, title: &str) -> &mut Self {
    self.title = Some(title.to_string());
    self
  }

  pub fn no_border(&mut self) { self.border = None; }

  pub fn border(&mut self) -> &mut StrokeStyle {
    self.border = Some(StrokeStyle::new(1.0));
    self.border.as_mut().unwrap()
  }

  pub fn grid(&mut self) -> &mut StrokeStyle {
    self.grid = Some(StrokeStyle::new(1.0));
    self.grid.as_mut().unwrap()
  }

  pub fn series(&mut self, x: &'a Column, y: &'a Column) -> &mut Series<'a> {
    self.series.push(Series::new(x, y));
    self.series.last_mut().unwrap()
  }

  fn bounds(&self) -> Bounds {
    let bounds = self
      .series
      .iter()
      .map(|s| s.data_bounds())
      .fold(Bounds::empty(), |a, b| a.union(b))
      .expand_by(0.1);

    Bounds::new(
      Range::new(self.x.min.unwrap_or(bounds.x.min), self.x.max.unwrap_or(bounds.x.max)),
      Range::new(self.y.min.unwrap_or(bounds.y.min), self.y.max.unwrap_or(bounds.y.max)),
    )
  }
}

impl StrokeStyle {
  fn new(width: f64) -> Self { Self { stroke: Stroke::new(width), brush: None } }

  pub fn width(&mut self, width: f64) -> &mut Self {
    self.stroke.width = width;
    self
  }

  pub fn dashed(&mut self) -> &mut Self { self.dash_style(&[4.0]) }

  pub fn dash_style(&mut self, dashes: &[f64]) -> &mut Self {
    self.stroke.dash_pattern.resize(dashes.len(), 0.0);
    self.stroke.dash_pattern.copy_from_slice(dashes);
    self
  }
}

impl Axis {
  pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
    self.title = Some(title.into());
    self
  }

  pub fn min(&mut self, min: f64) -> &mut Self {
    self.min = Some(min);
    self
  }

  pub fn max(&mut self, max: f64) -> &mut Self {
    self.max = Some(max);
    self
  }
}

impl<'a> Series<'a> {
  fn new(x: &'a Column, y: &'a Column) -> Self {
    Series { x, y, line: None, points: None, hue_column: None, hue_keys: None }
  }

  pub fn data_bounds(&self) -> Bounds {
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

  pub fn line(&mut self) -> &mut Self {
    self.line = Some(SeriesLine::default());
    self
  }

  pub fn points(&mut self) -> &mut Self {
    self.points = Some(SeriesPoints::default());
    self
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
}

impl Plot<'_> {
  fn draw(&self, render: &mut Render) {
    const TEXT_COLOR: Brush = Brush::Solid(Color::from_rgb8(32, 32, 32));
    const LINE_COLOR: Brush = Brush::Solid(Color::from_rgb8(128, 128, 128));

    let viewport = Bounds::new(Range::new(0.0, 1000.0), Range::new(1000.0, 0.0)).shrink(80.0);

    if let Some(title) = &self.title {
      render.draw_text(DrawText {
        text: title,
        size: 32.0,
        weight: FontWeight::BOLD,
        brush: TEXT_COLOR,
        position: Point { x: 500.0, y: viewport.y.max - 10.0 },
        horizontal_align: Align::Center,
        vertical_align: Align::End,
        ..Default::default()
      });
    }

    if let Some(x_label) = &self.x.title {
      render.draw_text(DrawText {
        text: x_label,
        size: 24.0,
        position: Point { x: 500.0, y: viewport.y.min + 40.0 },
        brush: TEXT_COLOR,
        horizontal_align: Align::Center,
        vertical_align: Align::Start,
        ..Default::default()
      });
    }

    if let Some(y_label) = &self.y.title {
      render.draw_text(DrawText {
        text: y_label,
        size: 24.0,
        position: Point { x: viewport.x.min - 40.0, y: 500.0 },
        brush: TEXT_COLOR,
        transform: vello::kurbo::Affine::rotate(-std::f64::consts::FRAC_PI_2),
        horizontal_align: Align::Center,
        vertical_align: Align::End,
        ..Default::default()
      });
    }

    if let Some(stroke) = &self.border {
      render.stroke(
        &Line::new(
          Point::new(viewport.x.min, viewport.y.min),
          Point::new(viewport.x.max, viewport.y.min),
        ),
        stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
        &stroke.stroke,
      );
      render.stroke(
        &Line::new(
          Point::new(viewport.x.min, viewport.y.min),
          Point::new(viewport.x.min, viewport.y.max),
        ),
        stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
        &stroke.stroke,
      );
    }

    let tick_stroke = Stroke::new(1.0);

    let data_bounds = self.bounds();
    let transform = data_bounds.transform_to(viewport);

    let ticks = 10;
    let iter = data_bounds.y.nice_ticks(ticks);
    let precision = iter.precision();
    for (y, vy) in iter
      .map(|v| (v, (transform * Point::new(0.0, v)).y))
      .filter(|(_, vy)| viewport.y.contains(vy))
    {
      render.stroke(
        &Line::new(Point::new(viewport.x.min, vy), Point::new(viewport.x.min - 10.0, vy)),
        &LINE_COLOR,
        &tick_stroke.clone().with_start_cap(Cap::Butt),
      );
      if let Some(stroke) = &self.grid {
        render.stroke(
          &Line::new(Point::new(viewport.x.min, vy), Point::new(viewport.x.max, vy)),
          stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
          &stroke.stroke,
        );
      }
      render.draw_text(DrawText {
        text: &format!("{:.*}", (precision - 3).min(0), y),
        size: 12.0,
        position: Point { x: viewport.x.min - 15.0, y: vy },
        brush: TEXT_COLOR,
        horizontal_align: Align::End,
        vertical_align: Align::Center,
        ..Default::default()
      });
    }

    let iter = data_bounds.x.nice_ticks(ticks);
    let precision = iter.precision();
    for (x, vx) in iter
      .map(|v| (v, (transform * Point::new(v, 0.0)).x))
      .filter(|(_, vx)| viewport.x.contains(vx))
    {
      render.stroke(
        &Line::new(Point::new(vx, viewport.y.min), Point::new(vx, viewport.y.min + 10.0)),
        &LINE_COLOR,
        &tick_stroke.clone().with_start_cap(Cap::Butt),
      );
      if let Some(stroke) = &self.grid {
        render.stroke(
          &Line::new(Point::new(vx, viewport.y.min), Point::new(vx, viewport.y.max)),
          stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
          &stroke.stroke,
        );
      }
      render.draw_text(DrawText {
        text: &format!("{:.*}", (precision - 3).min(0), x),
        size: 12.0,
        position: Point { x: vx, y: viewport.y.min + 15.0 },
        brush: TEXT_COLOR,
        horizontal_align: Align::Center,
        vertical_align: Align::Start,
        ..Default::default()
      });
    }

    for series in &self.series {
      let unique;

      let hues: Option<HashMap<AnyValue, usize>> = if let Some(order) = &series.hue_keys {
        Some(order.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect::<HashMap<_, _>>())
      } else if let Some(hue_column) = &series.hue_column {
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

      if let Some(line) = &series.line {
        let mut shape = BezPath::new();

        for (i, point) in series.iter().map(|p| transform * p).enumerate() {
          if i == 0 {
            shape.move_to(point);
          } else {
            shape.line_to(point);
          }
        }

        let mut stroke = Stroke::new(line.width);
        if let Some(dash) = &line.dash {
          stroke = stroke.with_dashes(0.0, dash.clone());
        }

        render.stroke(&shape, &line.color, &stroke);
      }

      let points = if series.points.is_none() && series.line.is_none() {
        &SeriesPoints::default()
      } else if let Some(points) = &series.points {
        points
      } else {
        continue;
      };

      for (i, point) in series.iter().map(|p| transform * p).enumerate() {
        let color = if let Some(ref hues) = hues {
          let v = series.hue_column.as_ref().unwrap().get(i).unwrap();

          // TODO: Themes
          let index = hues.get(&v).copied().unwrap_or(0) as u8;
          Brush::Solid(Color::from_rgb8(index * 16, 0, 0))
        } else {
          points.color.clone()
        };

        render.fill(&Circle::new(point, points.size), &color);
      }
    }
  }
}

impl Series<'_> {
  fn iter<'a>(&'a self) -> impl Iterator<Item = Point> + 'a {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i).unwrap().try_extract::<f64>().unwrap();
      let y = self.y.get(i).unwrap().try_extract::<f64>().unwrap();

      Point::new(x, y)
    })
  }
}
