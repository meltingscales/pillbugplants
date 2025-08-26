# Pillbugplants Design Notes

## Current Implementation Status (2025-08-26)

### ✅ Implemented Core Features
- Basic ratatui terminal UI with dynamic world sizing
- Physics simulation: gravity, falling sand/water, support checking
- Tile-based world with dirt, sand, water, nutrients, plants, pillbugs
- Day/night cycle affecting plant reproduction and rain
- Life systems:
  - Plants age (0-200), consume nutrients, reproduce during day, decompose
  - Pillbugs age (0-180), eat plants, move randomly, reproduce when fed
  - Nutrient diffusion and cycling
- Rain system during night cycles
- Taxonomy panel (toggle with 't' key)
- 8-way support checking for living entities

### 🚧 Missing Priority Features
- **Multi-segment body systems**: Plants and bugs should have body parts/segments
- **Complex plant structures**: Buds, branches, flowers that can wither and fall off
- **More sophisticated plant growth**: Branching patterns, different plant types
- **Enhanced bug behavior**: More intelligent movement, group behavior
- **Size variations**: Different sized plants and bugs
- **More physics materials**: Maybe stone, organic matter types
- **Seasonal cycles**: Beyond just day/night
- **Better decomposition**: Gradual decay rather than instant nutrient conversion

### 🎯 Next Development Priorities
1. Implement multi-segment plant bodies (stems, buds, leaves)
2. Add plant branching and bud development system
3. Create multi-segment pillbug bodies
4. Add size variations for organisms
5. Improve decomposition system with gradual decay

## Technical Architecture

### Core Types
- `TileType` enum for all world materials
- `World` struct managing simulation state
- `App` struct for UI and user interaction
- Deterministic physics updates each tick

### Simulation Loop
1. Rain spawning (night cycles)
2. Physics updates (gravity, falling, support)
3. Nutrient diffusion
4. Life updates (aging, reproduction, movement)

### UI Features
- Terminal-based with ratatui
- Dynamic world sizing based on terminal
- Color-coded tiles with aging effects
- Info panel with tick count, day/night, rain status
- Toggle-able taxonomy reference panel