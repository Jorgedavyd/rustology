# Rustology

A single repository for Rust exercises, experiments, and learning notes.

## Contents

- [`projects/rustology`](projects/rustology): implementations and experiments
  covering memory management, smart pointers, and concurrency.
- [`rustlings`](rustlings): the official Rustlings exercises.

## Usage

Run the project checks from the repository root:

```sh
cargo test --manifest-path projects/rustology/Cargo.toml
```

Work through Rustlings:

```sh
cd rustlings
rustlings
```

Add future standalone Cargo projects under `projects/`. The repository root is
intentionally not a Cargo workspace because Rustlings manages its exercises as
an independent project.
