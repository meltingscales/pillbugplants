# Pillbugplants Design Document

## Core Concept
A falling-sand style physics sandbox simulating a miniature ecosystem inspired by playground observations of pillbugs and plant life. The simulation runs deterministically with ASCII art rendering via ratatui.

## Architecture

### World System
- **Deterministic**: Every tick updates physics in the same order
- **Grid-based**: Fixed-size 2D grid where each cell contains a TileType
- **Physics**: Gravity affects sand, water; entities have support checking
- **Day/Night Cycle**: Affects plant photosynthesis and rain probability

### Entity Types

#### Plants (Multi-size organisms)
- **PlantStem**: Main structural support, ages and can extend upward
- **PlantLeaf**: Photosynthesis organs, shorter lifespan
- **PlantBud**: Growth points that mature into branches or flowers
- **PlantBranch**: Diagonal growth creating Y-shaped branching patterns
- **PlantFlower**: Reproductive organs that spread seeds
- **PlantWithered**: Dying plant matter that decomposes into nutrients

Size variants (Small/Medium/Large):
- Small: Fast growth, short life, efficient reproduction
- Medium: Balanced stats
- Large: Slow growth, long life, wide seed dispersal

#### Pillbugs (Multi-segment creatures)
- **PillbugHead**: Controls movement and feeding behavior
- **PillbugBody**: Main body segment, grows as pillbug matures
- **PillbugLegs**: Locomotive segment, affects movement speed
- **PillbugDecaying**: Dying pillbug parts that become nutrients

Size-based feeding efficiency:
- Large pillbugs handle large food better
- Small pillbugs are more efficient with small food
- Mismatched sizes reduce feeding efficiency

#### Environmental Elements
- **Dirt**: Solid ground providing support
- **Sand**: Falls and forms natural piles
- **Water**: Flows downward and spreads sideways
- **Nutrient**: Diffuses slowly, consumed by plants
- **Empty**: Air space

### Movement System

#### MovementStrategy Enum
- **SeekFood**: Direct movement toward detected food
- **Social**: Movement toward other pillbugs of same size  
- **Avoid**: Movement away from dangers (not yet implemented)
- **Explore**: Random movement for discovery
- **Rest**: Minimal movement, energy conservation

Movement probability varies by strategy and pillbug characteristics.

### Life Cycle Systems

#### Plant Growth
1. **Stem Extension**: Vertical growth creating plant structure
2. **Lateral Growth**: Leaves for photosynthesis, buds for reproduction
3. **Branching**: Buds mature into diagonal branches (Y-shapes)
4. **Reproduction**: Flowers spread seeds with distance based on size
5. **Death**: Aging leads to withering and nutrient release

#### Pillbug Behavior
1. **Maturation**: Body segments grow as pillbugs age (head → body → legs)
2. **Feeding**: Size-based efficiency eating plants and nutrients
3. **Movement**: AI-driven behavior using movement strategies
4. **Reproduction**: Well-fed mature pillbugs spawn offspring
5. **Death**: Old age leads to decay and nutrient release

### Physics Engine
- **Support Checking**: Plants need structural support or they fall/wither
- **Falling Physics**: Sand/water obey gravity with diagonal sliding
- **Flow Dynamics**: Water spreads horizontally when blocked
- **Nutrient Diffusion**: Slow random walk spreading nutrients

### Ecosystem Dynamics
- **Closed Loop**: Dead matter → nutrients → plant growth → pillbug food → death
- **Population Balance**: Automatic spawning maintains minimum populations
- **Environmental Cycles**: Day/night affects growth and rain patterns
- **Size Inheritance**: Offspring inherit size with variation chance

## Rendering
ASCII characters with color coding:
- Size variants use different character sets
- Colors fade with age for visual aging
- Terminal rendering via ratatui with real-time updates

## Simulation Modes
- **Interactive**: Full ratatui interface with user controls
- **Headless**: Command-line simulation for testing/automation
- **Batch**: Run N ticks and output final state

This creates an emergent ecosystem where simple rules lead to complex behaviors and natural population dynamics.
