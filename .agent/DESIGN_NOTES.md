# Pillbug Plants - Design Architecture

## Core Philosophy
The simulation embodies a **deterministic physics sandbox** where simple rules create complex emergent behaviors. Inspired by watching pillbugs in playgrounds, it focuses on the interplay between plants that grow and die, and pillbugs that wander, reproduce, and decay.

## System Architecture

### Tile-Based World Model
```
World {
    tiles: Vec<Vec<TileType>>,      // Main simulation grid
    biome_map: Vec<Vec<Biome>>,     // Regional environmental data
    environmental_state: {...},     // Weather, seasons, time
}
```

Each tile represents one unit of space and can contain exactly one entity type. The system uses **cellular automata** principles for deterministic updates.

### Entity Lifecycle Management
All entities follow consistent aging patterns:
- **Plants**: Stems (0-255 age), Leaves (0-150), Buds (0-50), Flowers (0-100)
- **Pillbugs**: All segments (0-180 age), then decay (0-20 before nutrient)
- **Environmental**: Seeds (0-100), Spores (0-50), Water depth (0-255)

### Size Variation System
Three size categories affect all organisms:
- **Small**: 30% faster growth, 30% shorter life, modified display characters
- **Medium**: Baseline values, normal display
- **Large**: 20% slower growth, 40% longer life, enhanced display characters

## Physics Systems

### Deterministic Updates
Each tick processes systems in fixed order:
1. **Rain generation** → Environmental water addition
2. **Physics update** → Gravity, water flow, settling
3. **Wind effects** → Seed/spore dispersal, particle movement  
4. **Plant support** → Structural integrity checks
5. **Nutrient diffusion** → Chemical spread simulation
6. **Life updates** → Growth, aging, reproduction, death
7. **Entity spawning** → New life generation

### Water Mechanics
- **Depth-based physics**: Higher depth = faster flow
- **Momentum system**: Falling water accumulates depth
- **Biome interaction**: Retention rates vary by environment
- **Multi-level visualization**: 4 depth ranges with distinct characters

### Wind System
- **Seasonal patterns**: Direction changes with seasons (Spring easterly → Fall westerly)
- **Particle interaction**: Seeds/spores/light particles affected by wind vectors
- **Collision physics**: Wind can displace small water droplets
- **Environmental feedback**: Plant reproduction responds to dispersal effectiveness

## Biological Systems

### Plant Architecture
**Multi-segment design** with specialized functions:
- **Roots**: Underground nutrient absorption network
- **Stems**: Primary structural support, vertical growth
- **Leaves**: Photosynthesis organs, light-dependent growth
- **Buds**: Growth points that become branches or flowers
- **Branches**: Diagonal expansion, Y-shaped growth patterns
- **Flowers**: Reproduction organs, seed production

**Growth mechanics**:
- Light dependence (day/night cycle affects leaf growth)
- Seasonal modifiers (temperature/humidity impact)
- Biome influence (woodland promotes dense growth)
- Support requirements (stems must support leaf/flower load)

### Animal Behavior System
**Multi-segment pillbugs** with intelligent decision-making:

**Movement Strategies**:
```rust
MovementStrategy {
    SeekFood(direction),    // Nutrient-seeking behavior
    Social(direction),      // Attraction to other pillbugs  
    Avoid(direction),       // Escape from threats
    Explore,               // Random movement
    Rest,                  // Minimal activity
}
```

**Threat Detection**: Larger pillbugs, deep water, unstable terrain

### Ecosystem Interactions

**Disease System**:
- Plants can become diseased, spreading to neighbors
- Diseased plants produce airborne spores
- Environmental conditions affect spread rates
- Creates population pressure and selection dynamics

**Reproduction Mechanics**:
- Plants: Flowers produce seeds dispersed by wind
- Pillbugs: Social behavior leads to reproduction when conditions allow
- Size inheritance: Offspring reflect parent characteristics
- Resource competition affects success rates

## Environmental Systems

### Biome Variation
Four distinct biome types create **microenvironmental diversity**:

**Wetland**: High moisture retention (1.4x), excellent plant growth (1.3x), frequent pooling
**Grassland**: Balanced conditions (1.0x), moderate growth, baseline environment  
**Drylands**: Low moisture (0.6x), sparse vegetation (0.7x), quick water loss, sandy terrain
**Woodland**: Dense growth (1.5x), rich nutrients (1.4x), high moisture retention (1.2x)

### Seasonal Dynamics
**Four-season cycle** (1600 ticks per season):
- **Spring**: Growth season, high humidity (0.7), mild temperature (0.3), easterly winds
- **Summer**: Hot season, low humidity (0.3), high temperature (0.8), southerly winds  
- **Fall**: Moderate temperature (0.1), increasing humidity (0.6), westerly storm winds
- **Winter**: Cold season (-0.5), variable humidity (0.4), northerly winds

**Weather Systems**:
- Rain probability tied to humidity and season
- Temperature affects all growth rates
- Wind patterns influence seed dispersal success
- Day/night cycles modulate photosynthesis

## Design Principles

### Emergent Complexity
Simple rules create sophisticated behaviors:
- **Local interactions** → Global ecosystem patterns
- **Individual decisions** → Population dynamics  
- **Physical constraints** → Evolutionary pressure
- **Environmental cycles** → Adaptive strategies

### Performance Architecture
- **Single-threaded determinism**: Reproducible simulation results
- **Efficient tile operations**: Direct array access patterns
- **Bounded aging**: Prevents infinite accumulation
- **Lazy evaluation**: Only process active regions

### Visual Design
**ASCII-based representation** optimized for terminal display:
- **Character hierarchy**: Size variations modify base symbols
- **Color coding**: RGB values reflect age, health, environmental state
- **Information density**: Each character conveys multiple data points
- **Real-time feedback**: Immediate visual response to simulation changes

## Current Implementation Status

The simulation implements a **complete ecosystem model** with:
- ✅ **Comprehensive physics**: Water, wind, gravity, particle dynamics
- ✅ **Rich biology**: Multi-segment organisms, lifecycle management, reproduction
- ✅ **Environmental systems**: Seasons, weather, biomes, day/night cycles
- ✅ **Disease ecology**: Infection spread, environmental pressures
- ✅ **Behavioral AI**: Movement strategies, threat avoidance, social interaction
- ✅ **Visual polish**: Size-aware rendering, color-coded health states

The architecture successfully demonstrates **emergent ecological dynamics** from the interaction of deterministic rules, creating a compelling digital ecosystem that mirrors natural complexity.