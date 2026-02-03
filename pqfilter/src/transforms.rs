use anyhow::{Context, Result};
use polars::prelude::*;

/// Represents a parsed transform specification
#[derive(Debug, Clone)]
pub struct TransformSpec {
    pub column: String,
    pub operation: TransformOperation,
}

#[derive(Debug, Clone)]
pub enum TransformOperation {
    /// Convert string to uppercase
    Uppercase,
    /// Convert string to lowercase
    Lowercase,
    /// Trim whitespace from strings
    Trim,
    /// Round numeric values to N decimal places
    Round(u32),
    /// Take absolute value
    Abs,
    /// Cast to a different data type
    Cast(DataType),
    /// Rename the column
    Rename(String),
    /// Fill null values with a default
    FillNull(String),
    /// Replace values matching a pattern
    Replace(String, String),
    /// Extract substring
    Substring(i64, Option<u64>),
    /// Add a constant value
    Add(f64),
    /// Multiply by a constant
    Multiply(f64),
    /// Apply log transformation
    Log,
    /// Apply natural log
    Ln,
    /// Square root
    Sqrt,
    /// Clip values to a range
    Clip(f64, f64),
    /// Convert to date
    ToDate(String),
    /// Convert to datetime
    ToDatetime(String),
    /// Extract year from date
    Year,
    /// Extract month from date
    Month,
    /// Extract day from date
    Day,
}

impl TransformSpec {
    /// Parse a transform string in format: column:operation[:args]
    /// Examples:
    ///   - "name:uppercase"
    ///   - "price:round:2"
    ///   - "old_name:rename:new_name"
    ///   - "value:fill_null:0"
    ///   - "score:clip:0,100"
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();

        if parts.len() < 2 {
            anyhow::bail!(
                "Invalid transform format: '{s}'. Expected column:operation or column:operation:args"
            );
        }

        let column = parts[0].to_string();
        let op_str = parts[1].to_lowercase();
        let args = parts.get(2).map(|s| s.to_string());

        let operation = match op_str.as_str() {
            "uppercase" | "upper" => TransformOperation::Uppercase,
            "lowercase" | "lower" => TransformOperation::Lowercase,
            "trim" => TransformOperation::Trim,

            "round" => {
                let decimals: u32 = args
                    .context("'round' requires decimal places")?
                    .parse()
                    .context("Invalid decimal places for round")?;
                TransformOperation::Round(decimals)
            }

            "abs" => TransformOperation::Abs,

            "cast" => {
                let dtype_str = args.context("'cast' requires a target type")?;
                let dtype = parse_dtype(&dtype_str)?;
                TransformOperation::Cast(dtype)
            }

            "rename" => {
                let new_name = args.context("'rename' requires a new column name")?;
                TransformOperation::Rename(new_name)
            }

            "fill_null" | "fillna" | "coalesce" => {
                let default = args.context("'fill_null' requires a default value")?;
                TransformOperation::FillNull(default)
            }

            "replace" => {
                let replace_args = args.context("'replace' requires pattern,replacement")?;
                let replace_parts: Vec<&str> = replace_args.split(',').collect();
                if replace_parts.len() != 2 {
                    anyhow::bail!("'replace' requires exactly two arguments: pattern,replacement");
                }
                TransformOperation::Replace(
                    replace_parts[0].to_string(),
                    replace_parts[1].to_string(),
                )
            }

            "substring" | "substr" => {
                let sub_args = args.context("'substring' requires start[,length]")?;
                let sub_parts: Vec<&str> = sub_args.split(',').collect();
                let start: i64 = sub_parts[0].parse().context("Invalid start index")?;
                let length: Option<u64> = if sub_parts.len() > 1 {
                    Some(sub_parts[1].parse().context("Invalid length")?)
                } else {
                    None
                };
                TransformOperation::Substring(start, length)
            }

            "add" | "+" => {
                let val: f64 = args
                    .context("'add' requires a value")?
                    .parse()
                    .context("Invalid numeric value for add")?;
                TransformOperation::Add(val)
            }

            "multiply" | "mul" | "*" => {
                let val: f64 = args
                    .context("'multiply' requires a value")?
                    .parse()
                    .context("Invalid numeric value for multiply")?;
                TransformOperation::Multiply(val)
            }

            "log" | "log10" => TransformOperation::Log,
            "ln" => TransformOperation::Ln,
            "sqrt" => TransformOperation::Sqrt,

            "clip" | "clamp" => {
                let clip_args = args.context("'clip' requires min,max values")?;
                let clip_parts: Vec<&str> = clip_args.split(',').collect();
                if clip_parts.len() != 2 {
                    anyhow::bail!("'clip' requires exactly two arguments: min,max");
                }
                let min: f64 = clip_parts[0].parse().context("Invalid min value")?;
                let max: f64 = clip_parts[1].parse().context("Invalid max value")?;
                TransformOperation::Clip(min, max)
            }

            "to_date" | "todate" => {
                let fmt = args.unwrap_or_else(|| "%Y-%m-%d".to_string());
                TransformOperation::ToDate(fmt)
            }

            "to_datetime" | "todatetime" => {
                let fmt = args.unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string());
                TransformOperation::ToDatetime(fmt)
            }

