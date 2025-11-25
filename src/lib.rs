use parley::FontWeight;
use polars::prelude::Column;
use vello::{
  kurbo::{BezPath, Circle, Point, Stroke},
  peniko::{Brush, Color, Fill},
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
  line:   Option<SeriesLine>,
  points: Option<SeriesPoints>,
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
    self.series.push(Series {
      x:      x,
      y:      y,
      line:   Some(SeriesLine::default()),
      points: None,
    });
    self.series.last_mut().unwrap()
  }
}

impl Series<'_> {
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

    render.draw_line(Point::new(50.0, 950.0), Point::new(950.0, 950.0), &LINE_COLOR, 2.0);
    render.draw_line(Point::new(50.0, 950.0), Point::new(50.0, 50.0), &LINE_COLOR, 2.0);

    for series in &self.series {
      if let Some(line) = &series.line {
        let mut shape = BezPath::new();

        for i in 0..series.x.len() {
          let x = series.x.get(i).unwrap().try_extract::<f64>().unwrap();
          let y = series.y.get(i).unwrap().try_extract::<f64>().unwrap();
          let point = Point::new(50.0 + (x * 900.0 / 10.0), 950.0 - (y * 900.0 / 10.0));

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

        render.scene.stroke(&stroke, render.transform, &line.color, None, &shape);
      }

      for i in 0..series.x.len() {
        let x = series.x.get(i).unwrap().try_extract::<f64>().unwrap();
        let y = series.y.get(i).unwrap().try_extract::<f64>().unwrap();
        let plot_x = 50.0 + (x * 900.0 / 10.0);
        let plot_y = 950.0 - (y * 900.0 / 10.0);

        if let Some(points) = &series.points {
          render.scene.fill(
            Fill::NonZero,
            render.transform,
            &points.color,
            None,
            &Circle::new(Point { x: plot_x, y: plot_y }, points.size),
          );
        }
      }
    }
  }
}
