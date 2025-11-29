use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let df = df! {
    "label" => &["A", "B", "C", "D"],
    "value" => &[10, 20, 15, 25],
  }?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("Label");
  plot.y.title("Counts");

  plot.bar_chart(df.column("label")?, df.column("value")?);

  plot.show();

  Ok(())
}
