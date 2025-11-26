use parley::FontWeight;
use polars::prelude::Column;
use vello::{
  kurbo::{BezPath, Cap, Circle, Line, Point, Stroke},
  peniko::{Brush, Color},
};

use crate::render::{Align, DrawText, Render};

mod render;

#[derive(Default)]
pub struct Plot<'a> {
  title:   Option<String>,
  x_label: Option<String>,
  y_label: Option<String>,

  series: Vec<Series<'a>>,
}

pub struct Series<'a> {
  x:      &'a Column,
  y:      &'a Column,
  bounds: Bounds,
  line:   Option<SeriesLine>,
  points: Option<SeriesPoints>,
}

pub struct Bounds {
  pub x: Range,
  pub y: Range,
}

pub struct Range {
  pub min: f64,
  pub max: f64,
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
  pub fn new() -> Plot<'a> { Plot::default() }

  pub fn title(&mut self, title: &str) -> &mut Self {
    self.title = Some(title.to_string());
    self
  }

  pub fn x_label(&mut self, label: &str) -> &mut Self {
    self.x_label = Some(label.to_string());
    self
  }

  pub fn y_label(&mut self, label: &str) -> &mut Self {
    self.y_label = Some(label.to_string());
    self
  }

  pub fn series(&mut self, x: &'a Column, y: &'a Column) -> &mut Series<'a> {
    self.series.push(Series::new(x, y));
    self.series.last_mut().unwrap()
  }
}

impl<'a> Series<'a> {
  fn new(x: &'a Column, y: &'a Column) -> Self {
    let x_range = Range::new(
      x.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      x.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
    );
    let y_range = Range::new(
      y.min_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
      y.max_reduce().unwrap().into_value().try_extract::<f64>().unwrap(),
    );

    Series {
      x,
      y,
      bounds: Bounds::new(x_range, y_range).expanded_by(0.1),
      line: Some(SeriesLine::default()),
      points: None,
    }
  }

  pub fn x_min(&mut self, min: f64) -> &mut Self {
    self.bounds.x.min = min;
    self
  }
  pub fn x_max(&mut self, max: f64) -> &mut Self {
    self.bounds.x.max = max;
    self
  }
  pub fn y_min(&mut self, min: f64) -> &mut Self {
    self.bounds.y.min = min;
    self
  }
  pub fn y_max(&mut self, max: f64) -> &mut Self {
    self.bounds.y.max = max;
    self
  }

  pub fn points(&mut self) -> &mut Self {
    self.points = Some(SeriesPoints::default());
    self
  }
}

impl Plot<'_> {
  fn draw(&self, render: &mut Render) {
    const TEXT_COLOR: Brush = Brush::Solid(Color::from_rgb8(32, 32, 32));
    const LINE_COLOR: Brush = Brush::Solid(Color::from_rgb8(128, 128, 128));

    let viewport = Bounds::new(Range::new(80.0, 920.0), Range::new(920.0, 80.0));

    if let Some(title) = &self.title {
      render.draw_text(DrawText {
        text: title,
        size: 32.0,
        weight: FontWeight::BOLD,
        brush: TEXT_COLOR,
        position: Point { x: 500.0, y: viewport.y.max - 30.0 },
        horizontal_align: Align::Center,
        ..Default::default()
      });
    }

    if let Some(x_label) = &self.x_label {
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

    if let Some(y_label) = &self.y_label {
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

    let border_stroke = Stroke::new(2.0);
    render.stroke(
      &Line::new(
        Point::new(viewport.x.min, viewport.y.min),
        Point::new(viewport.x.max, viewport.y.min),
      ),
      &LINE_COLOR,
      &border_stroke,
    );
    render.stroke(
      &Line::new(
        Point::new(viewport.x.min, viewport.y.min),
        Point::new(viewport.x.min, viewport.y.max),
      ),
      &LINE_COLOR,
      &border_stroke,
    );

    let ticks = 10;
    let iter = self.series[0].bounds.y.nice_ticks(ticks);
    let precision = iter.precision();
    for (y, vy) in iter
      .map(|v| (v, transform(v, &self.series[0].bounds.y, &viewport.y)))
      .filter(|(_, vy)| viewport.y.contains(vy))
    {
      render.stroke(
        &Line::new(Point::new(viewport.x.min, vy), Point::new(viewport.x.min - 10.0, vy)),
        &LINE_COLOR,
        &border_stroke.clone().with_start_cap(Cap::Butt),
      );
      render.draw_text(DrawText {
        text: &format!("{:.*}", precision - 3, y),
        size: 12.0,
        position: Point { x: viewport.x.min - 15.0, y: vy },
        brush: TEXT_COLOR,
        horizontal_align: Align::End,
        vertical_align: Align::Center,
        ..Default::default()
      });
    }

    let iter = self.series[0].bounds.x.nice_ticks(ticks);
    let precision = iter.precision();
    for (x, vx) in iter
      .map(|v| (v, transform(v, &self.series[0].bounds.x, &viewport.x)))
      .filter(|(_, vx)| viewport.x.contains(vx))
    {
      render.stroke(
        &Line::new(Point::new(vx, viewport.y.min), Point::new(vx, viewport.y.min + 10.0)),
        &LINE_COLOR,
        &border_stroke.clone().with_start_cap(Cap::Butt),
      );
      render.draw_text(DrawText {
        text: &format!("{:.*}", precision - 3, x),
        size: 12.0,
        position: Point { x: vx, y: viewport.y.min + 15.0 },
        brush: TEXT_COLOR,
        horizontal_align: Align::Center,
        vertical_align: Align::Start,
        ..Default::default()
      });
    }

    for series in &self.series {
      if let Some(line) = &series.line {
        let mut shape = BezPath::new();

        for (i, point) in series.iter_points(&viewport).enumerate() {
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

      for point in series.iter_points(&viewport) {
        if let Some(points) = &series.points {
          render.fill(&Circle::new(point, points.size), &points.color);
        }
      }
    }
  }
}

impl Series<'_> {
  fn iter_points<'a>(&'a self, bounds: &'a Bounds) -> impl Iterator<Item = Point> + 'a {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i).unwrap().try_extract::<f64>().unwrap();
      let y = self.y.get(i).unwrap().try_extract::<f64>().unwrap();

      transform_point(Point::new(x, y), &self.bounds, bounds)
    })
  }
}

