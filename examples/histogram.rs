use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let df = df! {
    "label" => &["A", "B", "C", "D", "E"],
    "counts" => &[5, 10, 3, 8, 2],
  }?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("Label");
  plot.y.title("Counts");

  plot.histogram(df.column("counts")?);

  plot.show();

  Ok(())
}
