use anyhow::{Context, Result};
use polars::prelude::*;

/// Represents a parsed filter specification
#[derive(Debug, Clone)]
pub struct FilterSpec {
    pub column: String,
    pub operator: FilterOperator,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
    Contains,
    StartsWith,
    EndsWith,
    IsNull,
    NotNull,
    In(Vec<String>),
    Between(String, String),
    Regex,
}

impl FilterSpec {
    /// Parse a filter string in format: column:operator:value
    /// Examples:
    ///   - "age:gt:30"
    ///   - "name:contains:John"
    ///   - "status:in:active,pending,done"
    ///   - "score:between:10,100"
    ///   - "deleted_at:isnull"
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();

        if parts.len() < 2 {
            anyhow::bail!(
                "Invalid filter format: '{s}'. Expected column:operator or column:operator:value"
            );
        }

        let column = parts[0].to_string();
        let op_str = parts[1].to_lowercase();
        let value = parts.get(2).map(|v| v.to_string());

        let operator = match op_str.as_str() {
            "eq" | "=" | "==" => FilterOperator::Equal,
            "ne" | "!=" | "<>" => FilterOperator::NotEqual,
            "gt" | ">" => FilterOperator::GreaterThan,
            "ge" | ">=" => FilterOperator::GreaterEqual,
            "lt" | "<" => FilterOperator::LessThan,
            "le" | "<=" => FilterOperator::LessEqual,
            "contains" | "like" => FilterOperator::Contains,
            "startswith" | "starts" => FilterOperator::StartsWith,
            "endswith" | "ends" => FilterOperator::EndsWith,
            "isnull" | "null" => FilterOperator::IsNull,
            "notnull" | "!null" => FilterOperator::NotNull,
            "regex" | "~" => FilterOperator::Regex,
            "in" => {
                let vals = value
                    .as_ref()
                    .context("'in' operator requires values")?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                FilterOperator::In(vals)
            }
            "between" => {
                let vals: Vec<&str> = value
                    .as_ref()
                    .context("'between' operator requires two values")?
                    .split(',')
                    .collect();
                if vals.len() != 2 {
                    anyhow::bail!("'between' operator requires exactly two values: min,max");
                }
                FilterOperator::Between(vals[0].trim().to_string(), vals[1].trim().to_string())
            }
            _ => anyhow::bail!("Unknown operator: '{op_str}'"),
        };

