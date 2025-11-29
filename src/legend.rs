use kurbo::{Affine, Point, Rect, RoundedRect, Stroke, Vec2};
use peniko::Brush;

use crate::{
  Axes, Bounds, LineOptions, Marker, Plot,
  render::{Align, DrawText, Render},
};

pub struct Legend {
  items: Vec<LegendItem>,
}

pub struct LegendItem {
  label:  String,
  line:   Option<LineOptions>,
  marker: Option<Marker>,
  color:  Brush,
}

impl Plot<'_> {
  pub(crate) fn draw_legend(&self, render: &mut Render, viewport: Bounds) {
    let mut items = vec![];
    for ax in &self.axes {
      match ax {
        Axes::Scatter(sa) => {
          items.push(LegendItem {
            label:  "scatter".to_string(),
            line:   None,
            marker: Some(sa.options.marker),
            color:  sa.options.color.clone(),
          });

          if let Some(trendline) = &sa.options.trendline {
            items.push(LegendItem {
              label: "trendline".to_string(),

              line:   Some(trendline.line.clone()),
              marker: None,

              color: trendline.line.color.clone(),
            });
          }
        }
        _ => {}
      }
    }

    let legend = Legend { items };

    const MARGIN: f64 = 20.0;
    const PADDING: f64 = 10.0;
    const GAP: f64 = 10.0;
    const FONT_SIZE: f64 = 20.0;
    const LINE_HEIGHT: f64 = 20.0;
    const MARKER_WIDTH: f64 = 40.0;

    let mut inner_width = 0.0_f64;
    let mut layouts = vec![];
    for item in &legend.items {
      let text = DrawText {
        text: &item.label,
        size: FONT_SIZE as f32,
        vertical_align: Align::Center,
        ..Default::default()
      };
      let layout = render.layout_text(&text);
      inner_width = inner_width.max(f64::from(layout.width()));
      layouts.push((layout, text));
    }

    inner_width += MARKER_WIDTH;
    let inner_height = legend.items.len() as f64 * LINE_HEIGHT;

    let rect = Rect::new(
      viewport.x.max - inner_width - MARGIN - PADDING * 2.0 - GAP,
      viewport.y.min - inner_height - MARGIN - PADDING * 2.0,
      viewport.x.max - MARGIN,
      viewport.y.min - MARGIN,
    );
    let background = RoundedRect::from_rect(rect, 5.0);
    render.fill(
      &background,
      Affine::IDENTITY,
      &Brush::Solid(peniko::Color::from_rgba8(255, 255, 255, 200)),
    );
    render.stroke(
      &background,
      Affine::IDENTITY,
      &Brush::Solid(peniko::Color::from_rgb8(128, 128, 128)),
      &Stroke::new(2.0),
    );

    for (i, (layout, mut text)) in layouts.into_iter().enumerate() {
      let pos = Point::new(
        rect.x0 + PADDING,
        rect.y0 + i as f64 * LINE_HEIGHT + PADDING + LINE_HEIGHT / 2.0,
      );

      if let Some(line_opts) = &legend.items[i].line {
        render.stroke(
          &kurbo::Line::new(pos, pos + Vec2::new(MARKER_WIDTH, 0.0)),
          Affine::IDENTITY,
          &legend.items[i].color,
          &line_opts.stroke(),
        );
      }

      if let Some(marker) = &legend.items[i].marker {
        render.fill(
          &marker.to_path(0.1),
          Affine::scale(10.0).then_translate(pos.to_vec2() + Vec2::new(MARKER_WIDTH / 2.0, 0.0)),
          &legend.items[i].color,
        );
      }

      text.position = pos + Vec2::new(MARKER_WIDTH + GAP, 0.0);
      render.draw_text_layout(layout, text);
    }
  }
}
