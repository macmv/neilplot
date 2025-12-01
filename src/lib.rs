use std::fmt;

use kurbo::{Affine, Cap, Line, Point, Stroke};
use parley::FontWeight;
use peniko::{Brush, Color};
use polars::prelude::{AnyValue, Column};

use crate::{
  bounds::{DataBounds, DataRange, RangeUnit, ViewportTransform},
  render::{Align, DrawText, Render},
};

mod axes;
mod bounds;
mod legend;
mod marker;
mod render;

pub mod theme;

pub use axes::*;
pub use bounds::{Bounds, Range};
pub use marker::Marker;

pub(crate) trait ResultExt<T> {
  fn log_err(self) -> Option<T>;
}

impl<T> ResultExt<T> for polars::prelude::PolarsResult<T> {
  fn log_err(self) -> Option<T> {
    match self {
      Ok(v) => Some(v),
      Err(e) => {
        eprintln!("Polars error: {}", e);
        None
      }
    }
  }
}

pub struct Plot<'a> {
  pub x: Axis,
  pub y: Axis,

  border: Option<StrokeStyle>,
  grid:   Option<StrokeStyle>,
  title:  Option<String>,

  axes: Vec<Axes<'a>>,
}

pub struct StrokeStyle {
  stroke: Stroke,
  brush:  Option<Brush>,
}

pub struct Axis {
  title:  Option<String>,
  scale:  Scale,
  min:    Option<f64>,
  max:    Option<f64>,
  margin: f64,
  ticks:  Ticks,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
  #[default]
  Linear,
  Logarithmic,
}

#[derive(Default)]
pub enum Ticks {
  #[default]
  Auto,
  Fixed(usize),
}

impl Default for Axis {
  fn default() -> Self {
    Axis {
      title:  None,
      scale:  Scale::Linear,
      min:    None,
      max:    None,
      margin: 0.1,
      ticks:  Ticks::Auto,
    }
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
      axes:   Vec::new(),
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

  fn bounds(&self) -> DataBounds<'_> {
    let mut bounds: Option<DataBounds> = None;
    for axes in &self.axes {
      let Some(bound) = axes.data_bounds().log_err() else { continue };
      bounds = Some(match bounds {
        Some(b) => self.union_bounds(b, bound),
        None => bound,
      });
    }

    bounds.unwrap_or(DataBounds {
      x: DataRange::Continuous {
        range:      Range::new(0.0, 1.0),
        unit:       RangeUnit::Absolute,
        margin_min: false,
        margin_max: false,
      },
      y: DataRange::Continuous {
        range:      Range::new(0.0, 1.0),
        unit:       RangeUnit::Absolute,
        margin_min: false,
        margin_max: false,
      },
    })
  }

  fn union_bounds(&self, a: DataBounds<'_>, b: DataBounds<'_>) -> DataBounds<'_> {
    DataBounds { x: self.union_range(a.x, b.x), y: self.union_range(a.y, b.y) }
  }

  fn union_range(&self, a: DataRange<'_>, b: DataRange<'_>) -> DataRange<'_> {
    match (a, b) {
      (
        DataRange::Continuous {
          range: range_a,
          unit: unit_a,
          margin_min: min_a,
          margin_max: max_a,
        },
        DataRange::Continuous { range: range_b, unit: _, margin_min: min_b, margin_max: max_b },
      ) => DataRange::Continuous {
        range:      range_a.union(range_b),
        unit:       unit_a, // TODO
        margin_min: min_a || min_b,
        margin_max: max_a || max_b,
      },
      _ => panic!("Cannot union non-continuous ranges"),
    }
  }

  fn pretty_bounds(&self, data_bounds: DataBounds<'_>) -> Bounds {
    Bounds { x: self.x.pretty_range(data_bounds.x), y: self.y.pretty_range(data_bounds.y) }
  }

  fn viewport_transform(&self, data_bounds: DataBounds<'_>, viewport: Bounds) -> ViewportTransform {
    let pretty = self.pretty_bounds(data_bounds);
    let from = Bounds::new(
      pretty.x.map(|v| self.x.scale.scale_value(v)),
      pretty.y.map(|v| self.y.scale.scale_value(v)),
    );
    let affine = from.transform_to(viewport);

    ViewportTransform { affine, x: self.x.scale, y: self.y.scale }
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

  pub fn margin(&mut self, margin: f64) -> &mut Self {
    self.margin = margin;
    self
  }

  pub fn ticks_fixed(&mut self, count: usize) -> &mut Self {
    self.ticks = Ticks::Fixed(count);
    self
  }

  pub fn log_scale(&mut self) -> &mut Self {
    self.scale = Scale::Logarithmic;
    self
  }
}

impl<'a> ScatterAxes<'a> {}