        Ok(FilterSpec {
            column,
            operator,
            value,
        })
    }

    /// Convert the filter spec to a Polars expression
    pub fn to_expr(&self) -> Result<Expr> {
        let c = col(&self.column);

        let expr = match &self.operator {
            FilterOperator::Equal => {
                let val = self.value.as_ref().context("'eq' requires a value")?;
                self.smart_compare(c, val, |c, v| c.eq(v))?
            }

            FilterOperator::NotEqual => {
                let val = self.value.as_ref().context("'ne' requires a value")?;
                self.smart_compare(c, val, |c, v| c.neq(v))?
            }

            FilterOperator::GreaterThan => {
                let val = self.value.as_ref().context("'gt' requires a value")?;
                self.smart_compare(c, val, |c, v| c.gt(v))?
            }

            FilterOperator::GreaterEqual => {
                let val = self.value.as_ref().context("'ge' requires a value")?;
                self.smart_compare(c, val, |c, v| c.gt_eq(v))?
            }

            FilterOperator::LessThan => {
                let val = self.value.as_ref().context("'lt' requires a value")?;
                self.smart_compare(c, val, |c, v| c.lt(v))?
            }

            FilterOperator::LessEqual => {
                let val = self.value.as_ref().context("'le' requires a value")?;
                self.smart_compare(c, val, |c, v| c.lt_eq(v))?
            }

            FilterOperator::Contains => {
                let val = self.value.as_ref().context("'contains' requires a value")?;
                c.str().contains_literal(lit(val.clone()))
            }

            FilterOperator::StartsWith => {
                let val = self
                    .value
                    .as_ref()
                    .context("'startswith' requires a value")?;
                c.str().starts_with(lit(val.clone()))
            }

            FilterOperator::EndsWith => {
                let val = self.value.as_ref().context("'endswith' requires a value")?;
                c.str().ends_with(lit(val.clone()))
            }

            FilterOperator::IsNull => c.is_null(),

            FilterOperator::NotNull => c.is_not_null(),

            FilterOperator::Regex => {
                let val = self.value.as_ref().context("'regex' requires a pattern")?;
                c.str().contains(lit(val.clone()), false)
            }

            FilterOperator::In(values) => {
                // Try to parse as numbers first, fall back to strings
                let series = if values.iter().all(|v| v.parse::<f64>().is_ok()) {
                    let nums: Vec<f64> = values.iter().map(|v| v.parse().unwrap()).collect();
                    Series::new(PlSmallStr::from("in_values"), nums)
                } else if values.iter().all(|v| v.parse::<i64>().is_ok()) {
                    let nums: Vec<i64> = values.iter().map(|v| v.parse().unwrap()).collect();
                    Series::new(PlSmallStr::from("in_values"), nums)
                } else {
                    Series::new(PlSmallStr::from("in_values"), values.clone())
                };
                c.is_in(lit(series), false)
            }

            FilterOperator::Between(min, max) => {
                // Try to parse as numbers
                if let (Ok(min_f), Ok(max_f)) = (min.parse::<f64>(), max.parse::<f64>()) {
                    c.clone().gt_eq(lit(min_f)).and(c.lt_eq(lit(max_f)))
                } else if let (Ok(min_i), Ok(max_i)) = (min.parse::<i64>(), max.parse::<i64>()) {
                    c.clone().gt_eq(lit(min_i)).and(c.lt_eq(lit(max_i)))
                } else {
                    // Treat as strings
                    c.clone()
                        .gt_eq(lit(min.clone()))
                        .and(c.lt_eq(lit(max.clone())))
                }
            }
        };

        Ok(expr)
    }

    /// Smart comparison that attempts to parse the value as the appropriate type
    fn smart_compare<F>(&self, c: Expr, val: &str, compare_fn: F) -> Result<Expr>
    where
        F: FnOnce(Expr, Expr) -> Expr,
    {
        // Try parsing as different types
        if let Ok(v) = val.parse::<i64>() {
            return Ok(compare_fn(c, lit(v)));
        }
        if let Ok(v) = val.parse::<f64>() {
            return Ok(compare_fn(c, lit(v)));
        }
        if val.eq_ignore_ascii_case("true") {
            return Ok(compare_fn(c, lit(true)));
        }
        if val.eq_ignore_ascii_case("false") {
            return Ok(compare_fn(c, lit(false)));
        }

        // Fall back to string comparison
        Ok(compare_fn(c, lit(val.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_filter() {
        let filter = FilterSpec::parse("age:gt:30").unwrap();
        assert_eq!(filter.column, "age");
        assert!(matches!(filter.operator, FilterOperator::GreaterThan));
        assert_eq!(filter.value, Some("30".to_string()));
    }

    #[test]
    fn test_parse_in_filter() {
        let filter = FilterSpec::parse("status:in:active,pending,done").unwrap();
        assert_eq!(filter.column, "status");
        if let FilterOperator::In(values) = filter.operator {
            assert_eq!(values, vec!["active", "pending", "done"]);
        } else {
            panic!("Expected In operator");
        }
    }

    #[test]
    fn test_parse_between_filter() {
        let filter = FilterSpec::parse("score:between:10,100").unwrap();
        assert_eq!(filter.column, "score");
        if let FilterOperator::Between(min, max) = filter.operator {
            assert_eq!(min, "10");
            assert_eq!(max, "100");
        } else {
            panic!("Expected Between operator");
        }
    }

    #[test]
    fn test_parse_null_filter() {
        let filter = FilterSpec::parse("deleted_at:isnull").unwrap();
        assert_eq!(filter.column, "deleted_at");
        assert!(matches!(filter.operator, FilterOperator::IsNull));
    }
}
