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
  x:       &'a Column,
  y:       &'a Column,
  x_range: Range,
  y_range: Range,
  line:    Option<SeriesLine>,
  points:  Option<SeriesPoints>,
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
      x_range: x_range.expanded_by(0.1),
      y_range: y_range.expanded_by(0.1),
      line: Some(SeriesLine::default()),
      points: None,
    }
  }

  pub fn x_min(&mut self, min: f64) -> &mut Self {
    self.x_range.min = min;
    self
  }
  pub fn x_max(&mut self, max: f64) -> &mut Self {
    self.x_range.max = max;
    self
  }
  pub fn y_min(&mut self, min: f64) -> &mut Self {
    self.y_range.min = min;
    self
  }
  pub fn y_max(&mut self, max: f64) -> &mut Self {
    self.y_range.max = max;
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

    if let Some(title) = &self.title {
      render.draw_text(DrawText {
        text: title,
        size: 32.0,
        weight: FontWeight::BOLD,
        brush: TEXT_COLOR,
        position: Point { x: 500.0, y: 20.0 },
        horizontal_align: Align::Center,
        ..Default::default()
      });
    }

    if let Some(x_label) = &self.x_label {
      render.draw_text(DrawText {
        text: x_label,
        size: 24.0,
        position: Point { x: 500.0, y: 950.0 },
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
        position: Point { x: 45.0, y: 500.0 },
        brush: TEXT_COLOR,
        transform: vello::kurbo::Affine::rotate(-std::f64::consts::FRAC_PI_2),
        horizontal_align: Align::Center,
        vertical_align: Align::End,
        ..Default::default()
      });
    }

    let viewport_x = Range::new(50.0, 950.0);
    let viewport_y = Range::new(950.0, 50.0);

    let border_stroke = Stroke::new(2.0);
    render.stroke(
      &Line::new(
        Point::new(viewport_x.min, viewport_y.min),
        Point::new(viewport_x.max, viewport_y.min),
      ),
      &LINE_COLOR,
      &border_stroke,
    );
    render.stroke(
      &Line::new(
        Point::new(viewport_x.min, viewport_y.min),
        Point::new(viewport_x.min, viewport_y.max),
      ),
      &LINE_COLOR,
      &border_stroke,
    );

    let ticks = 10;
    let iter = self.series[0].y_range.nice_ticks(ticks);
    let precision = iter.precision();
    for (y, vy) in iter
      .map(|v| (v, transform(v, &self.series[0].y_range, &viewport_y)))
      .filter(|(_, vy)| viewport_y.contains(vy))
    {
      render.stroke(
        &Line::new(Point::new(viewport_x.min, vy), Point::new(viewport_x.min - 10.0, vy)),
        &LINE_COLOR,
        &border_stroke.clone().with_start_cap(Cap::Butt),
      );
      render.draw_text(DrawText {
        text: &format!("{:.*}", precision - 3, y),
        size: 12.0,
        position: Point { x: viewport_x.min - 15.0, y: vy },
        brush: TEXT_COLOR,
        horizontal_align: Align::End,
        vertical_align: Align::Center,
        ..Default::default()
      });
    }

    let iter = self.series[0].x_range.nice_ticks(ticks);
    let precision = iter.precision();
    for (x, vx) in iter
      .map(|v| (v, transform(v, &self.series[0].x_range, &viewport_x)))
      .filter(|(_, vx)| viewport_x.contains(vx))
    {
      render.stroke(
        &Line::new(Point::new(vx, viewport_y.min), Point::new(vx, viewport_y.min + 10.0)),
        &LINE_COLOR,
        &border_stroke.clone().with_start_cap(Cap::Butt),
      );
      render.draw_text(DrawText {
        text: &format!("{:.*}", precision - 3, x),
        size: 12.0,
        position: Point { x: vx, y: viewport_y.min + 15.0 },
        brush: TEXT_COLOR,
        horizontal_align: Align::Center,
        vertical_align: Align::Start,
        ..Default::default()
      });
    }

    for series in &self.series {
      if let Some(line) = &series.line {
        let mut shape = BezPath::new();

        for (i, point) in series.iter_points().enumerate() {
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

      for point in series.iter_points() {
        if let Some(points) = &series.points {
          render.fill(&Circle::new(point, points.size), &points.color);
        }
      }
    }
  }
}

impl Series<'_> {
  fn iter_points(&self) -> impl Iterator<Item = Point> + '_ {
    (0..self.x.len()).map(move |i| {
      let x = self.x.get(i).unwrap().try_extract::<f64>().unwrap();
      let y = self.y.get(i).unwrap().try_extract::<f64>().unwrap();

      let x = 50.0 + ((x - self.x_range.min) * 900.0 / (self.x_range.max - self.x_range.min));
      let y = 950.0 - ((y - self.y_range.min) * 900.0 / (self.y_range.max - self.y_range.min));

      Point::new(x, y)
    })
  }
}

fn transform(value: f64, from_range: &Range, to_range: &Range) -> f64 {
  to_range.min + (value - from_range.min) * to_range.size() / from_range.size()
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
