use kurbo::{Affine, Cap, Line, Point, Stroke};
use parley::FontWeight;
use peniko::{Brush, Color};

use crate::{
  axes::{Axes, ScatterAxes},
  render::{Align, DrawText, Render},
};

mod axes;
mod bounds;
mod marker;
mod render;

pub mod theme;

pub use bounds::{Bounds, Range};
pub use marker::Marker;

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

#[derive(Default)]
pub struct Axis {
  title: Option<String>,
  min:   Option<f64>,
  max:   Option<f64>,
  ticks: Ticks,
}

#[derive(Default)]
pub enum Ticks {
  #[default]
  Auto,
  Fixed(usize),
  Labeled(Vec<String>),
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

  fn bounds(&self) -> Bounds {
    let bounds = self
      .axes
      .iter()
      .filter_map(|s| s.data_bounds())
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

  pub fn ticks_fixed(&mut self, count: usize) -> &mut Self {
    self.ticks = Ticks::Fixed(count);
    self
  }

  pub fn ticks_labeled(&mut self, labels: Vec<String>) -> &mut Self {
    self.ticks = Ticks::Labeled(labels);
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
        text: &format!("{:.*}", precision.saturating_sub(3), y),
        size: 12.0,
        position: Point { x: viewport.x.min - 15.0, y: vy },
        brush: TEXT_COLOR,
        horizontal_align: Align::End,
        vertical_align: Align::Center,
        ..Default::default()
      });
    }

    let precision;
    let iter = match self.x.ticks {
      Ticks::Auto => {
        let iter = data_bounds.x.nice_ticks(ticks);
        precision = iter.precision();
        iter.collect::<Vec<_>>()
      }
      Ticks::Fixed(count) => {
        let mut ticks = vec![0.0; count];
        precision = 5;
        for i in 0..count {
          ticks[i] =
            data_bounds.x.min + (i as f64 / (count - 1) as f64) * data_bounds.x.size() as f64;
        }
        ticks
      }
      _ => todo!(),
    };
    for (x, vx) in iter
      .into_iter()
      .map(|v| (v, (transform * Point::new(v, 0.0)).x))
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
        text: &format!("{:.*}", precision.saturating_sub(3), x),
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
  }
}
