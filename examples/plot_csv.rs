use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let file = std::fs::File::open("examples/foo.csv")?;
  let df = CsvReader::new(file).finish()?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("X Axis");
  plot.y.title("Y Axis").min(0.0);
  plot.axes(df.column("a")?, df.column("b")?).line().points();

  plot.show();

  Ok(())
}