impl Plot<'_> {
  fn draw(&self, render: &mut Render) {
    const TEXT_COLOR: Brush = Brush::Solid(Color::from_rgb8(32, 32, 32));
    const LINE_COLOR: Brush = Brush::Solid(Color::from_rgb8(128, 128, 128));

    let outer =
      Bounds::new(Range::new(0.0, render.size().width), Range::new(render.size().height, 0.0));

    let viewport = outer.shrink(80.0);

    if let Some(title) = &self.title {
      render.draw_text(DrawText {
        text: title,
        size: 32.0,
        weight: FontWeight::BOLD,
        brush: TEXT_COLOR,
        position: Point { x: outer.width() / 2.0, y: viewport.y.max - 10.0 },
        horizontal_align: Align::Center,
        vertical_align: Align::End,
        ..Default::default()
      });
    }

    if let Some(x_label) = &self.x.title {
      render.draw_text(DrawText {
        text: x_label,
        size: 24.0,
        position: Point { x: outer.width() / 2.0, y: viewport.y.min + 40.0 },
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
        position: Point { x: viewport.x.min - 40.0, y: -outer.height() / 2.0 },
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
        Affine::IDENTITY,
        stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
        &stroke.stroke,
      );
      render.stroke(
        &Line::new(
          Point::new(viewport.x.min, viewport.y.min),
          Point::new(viewport.x.min, viewport.y.max),
        ),
        Affine::IDENTITY,
        stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
        &stroke.stroke,
      );
    }

    let tick_stroke = Stroke::new(1.0);

    let data_bounds = self.bounds();
    let transform = self.viewport_transform(data_bounds, viewport);
    let transform = &transform;

    let ticks = 10;
    let iter = self.y.iter_ticks(data_bounds.y, ticks);
    for (y, vy) in iter
      .map(|t| {
        let y = (transform * Point::new(0.0, t.position())).y;
        (t, y)
      })
      .filter(|(_, vy)| viewport.y.contains(vy))
    {
      render.stroke(
        &Line::new(Point::new(viewport.x.min, vy), Point::new(viewport.x.min - 10.0, vy)),
        Affine::IDENTITY,
        &LINE_COLOR,
        &tick_stroke.clone().with_start_cap(Cap::Butt),
      );
      if let Some(stroke) = &self.grid {
        render.stroke(
          &Line::new(Point::new(viewport.x.min, vy), Point::new(viewport.x.max, vy)),
          Affine::IDENTITY,
          stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
          &stroke.stroke,
        );
      }
      render.draw_text(DrawText {
        text: &y.to_string(),
        size: 12.0,
        position: Point { x: viewport.x.min - 15.0, y: vy },
        brush: TEXT_COLOR,
        horizontal_align: Align::End,
        vertical_align: Align::Center,
        ..Default::default()
      });
    }

    let iter = self.x.iter_ticks(data_bounds.x, ticks);
    for (x, vx) in iter
      .map(|t| {
        let x = (transform * Point::new(t.position(), 0.0)).x;
        (t, x)
      })
      .filter(|(_, vx)| viewport.x.contains(vx))
    {
      render.stroke(
        &Line::new(Point::new(vx, viewport.y.min), Point::new(vx, viewport.y.min + 10.0)),
        Affine::IDENTITY,
        &LINE_COLOR,
        &tick_stroke.clone().with_start_cap(Cap::Butt),
      );
      if let Some(stroke) = &self.grid {
        render.stroke(
          &Line::new(Point::new(vx, viewport.y.min), Point::new(vx, viewport.y.max)),
          Affine::IDENTITY,
          stroke.brush.as_ref().unwrap_or(&LINE_COLOR),
          &stroke.stroke,
        );
      }
      render.draw_text(DrawText {
        text: &x.to_string(),
        size: 12.0,
        position: Point { x: vx, y: viewport.y.min + 15.0 },
        brush: TEXT_COLOR,
        horizontal_align: Align::Center,
        vertical_align: Align::Start,
        ..Default::default()
      });
    }

    for axes in &self.axes {
      axes.draw(render, transform);
    }

    self.draw_legend(render, viewport);
  }
}

enum TicksIter<'a> {
  Auto { iter: bounds::NiceTicksIter, scale: Scale, unit: RangeUnit },
  Fixed(FixedTicksIter),
  Labeled(ColumnIter<'a>),
}

struct FixedTicksIter {
  count:   usize,
  current: usize,
  start:   f64,
  step:    f64,
}

struct ColumnIter<'a> {
  column:  &'a Column,
  current: usize,
}

#[derive(Clone)]
enum Tick<'a> {
  Auto { value: f64, precision: u32, unit: RangeUnit },
  Fixed { value: f64 },
  Label { label: AnyValue<'a>, index: usize },
}

