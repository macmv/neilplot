use neilplot::Plot;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
  let column = ChunkedArray::<Float64Type>::rand_standard_normal("rand".into(), 1000);
  let df = DataFrame::new(vec![column.into_series().into()])?;

  let mut plot = Plot::new();
  plot.title("Foo");
  plot.x.title("Label");
  plot.y.title("Counts");

  let df = df
    .lazy()
    .with_column((col("rand") * lit(10)).floor().cast(DataType::Int32).alias("rand"))
    .group_by([col("rand")])
    .agg([col("rand").count().alias("counts")])
    .sort(["rand"], SortMultipleOptions::new())
    .collect()?;

  plot.histogram(df.column("counts")?);

  plot.show();

  Ok(())
}
