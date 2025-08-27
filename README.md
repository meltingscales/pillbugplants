# pillbugplants

A terminal-based ecosystem simulation featuring plants and pillbugs with realistic physics, life cycles, and interactions. Watch as plants grow from seeds, pillbugs move and feed, and a complete nutrient cycle maintains the ecosystem balance.

## Features

- **Multi-segment creatures**: Pillbugs with head, body, and legs that move as connected units
- **Size variations**: Small, medium, and large organisms with different growth rates, lifespans, and behaviors
- **Advanced physics**: Gravity, structural support, water flow, and realistic falling mechanics
- **Day/night cycles**: Affects plant photosynthesis and rain probability
- **Weather system**: Rain spawns water that flows and affects the environment
- **Complete ecosystem**: Closed nutrient loop where death feeds new life
- **Interactive terminal UI**: Real-time visualization with color-coded organisms

## Building

Build the project using Rust's Cargo:

```bash
cargo build
```

Or use the Makefile:

```bash
make build
```

## Running

### Interactive Mode

Run the interactive terminal simulation:

```bash
cargo run
```

Or:

```bash
make run
```

**Controls:**
- `q` - Quit the simulation
- `t` - Toggle taxonomy panel showing organism types

### Simulation Mode

Run headless simulations for testing:

```bash
# Quick 100-tick simulation
make sim-short

# Standard 500-tick simulation saved to file
make sim-test

# Long 1000-tick simulation saved to file
make sim-long
```

Or run directly with custom parameters:

```bash
cargo run -- --sim-ticks=1000 --output-file=my_simulation.txt
```

## Ecosystem Organisms

### Plants (with size variations)
- **Stems** (`i|║`): Structural support, consume nutrients
- **Leaves** (`lLŁ`): Photosynthesize during day, produce nutrients
- **Buds** (`°oO`): Growth points that develop into stems, leaves, or flowers
- **Flowers** (`·*✱`): Reproduce by spreading seeds, larger flowers spread farther
- **Withered** (`x`): Decaying plant matter that becomes nutrients

### Pillbugs (multi-segment with sizes)
- **Head** (`ó@●`): Eats plants, coordinates movement, can reproduce
- **Body** (`oO●`): Main body segment
- **Legs** (`vwW`): Locomotion segment
- **Decaying** (`░`): Decomposing pillbug parts that become nutrients

### Environment
- **Dirt** (`#`): Solid ground for plant growth
- **Sand** (`.`): Falls with gravity
- **Water** (`~`): Flows and falls, spawned by rain
- **Nutrients** (`+`): Essential for plant growth, diffuses through environment

## Size System

All organisms have size variants affecting their behavior:

- **Small**: Faster growth/movement, shorter lifespan, weaker
- **Medium**: Balanced stats
- **Large**: Slower growth/movement, longer lifespan, stronger

Size inheritance occurs with slight variation during reproduction.

## Development

```bash
# Code formatting
make fmt

# Lint checking
make clippy

# Run tests
make test

# Full quality check
make full-check
```

## Inspiration

Inspired by [repomirror documentation](https://github.com/repomirrorhq/repomirror/blob/main/repomirror.md) and watching pillbugs on the playground.

TODO: Ask on Aider discord what the right way is to use aider automatically. send them this repo.
