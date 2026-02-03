use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use polars::prelude::*;
use std::path::PathBuf;

mod filters;
mod transforms;

use filters::FilterSpec;
use transforms::TransformSpec;

/// A CLI tool for filtering and transforming Parquet files
#[derive(Parser, Debug)]
#[command(name = "pqfilter")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Filter rows based on column conditions
    Filter {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path (supports .parquet, .csv, .json)
        #[arg(short, long)]
        output: PathBuf,

        /// Filter expressions in format: column:operator:value
        /// Operators: eq, ne, gt, ge, lt, le, contains, startswith, endswith, isnull, notnull, in, between
        /// Examples: "age:gt:30", "name:contains:John", "status:in:active,pending", "score:between:10,100"
        #[arg(short, long, num_args = 1..)]
        filter: Vec<String>,

        /// Combine filters with AND (default) or OR
        #[arg(long, default_value = "and")]
        combine: CombineMode,
    },

    /// Select specific columns from the dataset
    Select {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Columns to select (comma-separated or multiple --columns flags)
        #[arg(short, long, num_args = 1.., value_delimiter = ',')]
        columns: Vec<String>,

        /// Exclude these columns instead of selecting them
        #[arg(long, default_value = "false")]
        exclude: bool,
    },

    /// Transform column values
    Transform {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Transform expressions in format: column:operation[:args]
        /// Operations: uppercase, lowercase, trim, round:decimals, abs, cast:dtype, rename:newname, fill_null:value
        #[arg(short, long, num_args = 1..)]
        transform: Vec<String>,
    },

    /// Sort the dataset by one or more columns
    Sort {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Columns to sort by (prefix with - for descending)
        /// Example: "age", "-created_at", "name,-score"
        #[arg(short, long, num_args = 1.., value_delimiter = ',')]
        by: Vec<String>,

        /// Use null-first ordering
        #[arg(long, default_value = "false")]
        nulls_first: bool,
    },

    /// Aggregate data with groupby operations
    Aggregate {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Columns to group by
        #[arg(short, long, num_args = 1.., value_delimiter = ',')]
        group_by: Vec<String>,

        /// Aggregation expressions: column:operation
        /// Operations: sum, mean, min, max, count, first, last, std, var, median
        #[arg(short, long, num_args = 1..)]
        agg: Vec<String>,
    },

    /// Show dataset schema and statistics
    Info {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Show detailed statistics
        #[arg(long, default_value = "false")]
        stats: bool,

        /// Show first N rows
        #[arg(long)]
        head: Option<usize>,
    },

    /// Sample rows from the dataset
    Sample {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Number of rows to sample
        #[arg(short, long)]
        n: Option<usize>,

        /// Fraction of rows to sample (0.0-1.0)
        #[arg(long)]
        fraction: Option<f64>,

        /// Random seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Apply multiple operations from a JSON config file
    Pipeline {
        /// Input Parquet file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// JSON configuration file with pipeline steps
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum CombineMode {
    #[default]
    And,
    Or,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Filter {
            input,
            output,
            filter,
            combine,
        } => cmd_filter(input, output, filter, combine),

        Commands::Select {
            input,
            output,
            columns,
            exclude,
        } => cmd_select(input, output, columns, exclude),

        Commands::Transform {
            input,
            output,
            transform,
        } => cmd_transform(input, output, transform),

        Commands::Sort {
            input,
            output,
            by,
            nulls_first,
        } => cmd_sort(input, output, by, nulls_first),

        Commands::Aggregate {
            input,
            output,
            group_by,
            agg,
        } => cmd_aggregate(input, output, group_by, agg),

        Commands::Info { input, stats, head } => cmd_info(input, stats, head),

        Commands::Sample {
            input,
            output,
            n,
            fraction,
            seed,
        } => cmd_sample(input, output, n, fraction, seed),

        Commands::Pipeline {
            input,
            output,
            config,
        } => cmd_pipeline(input, output, config),
    }
}

fn read_parquet(path: &PathBuf) -> Result<LazyFrame> {
    LazyFrame::scan_parquet(path, Default::default())
        .with_context(|| format!("Failed to read Parquet file: {}", path.display()))
}

fn write_output(df: DataFrame, path: &PathBuf) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("parquet");

    match extension.to_lowercase().as_str() {
        "parquet" => {
            let mut file = std::fs::File::create(path)?;
            ParquetWriter::new(&mut file).finish(&mut df.clone())?;
        }
        "csv" => {
            let mut file = std::fs::File::create(path)?;
            CsvWriter::new(&mut file).finish(&mut df.clone())?;
        }
        "json" => {
            let mut file = std::fs::File::create(path)?;
            JsonWriter::new(&mut file).finish(&mut df.clone())?;
        }
        _ => {
            anyhow::bail!("Unsupported output format: {extension}. Use .parquet, .csv, or .json");
        }
    }

    println!("✓ Written {} rows to {}", df.height(), path.display());
    Ok(())
}

