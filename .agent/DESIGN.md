# Design Notes - Pillbug Plants

## Core Architecture
- Tick-based simulation with deterministic updates
- 80x40 ASCII tile grid rendered with ratatui
- Each tile can hold one entity type at a time

## Entity Types
- **Empty**: Nothing
- **Dirt**: Static ground material
- **Sand**: Falls due to gravity
- **Water**: Falls and spreads horizontally
- **Plant**: Grows during day, static otherwise
- **Pillbug**: Wanders randomly
- **Nutrient**: Future - will diffuse and be consumed

## Physics System
- Gravity affects sand and water
- Water spreads horizontally when blocked
- All physics updates happen bottom-to-top to prevent double-processing

## Life Systems
- Day/night cycle using sine wave
- Plants grow (spread to adjacent empty tiles) during day
- Pillbugs move randomly with 10% chance per tick
- Future: nutrient consumption, reproduction, death

## Rendering
- Each tile type has unique character and color
- Info panel shows tick count and day/night status
- Press 'q' to quit

## Technical Details
- Uses crossterm for terminal control
- Ratatui for UI rendering
- Rand crate for random behavior
- No save/load yet - worlds are procedurally generated