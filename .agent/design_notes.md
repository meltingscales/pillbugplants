# Pillbugplants Design Notes

## Current Implementation Status (2025-08-26)

### ✅ Implemented Core Features
- Basic ratatui terminal UI with dynamic world sizing
- Physics simulation: gravity, falling sand/water, support checking
- Tile-based world with dirt, sand, water, nutrients, multi-segment plants, multi-segment pillbugs
- Day/night cycle affecting plant reproduction and rain
- **Multi-segment plant system**:
  - PlantStem (|): Structural support, consumes nutrients, grows buds
  - PlantLeaf (L): Photosynthesis, produces nutrients during day
  - PlantBud (o): Growth points that develop into stems/leaves/flowers
  - PlantFlower (*): Reproduction, spreads seeds during day
- **Multi-segment pillbug system** (NEW!):
  - PillbugHead (@): Eats plants, coordinates movement and reproduction
  - PillbugBody (O): Main body segment, ages with head
  - PillbugLegs (w): Locomotion segment, ages with body
- Life systems:
  - Plants have specialized parts with different lifespans and behaviors
  - Pillbugs now have 3-segment bodies, heads eat plants, bodies/legs follow
  - All segments age (0-180) and decompose into nutrients when they die
  - Nutrient diffusion and cycling
- Rain system during night cycles
- Taxonomy panel (toggle with 't' key) with detailed descriptions
- 8-way support checking with structural plant support

### 🚧 Missing Priority Features
- **Size variations**: Different sized plants and bugs  
- **Enhanced bug behavior**: More intelligent movement, group behavior, better segment coordination
- **More physics materials**: Maybe stone, organic matter types
- **Seasonal cycles**: Beyond just day/night
- **Better decomposition**: Gradual decay rather than instant nutrient conversion
- **Plant diseases/wilting**: Environmental stress effects

### 🎯 Next Development Priorities
1. ✅ ~~Implement multi-segment plant bodies (stems, buds, leaves, flowers)~~
2. ✅ ~~Add plant branching and bud development system~~
3. ✅ ~~Create multi-segment pillbug bodies (head, segments, legs)~~
4. Add size variations for organisms (small/medium/large)
5. Improve pillbug segment coordination and movement behavior
6. Improve decomposition system with gradual decay stages
7. Add more sophisticated pillbug AI and group behaviors

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