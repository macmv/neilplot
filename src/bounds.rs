use kurbo::Affine;
use polars::{error::PolarsResult, prelude::Column};

#[derive(Clone, Copy)]
pub struct Bounds {
  pub x: Range,
  pub y: Range,
}

#[derive(Clone, Copy)]
pub struct DataBounds<'a> {
  pub x: DataRange<'a>,
  pub y: DataRange<'a>,
}

#[derive(Clone, Copy)]
pub enum DataRange<'a> {
  Continuous { range: Range, unit: RangeUnit, margin_min: bool, margin_max: bool },
  Categorical(&'a Column),
}

#[derive(Clone, Copy)]
pub enum RangeUnit {
  Absolute,
  Duration,
  Date,
}

#[derive(Clone, Copy)]
pub struct Range {
  pub min: f64,
  pub max: f64,
}

impl From<Range> for DataRange<'_> {
  fn from(range: Range) -> Self {
    DataRange::Continuous { range, unit: RangeUnit::Absolute, margin_min: true, margin_max: true }
  }
}

impl Bounds {
  pub const fn empty() -> Self { Bounds { x: Range::empty(), y: Range::empty() } }
  pub const fn new(x: Range, y: Range) -> Self { Bounds { x, y } }

  pub fn width(&self) -> f64 { self.x.size() }
  pub fn height(&self) -> f64 { self.y.size() }

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

  pub(crate) fn transform_to(&self, viewport: Bounds) -> Affine {
    let scale_x = viewport.x.size() / self.x.size();
    let scale_y = viewport.y.size() / self.y.size();
    let translate_x = viewport.x.min - self.x.min * scale_x;
    let translate_y = viewport.y.min - self.y.min * scale_y;

    Affine::new([scale_x, 0.0, 0.0, scale_y, translate_x, translate_y])
  }
}

impl Default for Range {
  fn default() -> Self { Range::empty() }
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

  pub fn nice_ticks(&self, count: u32) -> NiceTicksIter {
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
    NiceTicksIter::new(lo, hi, step, precision)
  }
}

impl DataRange<'_> {
  pub(crate) fn from_column<'a, 'b>(column: &'a Column) -> PolarsResult<DataRange<'b>> {
    Ok(DataRange::Continuous {
      range:      Range::new(
        column.min_reduce()?.into_value().try_extract::<f64>()?,
        column.max_reduce()?.into_value().try_extract::<f64>()?,
      ),
      unit:       match column.dtype() {
        polars::prelude::DataType::Duration(_) => RangeUnit::Duration,
        polars::prelude::DataType::Date => RangeUnit::Date,
        _ => RangeUnit::Absolute,
      },
      margin_min: true,
      margin_max: true,
    })
  }
}

pub struct NiceTicksIter {
  current:   f64,
  step:      f64,
  hi:        f64,
  precision: usize,
}

impl NiceTicksIter {
  fn new(lo: f64, hi: f64, step: f64, precision: usize) -> Self {
    NiceTicksIter { current: lo, step, hi, precision }
  }

  pub fn precision(&self) -> usize { self.precision }
}

impl Iterator for NiceTicksIter {
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
