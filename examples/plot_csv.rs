use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let file = std::fs::File::open("examples/foo.csv")?;
  let df = CsvReader::new(file).finish()?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x_label("X Axis");
  plot.y_label("Y Axis");
  plot.line(df.column("a")?, df.column("b")?, "Line Plot");

  plot.save("examples/plot_csv.png");

  Ok(())
}