            "year" => TransformOperation::Year,
            "month" => TransformOperation::Month,
            "day" => TransformOperation::Day,

            _ => anyhow::bail!("Unknown transform operation: '{op_str}'"),
        };

        Ok(TransformSpec { column, operation })
    }

    /// Apply the transform to a LazyFrame
    pub fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        let c = col(&self.column);
        let column_name = &self.column;

        let result = match &self.operation {
            TransformOperation::Uppercase => {
                lf.with_column(c.str().to_uppercase().alias(column_name))
            }

            TransformOperation::Lowercase => {
                lf.with_column(c.str().to_lowercase().alias(column_name))
            }

            TransformOperation::Trim => {
                lf.with_column(c.str().strip_chars(lit("")).alias(column_name))
            }

            TransformOperation::Round(decimals) => lf.with_column(
                c.round(*decimals, RoundMode::HalfAwayFromZero)
                    .alias(column_name),
            ),

            TransformOperation::Abs => lf.with_column(c.abs().alias(column_name)),

            TransformOperation::Cast(dtype) => {
                lf.with_column(c.cast(dtype.clone()).alias(column_name))
            }

            TransformOperation::Rename(new_name) => {
                lf.rename([column_name], [new_name.as_str()], true)
            }

            TransformOperation::FillNull(default) => {
                // Try to parse as appropriate type
                let fill_expr = if let Ok(v) = default.parse::<i64>() {
                    c.fill_null(lit(v))
                } else if let Ok(v) = default.parse::<f64>() {
                    c.fill_null(lit(v))
                } else if default.eq_ignore_ascii_case("true") {
                    c.fill_null(lit(true))
                } else if default.eq_ignore_ascii_case("false") {
                    c.fill_null(lit(false))
                } else {
                    c.fill_null(lit(default.clone()))
                };
                lf.with_column(fill_expr.alias(column_name))
            }

            TransformOperation::Replace(pattern, replacement) => lf.with_column(
                c.str()
                    .replace_all(lit(pattern.clone()), lit(replacement.clone()), true)
                    .alias(column_name),
            ),

            TransformOperation::Substring(start, length) => {
                let expr = match length {
                    Some(len) => c.str().slice(lit(*start), lit(*len as i64)),
                    None => c.str().slice(lit(*start), lit(i64::MAX)),
                };
                lf.with_column(expr.alias(column_name))
            }

            TransformOperation::Add(val) => lf.with_column((c + lit(*val)).alias(column_name)),

            TransformOperation::Multiply(val) => lf.with_column((c * lit(*val)).alias(column_name)),

            TransformOperation::Log => lf.with_column(c.log(10.0).alias(column_name)),

            TransformOperation::Ln => lf.with_column(c.log(std::f64::consts::E).alias(column_name)),

            TransformOperation::Sqrt => lf.with_column(c.sqrt().alias(column_name)),

            TransformOperation::Clip(min, max) => {
                // Implement clip as: when(c < min, min).when(c > max, max).otherwise(c)
                let clipped = when(c.clone().lt(lit(*min)))
                    .then(lit(*min))
                    .when(c.clone().gt(lit(*max)))
                    .then(lit(*max))
                    .otherwise(c);
                lf.with_column(clipped.alias(column_name))
            }

            TransformOperation::ToDate(fmt) => lf.with_column(
                c.str()
                    .to_date(StrptimeOptions {
                        format: Some(PlSmallStr::from(fmt.as_str())),
                        ..Default::default()
                    })
                    .alias(column_name),
            ),

            TransformOperation::ToDatetime(fmt) => lf.with_column(
                c.str()
                    .to_datetime(
                        None,
                        None,
                        StrptimeOptions {
                            format: Some(PlSmallStr::from(fmt.as_str())),
                            ..Default::default()
                        },
                        lit("raise"),
                    )
                    .alias(column_name),
            ),

            TransformOperation::Year => {
                lf.with_column(c.dt().year().alias(format!("{column_name}_year")))
            }

            TransformOperation::Month => {
                lf.with_column(c.dt().month().alias(format!("{column_name}_month")))
            }

            TransformOperation::Day => {
                lf.with_column(c.dt().day().alias(format!("{column_name}_day")))
            }
        };

        Ok(result)
    }
}