fn transform(value: f64, from_range: &Range, to_range: &Range) -> f64 {
  to_range.min + (value - from_range.min) * to_range.size() / from_range.size()
}

fn transform_point(point: Point, from_bounds: &Bounds, to_bounds: &Bounds) -> Point {
  Point::new(
    transform(point.x, &from_bounds.x, &to_bounds.x),
    transform(point.y, &from_bounds.y, &to_bounds.y),
  )
}

impl Bounds {
  pub const fn new(x: Range, y: Range) -> Self { Bounds { x, y } }

  pub const fn expanded_by(self, fract: f64) -> Self {
    Bounds { x: self.x.expanded_by(fract), y: self.y.expanded_by(fract) }
  }
}

impl Range {
  pub const fn new(min: f64, max: f64) -> Self { Range { min, max } }
  pub const fn size(&self) -> f64 { self.max - self.min }

  pub const fn expanded_by(self, fract: f64) -> Self {
    Range { min: self.min - self.size() * fract, max: self.max + self.size() * fract }
  }

  pub const fn contains(&self, value: &f64) -> bool {
    (*value >= self.min && *value <= self.max) || (*value <= self.min && *value >= self.max)
  }

  pub fn nice_ticks(&self, count: u32) -> TicksIter {
    let step = (self.max - self.min) / f64::from(count);
    let k = step.log10().floor();
    let base = step / 10f64.powf(k);

    let nice_base = match base {
      b if b < 1.0 => 1.0,
      b if b < 2.0 => 2.0,
      b if b < 2.5 => 2.5,
      b if b < 5.0 => 5.0,
      _ => 10.0,
    };

    let step = nice_base * 10f64.powf(k);
    let lo = (self.min / step).floor() * step;
    let hi = (self.max / step).ceil() * step;

    let precision = (-k as i32 + 4).max(0) as usize;
    TicksIter::new(lo, hi, step, precision)
  }
}

pub struct TicksIter {
  current:   f64,
  step:      f64,
  hi:        f64,
  precision: usize,
}

impl TicksIter {
  fn new(lo: f64, hi: f64, step: f64, precision: usize) -> Self {
    TicksIter { current: lo, step, hi, precision }
  }

  pub fn precision(&self) -> usize { self.precision }
}

impl Iterator for TicksIter {
  type Item = f64;
  fn next(&mut self) -> Option<Self::Item> {
    if self.current < self.hi + self.step * 0.5 {
      let p = 10f64.powi(self.precision as i32);
      let result = (self.current * p).round() / p;
      self.current += self.step;
      Some(result)
    } else {
      None
    }
  }
}
