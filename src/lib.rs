use parley::FontWeight;
use polars::prelude::Column;
use vello::{
  kurbo::{Affine, BezPath, Cap, Circle, Line, Point, Stroke},
  peniko::{Brush, Color},
};

use crate::render::{Align, DrawText, Render};

mod render;

#[derive(Default)]
pub struct Plot<'a> {
  title:   Option<String>,
  x_label: Option<String>,
  y_label: Option<String>,
  bounds:  Option<Bounds>,

  series: Vec<Series<'a>>,
}

pub struct Series<'a> {
  x:      &'a Column,
  y:      &'a Column,
  bounds: Bounds,
  line:   Option<SeriesLine>,
  points: Option<SeriesPoints>,
}

#[derive(Clone, Copy)]
pub struct Bounds {
  pub x: Range,
  pub y: Range,
}

#[derive(Clone, Copy)]
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
      bounds: Bounds::new(x_range, y_range).expand_by(0.1),
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

    let viewport = Bounds::new(Range::new(0.0, 1000.0), Range::new(1000.0, 0.0)).shrink(80.0);

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

    let data_bounds = self.bounds.unwrap_or_else(|| {
      self.series.iter().map(|s| s.bounds).fold(Bounds::empty(), |a, b| a.union(b))
    });

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

    let iter = data_bounds.x.nice_ticks(ticks);
    let precision = iter.precision();
    for (x, vx) in iter
      .map(|v| (v, (transform * Point::new(v, 0.0)).x))
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

      for point in series.iter().map(|p| transform * p) {
        if let Some(points) = &series.points {
          render.fill(&Circle::new(point, points.size), &points.color);
        }
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

impl Bounds {
  pub const fn empty() -> Self { Bounds { x: Range::empty(), y: Range::empty() } }
  pub const fn new(x: Range, y: Range) -> Self { Bounds { x, y } }

  pub const fn shrink(self, amount: f64) -> Self {
    Bounds { x: self.x.shrink(amount), y: self.y.shrink(amount) }
  }
  pub const fn shrink_by(self, fract: f64) -> Self {
    Bounds { x: self.x.shrink_by(fract), y: self.y.shrink(fract) }
  }

  pub const fn expand(self, amount: f64) -> Self {
    Bounds { x: self.x.expand(amount), y: self.y.expand(amount) }
  }
  pub const fn expand_by(self, fract: f64) -> Self {
    Bounds { x: self.x.expand_by(fract), y: self.y.expand_by(fract) }
  }

  pub fn union(&self, other: Bounds) -> Bounds {
    Bounds { x: self.x.union(other.x), y: self.y.union(other.y) }
  }

  fn transform_to(&self, viewport: Bounds) -> Affine {
    let scale_x = viewport.x.size() / self.x.size();
    let scale_y = viewport.y.size() / self.y.size();
    let translate_x = viewport.x.min - self.x.min * scale_x;
    let translate_y = viewport.y.min - self.y.min * scale_y;

    Affine::new([scale_x, 0.0, 0.0, scale_y, translate_x, translate_y])
  }
}

impl Range {
  pub const fn empty() -> Self { Range { min: 0.0, max: 0.0 } }
  pub const fn new(min: f64, max: f64) -> Self { Range { min, max } }
  pub const fn size(&self) -> f64 { self.max - self.min }

  pub const fn shrink(self, amount: f64) -> Self { self.expand(-amount) }
  pub const fn shrink_by(self, fract: f64) -> Self { self.shrink(self.size() * fract) }
  pub const fn expand(self, amount: f64) -> Self {
    Range {
      min: self.min - amount * self.size().signum(),
      max: self.max + amount * self.size().signum(),
    }
  }
  pub const fn expand_by(self, fract: f64) -> Self { self.expand(self.size() * fract) }

  pub const fn contains(&self, value: &f64) -> bool {
    (*value >= self.min && *value <= self.max) || (*value <= self.min && *value >= self.max)
  }

  pub fn union(&self, other: Range) -> Range {
    if self.size() == 0.0 {
      other
    } else if other.size() == 0.0 {
      *self
    } else {
      Range { min: self.min.min(other.min), max: self.max.max(other.max) }
    }
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