/// Parse a data type string into a Polars DataType
fn parse_dtype(s: &str) -> Result<DataType> {
    let dtype = match s.to_lowercase().as_str() {
        "int8" | "i8" => DataType::Int8,
        "int16" | "i16" => DataType::Int16,
        "int32" | "i32" | "int" => DataType::Int32,
        "int64" | "i64" => DataType::Int64,
        "uint8" | "u8" => DataType::UInt8,
        "uint16" | "u16" => DataType::UInt16,
        "uint32" | "u32" => DataType::UInt32,
        "uint64" | "u64" => DataType::UInt64,
        "float32" | "f32" | "float" => DataType::Float32,
        "float64" | "f64" | "double" => DataType::Float64,
        "bool" | "boolean" => DataType::Boolean,
        "string" | "str" | "utf8" => DataType::String,
        "date" => DataType::Date,
        _ => anyhow::bail!("Unknown data type: '{s}'"),
    };
    Ok(dtype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_transform() {
        let transform = TransformSpec::parse("name:uppercase").unwrap();
        assert_eq!(transform.column, "name");
        assert!(matches!(transform.operation, TransformOperation::Uppercase));
    }

    #[test]
    fn test_parse_round_transform() {
        let transform = TransformSpec::parse("price:round:2").unwrap();
        assert_eq!(transform.column, "price");
        if let TransformOperation::Round(decimals) = transform.operation {
            assert_eq!(decimals, 2);
        } else {
            panic!("Expected Round operation");
        }
    }

    #[test]
    fn test_parse_rename_transform() {
        let transform = TransformSpec::parse("old_col:rename:new_col").unwrap();
        assert_eq!(transform.column, "old_col");
        if let TransformOperation::Rename(new_name) = transform.operation {
            assert_eq!(new_name, "new_col");
        } else {
            panic!("Expected Rename operation");
        }
    }

    #[test]
    fn test_parse_clip_transform() {
        let transform = TransformSpec::parse("score:clip:0,100").unwrap();
        assert_eq!(transform.column, "score");
        if let TransformOperation::Clip(min, max) = transform.operation {
            assert_eq!(min, 0.0);
            assert_eq!(max, 100.0);
        } else {
            panic!("Expected Clip operation");
        }
    }
}
