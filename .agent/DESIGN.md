# Design Notes - Pillbug Plants

## Core Architecture
- Tick-based simulation with deterministic updates
- Dynamic ASCII tile grid rendered with ratatui (adapts to terminal size)
- Each tile can hold one entity type at a time
- All updates happen in parallel using tile cloning to avoid conflicts

## Entity Types
- **Empty**: Nothing
- **Dirt**: Static ground material  
- **Sand**: Falls due to gravity
- **Water**: Falls and spreads horizontally
- **Plant(age)**: Ages from 0-200, reproduces with nutrients during day
- **Pillbug(age)**: Ages from 0-180, eats plants, reproduces when fed
- **Nutrient**: Diffuses randomly, consumed by plants

## Physics System
- Gravity affects sand, water, plants, and pillbugs
- Water spreads horizontally when blocked
- Plants/pillbugs fall unless supported by any of 8 adjacent solid tiles
- Water displacement when entities fall into it
- All physics updates happen bottom-to-top to prevent double-processing

## Life Systems
### Day/Night Cycle
- Sine wave cycle affects plant reproduction and rain probability
- Plants only reproduce during daylight hours
- Rain more likely during nighttime

### Aging & Death
- Plants age +1/tick, die at 200, decompose to nutrients
- Pillbugs age +1/tick, die at 180, decompose to nutrients  
- Nutrients slow aging: plants -5, pillbugs -10
- Starvation accelerates aging: plants +1, pillbugs +2

### Reproduction
- Plants: Need nutrients + daylight, spread to adjacent empty tiles
- Pillbugs: Need to eat plants, 50% chance to leave baby when moving

### Ecosystem Balance
- Closed nutrient loop: death → nutrients → plant growth → pillbug food → death
- Rain adds water periodically to maintain moisture levels

## Visual System
- Age-based color fading (older entities get darker)
- Rain intensity displayed in status bar
- Dynamic world sizing based on terminal dimensions
- Real-time tick counter and day/night indicator

## Technical Details
- Uses crossterm for terminal control
- Ratatui for UI rendering  
- Rand crate for all randomness
- Vec<Vec<TileType>> for dynamic world sizing
- Entity ages stored as u8 in enum variants