# pqfilter

A fast, ergonomic CLI tool for filtering and transforming Parquet files, built with Rust and Polars.

## Features

- **Filter**: Row filtering with rich operators (eq, ne, gt, lt, contains, in, between, regex, etc.)
- **Select**: Column selection or exclusion
- **Transform**: Column transformations (uppercase, round, cast, rename, fill_null, clip, etc.)
- **Sort**: Multi-column sorting with ascending/descending control
- **Aggregate**: GroupBy operations with common aggregations (sum, mean, min, max, count, etc.)
- **Sample**: Random sampling by count or fraction
- **Info**: Schema inspection and statistics
- **Pipeline**: Multi-step processing via JSON configuration

## Installation

```bash
# Build from source
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

## Quick Start

```bash
# View file info
pqfilter info -i data.parquet --stats --head 10

# Filter rows
pqfilter filter -i data.parquet -o filtered.parquet -f "age:gt:30" -f "status:eq:active"

# Select columns
pqfilter select -i data.parquet -o subset.parquet -c id,name,email

# Transform columns
pqfilter transform -i data.parquet -o transformed.parquet -t "name:uppercase" -t "price:round:2"

# Sort data
pqfilter sort -i data.parquet -o sorted.parquet -b "-created_at,name"

# Aggregate
pqfilter aggregate -i data.parquet -o agg.parquet -g department -a "salary:mean" -a "id:count"
```

## Commands

### filter

Filter rows based on column conditions.

```bash
pqfilter filter -i input.parquet -o output.parquet -f "column:operator:value"
```

**Operators:**
| Operator | Aliases | Example | Description |
|----------|---------|---------|-------------|
| `eq` | `=`, `==` | `status:eq:active` | Equal to |
| `ne` | `!=`, `<>` | `type:ne:deleted` | Not equal |
| `gt` | `>` | `age:gt:18` | Greater than |
| `ge` | `>=` | `score:ge:90` | Greater or equal |
| `lt` | `<` | `price:lt:100` | Less than |
| `le` | `<=` | `count:le:10` | Less or equal |
| `contains` | `like` | `name:contains:John` | String contains |
| `startswith` | `starts` | `email:startswith:admin` | String starts with |
| `endswith` | `ends` | `file:endswith:.pdf` | String ends with |
| `isnull` | `null` | `deleted_at:isnull` | Is null |
| `notnull` | `!null` | `email:notnull` | Is not null |
| `in` | - | `status:in:a,b,c` | Value in list |
| `between` | - | `age:between:18,65` | Value in range |
| `regex` | `~` | `code:regex:^[A-Z]{3}` | Regex match |

**Combine multiple filters:**
```bash
# AND (default)
pqfilter filter -i data.parquet -o out.parquet -f "age:gt:30" -f "status:eq:active"

# OR
pqfilter filter -i data.parquet -o out.parquet -f "status:eq:active" -f "status:eq:pending" --combine or
```

### select

Select or exclude specific columns.

```bash
# Select specific columns
pqfilter select -i input.parquet -o output.parquet -c id,name,email

# Exclude columns (keep all others)
pqfilter select -i input.parquet -o output.parquet -c internal_id,debug_info --exclude
```

### transform

Transform column values.

```bash
pqfilter transform -i input.parquet -o output.parquet -t "column:operation[:args]"
```

**Operations:**
| Operation | Args | Example | Description |
|-----------|------|---------|-------------|
| `uppercase` | - | `name:uppercase` | To uppercase |
| `lowercase` | - | `name:lowercase` | To lowercase |
| `trim` | - | `text:trim` | Trim whitespace |
| `round` | decimals | `price:round:2` | Round to decimals |
| `abs` | - | `delta:abs` | Absolute value |
| `cast` | dtype | `id:cast:string` | Cast type |
| `rename` | new_name | `col:rename:new_col` | Rename column |
| `fill_null` | value | `score:fill_null:0` | Fill nulls |
| `replace` | pat,repl | `text:replace:old,new` | Replace substring |
| `substring` | start[,len] | `code:substring:0,3` | Extract substring |
| `add` | value | `price:add:10` | Add constant |
| `multiply` | value | `qty:multiply:1.1` | Multiply by constant |
| `log` | - | `value:log` | Log base 10 |
| `ln` | - | `value:ln` | Natural log |
| `sqrt` | - | `value:sqrt` | Square root |
| `clip` | min,max | `score:clip:0,100` | Clamp to range |
| `to_date` | [format] | `date_str:to_date:%Y-%m-%d` | Parse to date |
| `year` | - | `created:year` | Extract year |
| `month` | - | `created:month` | Extract month |
| `day` | - | `created:day` | Extract day |

**Cast types:** `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float32`, `float64`, `bool`, `string`, `date`

### sort

Sort by one or more columns.

```bash
# Ascending
pqfilter sort -i input.parquet -o output.parquet -b name,age

