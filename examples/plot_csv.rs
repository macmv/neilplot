use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let df = LazyCsvReader::new(PlPath::new("examples/foo.csv")).finish()?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("X Axis");
  plot.y.title("Y Axis").min(0.0);

  let all = df.clone().collect()?;
  plot.scatter(all.column("a")?, all.column("b")?);

  let filtered = df.filter(col("a").gt_eq(lit(2))).collect()?;
  plot.line(filtered.column("a")?, filtered.column("b")?);

  plot.show();

  Ok(())
}
