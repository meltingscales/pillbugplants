# Pillbugplants TODO

## Current Status
- Full ecosystem simulation working
- Multi-segment pillbugs with coordinated movement
- Complete plant lifecycle with buds and flowers
- Realistic physics for sand and water
- Day/night cycle with rain
- Nutrient cycling
- Size variations affecting behavior
- Pillbug reproduction and growth
- Plant support system

## Completed Features
- [x] Explore existing codebase structure
- [x] Check current build status
- [x] Review existing simulation features
- [x] Identify areas for improvement
- [x] Add multi-segment pillbugs with body and legs
- [x] Implement pillbug movement and wandering behavior
- [x] Add plant buds and flowers for reproduction
- [x] Implement sand physics with pile formation
- [x] Add pillbug reproduction
- [x] Improve plant growth patterns
- [x] Add water flow physics
- [x] Implement plant support checking
- [x] Add intelligent pillbug movement with MovementStrategy
- [x] Implement Y-shaped plant branching system
- [x] Add size-based feeding efficiency for pillbugs

## Immediate Priorities
- [x] Implement MovementStrategy enum for better pillbug AI
- [x] Add plant branching patterns (Y-shaped growth)
- [x] Improve size-based feeding efficiency
- [ ] Add plant-pillbug shelter interactions
- [x] Enhance bud development (branch vs flower choice)

## Potential Future Improvements
- [ ] Add seasonal variations
- [ ] Implement pillbug social behavior (clustering, trails)
- [ ] Add more plant species (grasses, trees, shrubs)
- [ ] Create food chain dynamics
- [ ] Add underground root systems
- [ ] Implement weather patterns beyond rain
- [ ] Genetic traits and inheritance
- [ ] Disease and infection mechanics
- [ ] Ecosystem succession over time

## Design Notes

### Pillbug Structure
- Head: Main control segment, eating, reproduction
- Body: Middle segment, grows at age 10
- Legs: Movement segment, grows at age 20
- Full lifecycle: birth → growth → reproduction → aging → decay

### Plant Structure
- Stem: Main support, grows upward, needs ground support
- Leaves: Photosynthesis, side growth
- Buds: Growth points at age 30+ that become flowers
- Flowers: Reproduction, spread seeds based on size
- Withered: Dying state before becoming nutrients

### Physics
- Gravity affects sand and water
- Water flows sideways when blocked
- Sand forms natural piles
- Plant parts check for structural support
- Unsupported plants fall or wither

### Ecosystem Dynamics
- Closed nutrient loop
- Size variations affect lifespan and behavior
- Day/night affects rain probability
- Well-fed pillbugs reproduce
- Plants seed new areas
- Intelligent pillbug AI (food-seeking, social, exploration behaviors)
- Y-shaped plant branching creates complex tree structures
- Size-based feeding efficiency creates ecological niches

### Recent Enhancements (2025-09-04)
- **Movement AI**: Pillbugs now actively seek food, sometimes socialize, and explore intelligently
- **Plant Branching**: Buds mature into branches (60%) or flowers (40%), creating Y-shaped growth
- **Feeding Efficiency**: Large pillbugs excel with big plants, small pillbugs efficient with small food
- **Branch Nutrition**: Branches provide more nutrition but are harder to eat than leaves