use kurbo::{Affine, Point, Rect, RoundedRect, Size, Stroke, Vec2};
use peniko::Brush;

use crate::{
  Axes, Bounds, Plot,
  render::{Align, DrawText, Render},
};

pub struct Legend {
  items: Vec<LegendItem>,
}

pub struct LegendItem {
  label: String,
  color: Brush,
}

impl Plot<'_> {
  pub(crate) fn draw_legend(&self, render: &mut Render, viewport: Bounds) {
    let mut items = vec![];
    for ax in &self.axes {
      match ax {
        Axes::Scatter(sa) => {
          items.push(LegendItem { label: "scatter".to_string(), color: sa.options.color.clone() });

          if let Some(trendline) = &sa.options.trendline {
            items
              .push(LegendItem { label: "trendline".to_string(), color: trendline.color.clone() });
          }
        }
        _ => {}
      }
    }

    let legend = Legend { items };

    const MARGIN: f64 = 20.0;
    const PADDING: f64 = 10.0;
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
      viewport.x.max - inner_width - MARGIN - PADDING * 2.0,
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

      let marker_rect =
        Rect::from_origin_size(pos - Vec2::new(0.0, 1.0), Size::new(MARKER_WIDTH - 5.0, 2.0));
      render.fill(&marker_rect, Affine::IDENTITY, &legend.items[i].color);

      text.position = pos + Vec2::new(MARKER_WIDTH, 0.0);
      render.draw_text_layout(layout, text);
    }
  }
}
