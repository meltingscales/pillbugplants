# Pillbugplants TODO

## Current Status
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
- ✅ All utility methods (`is_plant`, `is_pillbug`, `get_size`) fully utilized

## Next Priority Features
- [✅] Implement plant disease/infection spread system
- [✅] Add biome variations with distinct wet/dry microenvironments
- [✅] Create more sophisticated water flow and pooling mechanics
- [✅] Add wind effects on seed dispersal and small particles

## Engine Enhancements
- [ ] Optimize physics calculations for larger worlds
- [ ] Add more sophisticated collision detection
- [ ] Implement water flow patterns and pooling
- [ ] Add wind effects on seeds and small particles
- [ ] Create biome variations (wet/dry areas)

## Simulation Features
- [ ] Add more plant species with different growth patterns
- [ ] Implement symbiotic relationships between species
- [ ] Add decomposer organisms (bacteria, fungi)
- [ ] Create food web dynamics
- [ ] Add territorial behavior for pillbugs

## UI/UX Improvements
- [ ] Add pause/resume functionality
- [ ] Implement speed controls
- [ ] Create detailed entity inspection mode
- [ ] Add simulation statistics panel
- [ ] Implement save/load functionality

## Testing & Quality
- [ ] Add unit tests for core simulation logic
- [ ] Create benchmark tests for performance
- [ ] Add integration tests for complex scenarios
- [ ] Implement automated regression testing

## Documentation
- [ ] Create comprehensive API documentation
- [ ] Write simulation behavior guide
- [ ] Add contributing guidelines
- [ ] Create performance optimization guide

## Recent Achievements
- 🎉 All compiler warnings resolved - ecosystem now uses all intended features
- 🎉 Seasonal ecosystem dynamics create realistic environmental pressures
- 🎉 Advanced AI behaviors with survival instincts and resource competition
- 🎉 Underground root networks actively reshape nutrient distribution
- 🎉 Plant disease/infection system with seasonal spread patterns and ecosystem pressure
- 🎉 **Comprehensive biome variation system** with four distinct microenvironments:
  * **Wetland**: High moisture retention, lush plant growth, water pooling
  * **Grassland**: Balanced conditions, moderate growth and moisture
  * **Drylands**: Low moisture, sparse vegetation, quick evaporation, sandy soil
  * **Woodland**: Dense plant growth, rich nutrients, extensive root systems
- 🎉 Biome-influenced terrain generation, water physics, and plant ecology
- 🎉 Regional biome maps with natural boundaries create realistic ecological gradients
- 🎉 **Sophisticated water flow and pooling mechanics** with depth-based physics:
  * **Depth visualization**: Droplets (·), normal (~), deep (≈), very deep (█) water
  * **Pressure-driven flow**: Deeper water flows more readily, creates realistic streams
  * **Biome-specific pooling**: Wetlands retain water, drylands drain quickly
  * **Momentum physics**: Falling water gains depth, water seeks equilibrium levels
  * **Enhanced evaporation**: Depth, biome, temperature, and day/night affect evaporation rates
- 🎉 **Comprehensive wind effects system** with realistic particle dispersal:
  * **Dynamic wind patterns**: Seasonal direction changes (Spring easterly, Fall westerly)
  * **Seed dispersal mechanics**: Flowers produce seeds that travel with wind currents
  * **Spore transmission**: Diseased plants generate airborne spores for infection spread
  * **Size-based wind susceptibility**: Small seeds highly mobile, large seeds more stable
  * **Environmental feedback**: Wind strength affects seed/spore generation rates
  * **Particle physics**: Wind-particle collisions can displace light water droplets
  * **Lifecycle systems**: Seeds age, germinate, or decay; spores spread disease then fade