fn cmd_filter(
    input: PathBuf,
    output: PathBuf,
    filters: Vec<String>,
    combine: CombineMode,
) -> Result<()> {
    let lf = read_parquet(&input)?;

    let filter_specs: Vec<FilterSpec> = filters
        .iter()
        .map(|f| FilterSpec::parse(f))
        .collect::<Result<Vec<_>>>()?;

    let mut combined_expr: Option<Expr> = None;

    for spec in filter_specs {
        let expr = spec.to_expr()?;
        combined_expr = Some(match (&combined_expr, &combine) {
            (None, _) => expr,
            (Some(existing), CombineMode::And) => existing.clone().and(expr),
            (Some(existing), CombineMode::Or) => existing.clone().or(expr),
        });
    }

    let result = if let Some(expr) = combined_expr {
        lf.filter(expr).collect()?
    } else {
        lf.collect()?
    };

    write_output(result, &output)
}

fn cmd_select(input: PathBuf, output: PathBuf, columns: Vec<String>, exclude: bool) -> Result<()> {
    let lf = read_parquet(&input)?;

    let result = if exclude {
        let schema = lf.clone().collect_schema()?;
        let keep_cols: Vec<_> = schema
            .iter_names()
            .filter(|name| !columns.contains(&name.to_string()))
            .map(|name| col(name.as_str()))
            .collect();
        lf.select(keep_cols).collect()?
    } else {
        let select_cols: Vec<_> = columns.iter().map(|c| col(c.as_str())).collect();
        lf.select(select_cols).collect()?
    };

    write_output(result, &output)
}

fn cmd_transform(input: PathBuf, output: PathBuf, transforms: Vec<String>) -> Result<()> {
    let lf = read_parquet(&input)?;

    let transform_specs: Vec<TransformSpec> = transforms
        .iter()
        .map(|t| TransformSpec::parse(t))
        .collect::<Result<Vec<_>>>()?;

    let mut result_lf = lf;
    for spec in transform_specs {
        result_lf = spec.apply(result_lf)?;
    }

    write_output(result_lf.collect()?, &output)
}

fn cmd_sort(input: PathBuf, output: PathBuf, by: Vec<String>, nulls_first: bool) -> Result<()> {
    let lf = read_parquet(&input)?;

    let mut sort_exprs = Vec::new();
    let mut descending = Vec::new();

    for col_spec in &by {
        let (col_name, desc) = if let Some(name) = col_spec.strip_prefix('-') {
            (name, true)
        } else {
            (col_spec.as_str(), false)
        };

        sort_exprs.push(col(col_name));
        descending.push(desc);
    }

    let sort_options = SortMultipleOptions::default()
        .with_order_descending_multi(descending)
        .with_nulls_last(!nulls_first);

    let result = lf.sort_by_exprs(sort_exprs, sort_options).collect()?;

    write_output(result, &output)
}

