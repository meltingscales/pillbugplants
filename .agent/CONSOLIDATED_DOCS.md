# Pillbugplants - Consolidated Documentation

## Project Overview

A falling-sand style physics sandbox simulating a miniature ecosystem inspired by playground observations of pillbugs and plant life. The simulation runs deterministically with ASCII art rendering via ratatui.

---

## Core Philosophy & Architecture

### Design Philosophy
The simulation embodies a **deterministic physics sandbox** where simple rules create complex emergent behaviors. The focus is on the interplay between plants that grow and die, and pillbugs that wander, reproduce, and decay.

### System Architecture
```
World {
    tiles: Vec<Vec<TileType>>,      // Main simulation grid
    biome_map: Vec<Vec<Biome>>,     // Regional environmental data
    environmental_state: {...},     // Weather, seasons, time
}
```

Each tile represents one unit of space and can contain exactly one entity type. The system uses **cellular automata** principles for deterministic updates.

---

## Entity Systems

### Entity Lifecycle Management
All entities follow consistent aging patterns:
- **Plants**: Stems (0-255 age), Leaves (0-150), Buds (0-50), Flowers (0-100)
- **Pillbugs**: All segments (0-180 age), then decay (0-20 before nutrient)
- **Environmental**: Seeds (0-100), Spores (0-50), Water depth (0-255)

### Size Variation System
Three size categories affect all organisms:
- **Small**: 30% faster growth, 30% shorter life (5.6x lifespan multiplier), modified display characters
- **Medium**: Baseline values (8x lifespan multiplier), normal display
- **Large**: 20% slower growth, 40% longer life (11.2x lifespan multiplier), enhanced display characters

**Note**: Lifespans have been extended 8x from original values to allow for longer ecosystem observation.

### Plant Architecture
**Multi-segment design** with specialized functions:
- **PlantStem** (|): Primary structural support, vertical growth
- **PlantLeaf** (L): Photosynthesis organs, light-dependent growth
- **PlantBud** (o): Growth points that become branches or flowers
- **PlantBranch** (/): Diagonal expansion, Y-shaped growth patterns
- **PlantFlower** (*): Reproduction organs, seed production
- **PlantRoot** (r): Underground nutrient absorption network
- **PlantWithered** (x): Dying plant matter
- **PlantDiseased** (?): Infected plants that spread disease

### Animal Behavior System
**Multi-segment pillbugs** with intelligent decision-making:
- **PillbugHead** (@): Controls movement and feeding behavior
- **PillbugBody** (O): Main body segment
- **PillbugLegs** (w): Locomotive segment
- **PillbugDecaying** (░): Dying pillbug parts

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

---

## Physics Systems

### Deterministic Updates
Each tick processes systems in fixed order:
1. **Rain generation** → Environmental water addition
2. **Physics update** → Gravity, water flow, settling
3. **Gravity application** → Unsupported entity falling (NEW!)
4. **Wind effects** → Seed/spore dispersal, particle movement  
5. **Plant support** → Structural integrity checks
6. **Nutrient diffusion** → Chemical spread simulation
7. **Life updates** → Growth, aging, reproduction, death
8. **Entity spawning** → New life generation

### Water Mechanics (Enhanced with Absorption)
- **Depth-based physics**: Higher depth = faster flow
- **Momentum system**: Falling water accumulates depth
- **Biome interaction**: Retention rates vary by environment
- **Water wetting earth**: Water can soak into dirt/sand instead of just piling up (NEW!)
- **Multi-level visualization**: 4 depth ranges with distinct characters

### Gravity System (NEW!)
- **Pillbug gravity**: Unsupported pillbug segments fall if no solid support
- **Connected segment logic**: Pillbugs with ground contact support connected segments
- **Loose particle physics**: Seeds, spores, and nutrients fall at different rates
- **Support detection**: Checks for dirt, sand, plants, and connected pillbug parts

### Wind System
- **Seasonal patterns**: Direction changes with seasons (Spring easterly → Fall westerly)
- **Particle interaction**: Seeds/spores/light particles affected by wind vectors
- **Collision physics**: Wind can displace small water droplets
- **Environmental feedback**: Plant reproduction responds to dispersal effectiveness

---

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

---

## Implementation Status

## ✅ COMPLETED FEATURES

### Core Systems
- ✅ Basic simulation engine with physics sandbox
- ✅ Multi-size entities (plants and pillbugs with small/medium/large variants)
- ✅ Plant growth system with stems, leaves, buds, branches, flowers
- ✅ Y-shaped plant branching system
- ✅ Pillbug multi-segment bodies (head-body-legs)
- ✅ Size-based feeding efficiency for pillbugs
- ✅ Intelligent movement with MovementStrategy enum (including Avoid strategy)
- ✅ Comprehensive danger detection (larger pillbugs, water, unstable ground)
- ✅ Advanced plant root system with active nutrient absorption
- ✅ Seasonal weather system (Spring/Summer/Fall/Winter cycles)
- ✅ Temperature and humidity affecting all growth rates
- ✅ Day/night cycle with seasonal rain patterns
- ✅ Nutrient diffusion system
- ✅ Reproduction systems for both plants and pillbugs

### Recent Major Updates
- ✅ **Water wetting earth mechanic**: Water can soak into dirt/sand instead of piling up
- ✅ **Gravity system for unsupported entities**: Pillbugs and loose objects fall when unsupported
- ✅ Plant disease/infection spread system
- ✅ Biome variations with distinct wet/dry microenvironments
- ✅ Sophisticated water flow and pooling mechanics
- ✅ Wind effects on seed dispersal and small particles
- ✅ Optimize physics calculations (tile change queue system implemented)
- ✅ Ecosystem statistics system with real-time monitoring