impl Tick<'_> {
  fn position(&self) -> f64 {
    match self {
      Tick::Auto { value, .. } => *value,
      Tick::Fixed { value } => *value,
      Tick::Label { index, .. } => *index as f64,
    }
  }
}

impl Axis {
  fn iter_ticks<'a>(&self, range: DataRange<'a>, nice_ticks: u32) -> TicksIter<'a> {
    match &self.ticks {
      Ticks::Auto => match range {
        DataRange::Categorical(labels) => {
          TicksIter::Labeled(ColumnIter { column: labels, current: 0 })
        }
        DataRange::Continuous { range, unit, .. } => {
          let range = match self.scale {
            Scale::Linear => range,
            Scale::Logarithmic => Range::new(range.min.log10(), range.max.log10()),
          };
          TicksIter::Auto { iter: range.nice_ticks(nice_ticks), scale: self.scale, unit }
        }
      },
      Ticks::Fixed(count) => {
        TicksIter::Fixed(FixedTicksIter::new(self.pretty_range(range), *count))
      }
    }
  }

  fn pretty_range(&self, r: DataRange) -> Range {
    match r {
      DataRange::Continuous { range, margin_min, margin_max, .. } => {
        let mut r = range;
        if margin_min {
          match self.scale {
            Scale::Linear => r.min -= (r.size() * self.margin).abs(),
            Scale::Logarithmic => {
              r.min -= 10_f64.powf(((r.max.log10() - r.min.log10()) * self.margin).abs())
            }
          }
        }
        if margin_max {
          match self.scale {
            Scale::Linear => r.max += (r.size() * self.margin).abs(),
            Scale::Logarithmic => {
              r.max += 10_f64.powf(((r.max.log10() - r.min.log10()) * self.margin).abs())
            }
          }
        }
        Range::new(self.min.unwrap_or(r.min), self.max.unwrap_or(r.max))
      }
      DataRange::Categorical(labels) => Range::new(-0.5, labels.len() as f64 - 0.5),
    }
  }
}

impl<'a> Iterator for TicksIter<'a> {
  type Item = Tick<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      TicksIter::Auto { iter, scale, unit } => iter.next().map(|v| Tick::Auto {
        value:     match scale {
          Scale::Linear => v,
          Scale::Logarithmic => 10f64.powf(v),
        },
        precision: iter.precision() as u32,
        unit:      *unit,
      }),
      TicksIter::Fixed(iter) => iter.next().map(|v| Tick::Fixed { value: v }),
      TicksIter::Labeled(iter) => iter.next().map(|(i, v)| Tick::Label { label: v, index: i }),
    }
  }
}

impl FixedTicksIter {
  pub fn new(range: Range, count: usize) -> Self {
    FixedTicksIter {
      count,
      current: 0,
      start: range.min,
      step: range.size() / count.saturating_sub(1) as f64,
    }
  }
}

impl Iterator for FixedTicksIter {
  type Item = f64;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current >= self.count {
      None
    } else {
      let value = self.start + (self.current as f64) * self.step;
      self.current += 1;
      Some(value)
    }
  }
}

impl<'a> Iterator for ColumnIter<'a> {
  type Item = (usize, AnyValue<'a>);

  fn next(&mut self) -> Option<Self::Item> {
    if self.current >= self.column.len() {
      None
    } else {
      let v = self.column.get(self.current).unwrap();
      let curr = self.current;
      self.current += 1;
      Some((curr, v))
    }
  }
}

impl fmt::Display for Tick<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self {
      Tick::Auto { value, precision, unit: RangeUnit::Absolute } => {
        write!(f, "{value:.*}", precision.saturating_sub(3) as usize)
      }
      Tick::Auto { value, precision: _, unit: RangeUnit::Duration } => {
        write!(f, "{:?}", std::time::Duration::from_nanos(*value as u64))
      }
      Tick::Auto { value, precision: _, unit: RangeUnit::Date } => {
        write!(f, "{}", AnyValue::Date(*value as i32))
      }
      Tick::Fixed { value } => write!(f, "{value:.2}"),
      Tick::Label { label, .. } => match label {
        AnyValue::String(s) => write!(f, "{s}"),
        AnyValue::StringOwned(s) => write!(f, "{s}"),
        _ => write!(f, "{label}"),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fixed_iter_works() {
    let iter = FixedTicksIter::new(Range::new(0.0, 1.0), 5);

    let results: Vec<f64> = iter.collect();
    assert_eq!(results, vec![0.0, 0.25, 0.5, 0.75, 1.0]);

    let iter = FixedTicksIter::new(Range::new(0.0, 2.0), 5);

    let results: Vec<f64> = iter.collect();
    assert_eq!(results, vec![0.0, 0.5, 1.0, 1.5, 2.0]);
  }
}
