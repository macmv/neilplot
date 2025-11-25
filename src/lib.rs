use parley::FontWeight;
use polars::prelude::Column;
use vello::{kurbo::Point, peniko::Color};

use crate::render::{DrawText, Render};

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
    if let Some(title) = &self.title {
      render.draw_text(DrawText {
        text: title,
        size: 32.0,
        weight: FontWeight::BOLD,
        brush: Color::BLACK.into(),
        position: Point { x: 512.0, y: 20.0 },
        ..Default::default()
      });
    }
  }
}
