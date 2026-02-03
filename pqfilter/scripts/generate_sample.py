#!/usr/bin/env python3
"""Generate a sample Parquet file for testing pqfilter."""

import random
import sys

try:
    import polars as pl
except ImportError:
    print("Python polars not installed. Install with: pip install polars")
    sys.exit(1)

def main():
    random.seed(42)  # For reproducibility
    
    df = pl.DataFrame({
        'id': range(1, 101),
        'name': [f'User_{i}' for i in range(1, 101)],
        'age': [random.randint(18, 80) for _ in range(100)],
        'score': [round(random.uniform(0, 100), 2) for _ in range(100)],
        'status': [random.choice(['active', 'pending', 'inactive']) for _ in range(100)],
        'department': [random.choice(['Engineering', 'Sales', 'Marketing', 'HR']) for _ in range(100)],
    })
    
    df.write_parquet('sample.parquet')
    print(f'Created sample.parquet with {len(df)} rows')
    print(df.head(5))

if __name__ == '__main__':
    main()