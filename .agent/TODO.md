# Pillbugplants TODO

## Current Status
- ✅ Basic simulation engine with physics sandbox
- ✅ Multi-size entities (plants and pillbugs with small/medium/large variants)
- ✅ Plant growth system with stems, leaves, buds, branches, flowers
- ✅ Y-shaped plant branching system
- ✅ Pillbug multi-segment bodies (head-body-legs)
- ✅ Size-based feeding efficiency for pillbugs
- ✅ Intelligent movement with MovementStrategy enum
- ✅ Day/night cycle with rain
- ✅ Nutrient diffusion system
- ✅ Reproduction systems for both plants and pillbugs

## Immediate Improvements
- [ ] Implement `Avoid` movement strategy for pillbugs (currently unused)
- [ ] Add predator or danger detection system
- [ ] Improve plant root system for better nutrient absorption
- [ ] Add seasonal changes affecting growth rates
- [ ] Implement plant diseases/infections that spread

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

## Current Warnings to Address
- [ ] `Avoid` variant in MovementStrategy is never constructed
- [ ] `is_plant`, `is_pillbug`, `get_size` methods are never used
