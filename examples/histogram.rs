use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let column = ChunkedArray::<Float64Type>::rand_standard_normal("rand".into(), 1000);
  let df = DataFrame::new(vec![column.into_series().into()])?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("Label");
  plot.y.title("Counts");

  plot.histogram(df.column("rand")?, 30);

  plot.show();

  Ok(())
}
