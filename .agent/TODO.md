# Pillbugplants TODO

## Current Status
- Basic world generation working
- Simple plants with stems and leaves
- Basic pillbugs (head only)
- Day/night cycle
- Rain system
- Nutrient diffusion
- Size variations

## In Progress
- [ ] Add multi-segment pillbugs with body and legs

## TODO
- [ ] Implement pillbug movement and wandering behavior
- [ ] Add plant buds and flowers for reproduction
- [ ] Implement sand physics
- [ ] Add pillbug reproduction
- [ ] Improve plant growth patterns

## Completed
- [x] Explore existing codebase structure
- [x] Check current build status
- [x] Review existing simulation features
- [x] Identify areas for improvement

## Design Notes

### Pillbug Structure
- Head: Main control segment, contains eating behavior
- Body: Multiple segments (1-3), provides mass and health storage
- Legs: Movement segment, enables wandering

### Plant Structure
- Stem: Main support, grows upward
- Leaves: Photosynthesis, energy production
- Buds: Growth points that can become branches or flowers
- Flowers: Reproduction, seed spreading

### Physics
- Gravity for sand and water
- Water flows sideways when blocked
- Sand forms piles
- Support checking for plant parts