fn cmd_aggregate(
    input: PathBuf,
    output: PathBuf,
    group_by: Vec<String>,
    agg: Vec<String>,
) -> Result<()> {
    let lf = read_parquet(&input)?;

    let group_cols: Vec<_> = group_by.iter().map(|c| col(c.as_str())).collect();

    let agg_exprs: Vec<Expr> = agg
        .iter()
        .map(|a| {
            let parts: Vec<&str> = a.splitn(2, ':').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid aggregation format: {a}. Use column:operation");
            }
            let (col_name, op) = (parts[0], parts[1]);
            let c = col(col_name);

            Ok(match op.to_lowercase().as_str() {
                "sum" => c.sum().alias(format!("{col_name}_sum")),
                "mean" | "avg" => c.mean().alias(format!("{col_name}_mean")),
                "min" => c.min().alias(format!("{col_name}_min")),
                "max" => c.max().alias(format!("{col_name}_max")),
                "count" => c.count().alias(format!("{col_name}_count")),
                "first" => c.first().alias(format!("{col_name}_first")),
                "last" => c.last().alias(format!("{col_name}_last")),
                "std" => c.std(1).alias(format!("{col_name}_std")),
                "var" => c.var(1).alias(format!("{col_name}_var")),
                "median" => c.median().alias(format!("{col_name}_median")),
                _ => anyhow::bail!("Unknown aggregation: {op}"),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let result = lf.group_by(group_cols).agg(agg_exprs).collect()?;

    write_output(result, &output)
}

fn cmd_info(input: PathBuf, stats: bool, head: Option<usize>) -> Result<()> {
    let lf = read_parquet(&input)?;
    let df = lf.clone().collect()?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    PARQUET FILE INFO                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ File: {:<55} ║", input.display());
    println!("║ Rows: {:<55} ║", df.height());
    println!("║ Columns: {:<52} ║", df.width());
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\n📋 Schema:");
    println!("┌────────────────────────────────┬────────────────────────────────┐");
    println!("│ Column                         │ Type                           │");
    println!("├────────────────────────────────┼────────────────────────────────┤");
    for field in df.schema().iter_fields() {
        println!("│ {:<30} │ {:<30} │", field.name(), field.dtype());
    }
    println!("└────────────────────────────────┴────────────────────────────────┘");

    if stats {
        println!("\n📊 Statistics:");
        println!("┌────────────────────────────────┬──────────────┬──────────────┬──────────────┐");
        println!("│ Column                         │ Null Count   │ Min          │ Max          │");
        println!("├────────────────────────────────┼──────────────┼──────────────┼──────────────┤");
        for name in df.get_column_names() {
            let series = df.column(name)?;
            let null_count = series.null_count();
            let dtype = series.dtype();
            let (min_str, max_str) = if dtype.is_primitive_numeric() {
                let min = series
                    .min_reduce()
                    .map(|s| format!("{:?}", s.value()))
                    .unwrap_or_else(|_| "N/A".to_string());
                let max = series
                    .max_reduce()
                    .map(|s| format!("{:?}", s.value()))
                    .unwrap_or_else(|_| "N/A".to_string());
                (min, max)
            } else {
                ("N/A".to_string(), "N/A".to_string())
            };
            println!(
                "│ {:<30} │ {:<12} │ {:<12} │ {:<12} │",
                name, null_count, min_str, max_str
            );
        }
        println!("└────────────────────────────────┴──────────────┴──────────────┴──────────────┘");
    }

    if let Some(n) = head {
        println!("\n🔍 First {} rows:", n);
        println!("{}", df.head(Some(n)));
    }

    Ok(())
}

fn cmd_sample(
    input: PathBuf,
    output: PathBuf,
    n: Option<usize>,
    fraction: Option<f64>,
    seed: Option<u64>,
) -> Result<()> {
    let lf = read_parquet(&input)?;
    let df = lf.collect()?;

    let result = match (n, fraction) {
        (Some(n), _) => df.sample_n_literal(n, false, false, seed)?,
        (None, Some(frac)) => df.sample_frac(
            &Series::new(PlSmallStr::from("frac"), &[frac]),
            false,
            false,
            seed,
        )?,
        (None, None) => anyhow::bail!("Must specify either --n or --fraction"),
    };

    write_output(result, &output)
}

fn cmd_pipeline(input: PathBuf, output: PathBuf, config: PathBuf) -> Result<()> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct PipelineConfig {
        steps: Vec<PipelineStep>,
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum PipelineStep {
        #[serde(rename = "filter")]
        Filter {
            filters: Vec<String>,
            #[serde(default)]
            combine: String,
        },
        #[serde(rename = "select")]
        Select {
            columns: Vec<String>,
            #[serde(default)]
            exclude: bool,
        },
        #[serde(rename = "transform")]
        Transform { transforms: Vec<String> },
        #[serde(rename = "sort")]
        Sort {
            by: Vec<String>,
            #[serde(default)]
            nulls_first: bool,
        },
        #[serde(rename = "sample")]
        Sample {
            n: Option<usize>,
            fraction: Option<f64>,
            seed: Option<u64>,
        },
    }

    let config_content = std::fs::read_to_string(&config)
        .with_context(|| format!("Failed to read config file: {}", config.display()))?;
    let pipeline: PipelineConfig =
        serde_json::from_str(&config_content).with_context(|| "Failed to parse pipeline config")?;

    let mut lf = read_parquet(&input)?;

    for (i, step) in pipeline.steps.iter().enumerate() {
        println!("⚙️  Executing step {}...", i + 1);

        match step {
            PipelineStep::Filter { filters, combine } => {
                let filter_specs: Vec<FilterSpec> = filters
                    .iter()
                    .map(|f| FilterSpec::parse(f))
                    .collect::<Result<Vec<_>>>()?;

                let mut combined_expr: Option<Expr> = None;
                let use_and = combine.to_lowercase() != "or";

                for spec in filter_specs {
                    let expr = spec.to_expr()?;
                    combined_expr = Some(match &combined_expr {
                        None => expr,
                        Some(existing) if use_and => existing.clone().and(expr),
                        Some(existing) => existing.clone().or(expr),
                    });
                }

                if let Some(expr) = combined_expr {
                    lf = lf.filter(expr);
                }
            }

            PipelineStep::Select { columns, exclude } => {
                if *exclude {
                    let schema = lf.clone().collect_schema()?;
                    let keep_cols: Vec<_> = schema
                        .iter_names()
                        .filter(|name| !columns.contains(&name.to_string()))
                        .map(|name| col(name.as_str()))
                        .collect();
                    lf = lf.select(keep_cols);
                } else {
                    let select_cols: Vec<_> = columns.iter().map(|c| col(c.as_str())).collect();
                    lf = lf.select(select_cols);
                }
            }

            PipelineStep::Transform { transforms } => {
                let transform_specs: Vec<TransformSpec> = transforms
                    .iter()
                    .map(|t| TransformSpec::parse(t))
                    .collect::<Result<Vec<_>>>()?;

                for spec in transform_specs {
                    lf = spec.apply(lf)?;
                }
            }

            PipelineStep::Sort { by, nulls_first } => {
                let mut sort_exprs = Vec::new();
                let mut descending_vec = Vec::new();

                for col_spec in by {
                    let (col_name, desc) = if let Some(name) = col_spec.strip_prefix('-') {
                        (name, true)
                    } else {
                        (col_spec.as_str(), false)
                    };

                    sort_exprs.push(col(col_name));
                    descending_vec.push(desc);
                }

                let sort_options = SortMultipleOptions::default()
                    .with_order_descending_multi(descending_vec)
                    .with_nulls_last(!nulls_first);

                lf = lf.sort_by_exprs(sort_exprs, sort_options);
            }

            PipelineStep::Sample { n, fraction, seed } => {
                let df = lf.collect()?;
                let sampled = match (n, fraction) {
                    (Some(n), _) => df.sample_n_literal(*n, false, false, *seed)?,
                    (None, Some(frac)) => df.sample_frac(
                        &Series::new(PlSmallStr::from("frac"), &[*frac]),
                        false,
                        false,
                        *seed,
                    )?,
                    (None, None) => anyhow::bail!("Sample step requires either n or fraction"),
                };
                lf = sampled.lazy();
            }
        }
    }

    let result = lf.collect()?;
    write_output(result, &output)
}