# Descending (prefix with -)
pqfilter sort -i input.parquet -o output.parquet -b "-score,name"

# Nulls first
pqfilter sort -i input.parquet -o output.parquet -b created_at --nulls-first
```

### aggregate

Group and aggregate data.

```bash
pqfilter aggregate -i input.parquet -o output.parquet -g column1,column2 -a "column:operation"
```

**Aggregations:** `sum`, `mean`/`avg`, `min`, `max`, `count`, `first`, `last`, `std`, `var`, `median`

```bash
# Sales by region and product
pqfilter aggregate -i sales.parquet -o summary.parquet \
    -g region,product \
    -a "amount:sum" -a "amount:mean" -a "order_id:count"
```

### info

Display schema and statistics.

```bash
# Basic info
pqfilter info -i data.parquet

# With statistics
pqfilter info -i data.parquet --stats

# Preview rows
pqfilter info -i data.parquet --head 20
```

### sample

Random sampling.

```bash
# Sample N rows
pqfilter sample -i input.parquet -o sample.parquet -n 1000

# Sample fraction
pqfilter sample -i input.parquet -o sample.parquet --fraction 0.1

# Reproducible sampling
pqfilter sample -i input.parquet -o sample.parquet -n 1000 --seed 42
```

### pipeline

Run multiple operations from a JSON config.

```bash
pqfilter pipeline -i input.parquet -o output.parquet -c pipeline.json
```

**Example pipeline.json:**
```json
{
  "steps": [
    {
      "type": "filter",
      "filters": ["status:eq:active", "age:ge:18"],
      "combine": "and"
    },
    {
      "type": "transform",
      "transforms": ["name:uppercase", "score:round:2"]
    },
    {
      "type": "select",
      "columns": ["id", "name", "score", "department"]
    },
    {
      "type": "sort",
      "by": ["-score", "name"]
    },
    {
      "type": "sample",
      "n": 100,
      "seed": 42
    }
  ]
}
```

## Output Formats

The output format is determined by file extension:

- `.parquet` - Apache Parquet (default, recommended)
- `.csv` - Comma-separated values
- `.json` - JSON (newline-delimited)

```bash
pqfilter filter -i data.parquet -o filtered.csv -f "active:eq:true"
pqfilter aggregate -i data.parquet -o summary.json -g dept -a "salary:mean"
```

## Examples

### E-commerce Analytics

```bash
# High-value orders from California
pqfilter filter -i orders.parquet -o ca_high_value.parquet \
    -f "state:eq:CA" -f "total:gt:500"

# Daily revenue by category
pqfilter aggregate -i orders.parquet -o daily_revenue.parquet \
    -g order_date,category \
    -a "total:sum" -a "order_id:count"
```

### Log Analysis

```bash
# Error logs from last week
pqfilter filter -i logs.parquet -o errors.parquet \
    -f "level:eq:ERROR" -f "timestamp:gt:2024-01-01"

# Errors by service
pqfilter aggregate -i errors.parquet -o error_counts.parquet \
    -g service,error_code \
    -a "id:count"
```

### Data Cleaning

```bash
pqfilter transform -i raw.parquet -o cleaned.parquet \
    -t "email:lowercase" \
    -t "name:trim" \
    -t "phone:replace:-," \
    -t "age:clip:0,120" \
    -t "missing_col:fill_null:unknown"
```

## Performance

Built on [Polars](https://pola.rs/), pqfilter leverages:
- Lazy evaluation for query optimization
- Parallel processing
- Memory-efficient streaming
- Predicate pushdown for Parquet files

For large datasets, the release build (`cargo build --release`) provides significantly better performance.

## License

MIT