### Engine Performance
- ✅ **Memory efficiency**: Eliminated full world array clones during physics updates
- ✅ **Reduced allocations**: Change queue system minimizes memory pressure for large worlds
- ✅ **Maintained determinism**: Identical simulation behavior with better performance
- ✅ **Scalable foundation**: Architecture supports efficient optimization of additional physics systems

---

## 🎯 TODO - Future Development

### Next Priority Features
- [ ] Add more plant species with different growth patterns
- [ ] Implement symbiotic relationships between species
- [ ] Add decomposer organisms (bacteria, fungi)
- [ ] Create food web dynamics
- [ ] Add territorial behavior for pillbugs

### Engine Enhancements
- [ ] Add more sophisticated collision detection  
- [ ] Create biome-specific plant species

### Simulation Features
- [ ] Implement pillbug lifecycle stages (egg, larva, adult)
- [ ] Add digging behavior (pillbugs can move dirt/sand)
- [ ] Add group behavior and pheromone trails

### UI/UX Improvements
- [ ] Add pause/resume functionality
- [ ] Implement speed controls
- [ ] Create detailed entity inspection mode
- [ ] Implement save/load functionality

### Testing & Quality
- [ ] Add unit tests for core simulation logic
- [ ] Create benchmark tests for performance
- [ ] Add integration tests for complex scenarios
- [ ] Implement automated regression testing

### Documentation
- [ ] Create comprehensive API documentation
- [ ] Write simulation behavior guide
- [ ] Add contributing guidelines
- [ ] Create performance optimization guide

---

## Recent Achievements 🎉

### Ecosystem Maturity Milestone
- **Multi-generational complexity**: Extensive plant communities spanning multiple life cycles
- **Advanced water physics**: Deep pools, flowing streams, droplet formations showing sophisticated fluid dynamics  
- **Diverse pillbug populations**: Multiple age groups and sizes coexisting with realistic lifespans
- **Seasonal ecosystem evolution**: Full winter-to-spring transitions with temperature-driven changes
- **Emergent ecosystem behaviors**: Self-organizing communities, resource competition, and natural selection

### Major System Implementations
- **Comprehensive biome variation system** with four distinct microenvironments
- **Sophisticated water flow and pooling mechanics** with depth-based physics
- **Comprehensive wind effects system** with realistic particle dispersal
- **Engine performance optimization** with tile change queue system
- **Ecosystem statistics system** with real-time monitoring and health metrics

### Latest Updates (Current Session)
- **🚀 MASSIVE PERFORMANCE OPTIMIZATION**: Gravity checks now skip roots buried in soil - major CPU savings
- **📊 Performance monitoring system**: Comprehensive lag detection and system profiling
- **🎛️ Toggleable performance display**: Press 'p' to show detailed performance metrics with TPS/frame times
- **⚡ System-level profiling**: Individual timing breakdown for physics, gravity, life updates, etc.
- **📁 MODULAR ARCHITECTURE**: Successfully broke world.rs (2400+ lines) into logical modules:
  - **life.rs** (93 lines) - Plant/pillbug lifecycle systems
  - **physics.rs** (61 lines) - Sand falling, particle gravity 
  - **environment.rs** (51 lines) - Weather, seasons, environmental conditions
  - **world.rs** (2383 lines) - Core world logic (reduced by ~200 lines)
- **🎯 Seed velocity system**: Flowers now shoot seeds with physics-based trajectories instead of dropping them nearby
- **🌪️ Projectile mechanics**: Seeds have velocity, bounce physics, and wind interactions during flight
- **💧 Water wetting earth**: Water can now soak into dirt/sand instead of just accumulating
- **⬇️ Gravity for unsupported entities**: Pillbugs and plants fall as connected groups when unsupported
- **🔗 Group-based falling**: Connected organisms maintain structural integrity during gravity events
- **⏱️ Extended lifespans**: All organisms now live 8x longer for better ecosystem observation
- **🐛 Race condition fix**: Pillbugs no longer grow extra segments while falling (stability check added)
- **🌱 Nutrient-rich dirt system**: New `NutrientDirt` tile type that stores absorbed nutrients
- **🔄 Enhanced nutrient absorption**: Plants absorbing nutrients delay their aging and death timers
- **🌿 Root-soil merging**: Plant roots can merge with dirt to create nutrient-rich soil
- **💰 Nutrient soil dynamics**: Free nutrients can absorb into dirt, creating nutrient reservoirs
- **🍽️ Advanced root feeding**: Roots can extract nutrients from nutrient-rich dirt over time
- **⚙️ Enhanced physics**: Better support detection for multi-segment organisms
- **🌍 Improved realism**: More natural water behavior, object physics, and nutrient cycling

---

## Technical Details

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

### Ecosystem Interactions
**Disease System**:
- Plants can become diseased, spreading to neighbors
- Diseased plants produce airborne spores
- Environmental conditions affect spread rates

**Reproduction Mechanics**:
- Plants: Flowers produce seeds dispersed by wind
- Pillbugs: Social behavior leads to reproduction when conditions allow
- Size inheritance: Offspring reflect parent characteristics

The architecture successfully demonstrates **emergent ecological dynamics** from the interaction of deterministic rules, creating a compelling digital ecosystem that mirrors natural complexity.