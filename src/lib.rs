use parley::FontWeight;
use polars::prelude::Column;
use vello::{
  kurbo::Point,
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

struct Series<'a> {
  x:    &'a Column,
  y:    &'a Column,
  name: Option<String>,
  data: SeriesData,
}

struct SeriesData {}

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

  pub fn line(&mut self, x: &'a Column, y: &'a Column, name: &str) -> &mut Self {
    self.series.push(Series {
      x:    x,
      y:    y,
      name: Some(name.to_string()),
      data: SeriesData {},
    });
    self
  }

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

    render.draw_line(Point::new(50.0, 940.0), Point::new(950.0, 940.0), LINE_COLOR, 2.0);
    render.draw_line(Point::new(50.0, 940.0), Point::new(50.0, 50.0), LINE_COLOR, 2.0);
  }
}
