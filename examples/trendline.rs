use neilplot::{Plot, TrendlineKind};
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let df = df! {
    "x" => &[1, 2, 3, 4, 5],
    "y" => &[2.2, 2.5, 3.6, 4.7, 5.1],
  }?;

  let mut plot = Plot::new();
  plot.title("Foo");

  plot.scatter(df.column("x")?, df.column("y")?).trendline(TrendlineKind::LINEAR);

  plot.show();

  Ok(())
}
