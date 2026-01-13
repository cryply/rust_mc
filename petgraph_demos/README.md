
# Rust Graph Algorithms with Jupyter

This repository contains Jupyter notebooks that demonstrate graph algorithms implemented in Rust. The notebooks use the `petgraph` crate for graph operations and visualization, allowing for interactive exploration of algorithms such as Kosaraju's Strongly Connected Components and Dijkstra's shortest path.

## Purpose

The goal is to provide a hands-on environment for learning and experimenting with graph algorithms in Rust. By using Jupyter notebooks, you can run code, visualize results, and document your findings all in one place.

## How to Run

- Install Rust and Cargo.
- Install the `evcxr_jupyter` kernel by running:

```
cargo install evcxr_jupyter
evcxr_jupyter --install
```

- Launch Jupyter Notebook:

```
jupyter notebook
```

- Open the notebooks and select the Rust (Evcxr) kernel.
- Use `:dep petgraph` in a cell to add the petgraph dependency.


## License
This repository is released under the CC0 1.0 Universal Public Domain Dedication.

To the extent possible under law, the author has waived all copyrights and related rights to the contents of this repository. You are free to copy, modify, distribute, and use the material for any purpose, including commercial uses, without asking permission or providing attribution.


## Why Evcxr?

The `evcxr` project makes it possible to run Rust code directly in Jupyter notebooks. This allows for rapid prototyping, interactive debugging, and easy sharing of code and results.

## Dependencies

- Rust toolchain
- Jupyter Notebook
- evcxr_jupyter kernel
- petgraph crate


## Usage

- Add dependencies using `:dep` in notebook cells.
- Run the notebooks to explore graph algorithms and visualize their outputs.

This setup enables fast experimentation and learning of graph algorithms in Rust, leveraging the power of interactive notebooks and modern Rust crates.
<span style="display:none">[^1][^2][^3][^4][^5][^6][^7][^8][^9]</span>

<div align="center">⁂</div>

[^1]: https://github.com/prof-merli/rust-crash-course-jupyter-notebooks

[^2]: https://gitlab.com/nicolalandro/rust-jupyter-notebook

[^3]: https://dev.to/iprosk/generics-in-rust-visualizing-bezier-curves-in-a-jupyter-notebook-part-3-565n

[^4]: https://depth-first.com/articles/2020/09/21/interactive-rust-in-a-repl-and-jupyter-notebook-with-evcxr/

[^5]: https://ratulmaharaj.com/posts/interactive-rust-with-jupyter-notebooks/

[^6]: https://www.40tude.fr/docs/06_programmation/rust/001_rust_jupyter/rust_jupyter.html

[^7]: https://www.reddit.com/r/IPython/comments/3hvzu4/render_jupyter_notebook_as_readme_file_for_github/

[^8]: https://news.ycombinator.com/item?id=34380914

[^9]: https://www.chevdor.com/post/2021/01/play-rust/

