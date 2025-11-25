use polars::prelude::Column;

pub struct Plot {}

#[derive(Default)]
pub struct PlotBuilder<'a> {
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

impl Plot {
  pub fn new() -> PlotBuilder<'static> { PlotBuilder::default() }
}

impl<'a> PlotBuilder<'a> {
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

  pub fn build(self) -> Plot { Plot {} }
}
