#!/usr/bin/env bash

export AWS_LAMBDA_RUNTIME_API=""  # prevents lambda_runtime from waiting
cargo run --release -- <<EOF
{
  "payload": { "name": "Alice" },
  "context": { "request_id": "local-123" }
}
EOF