# pillbugplants TODO

## Current Status ✅ (RECENTLY UPDATED)
- ✅ **Multi-segment plant system** - plants now have stems, leaves, buds, flowers with individual aging
- ✅ **Multi-segment pillbug system** - pillbugs have head, body, legs segments
- ✅ **Size variations** - Small/Medium/Large sizes with different lifespans & growth rates
- ✅ **Advanced physics** - 8-way support checking, realistic falling behavior
- ✅ **Day/night cycle** - affects plant photosynthesis and reproduction
- ✅ **Rain system** - spawns at night with variable intensity
- ✅ **Nutrient system** - closed loop: decomposition → nutrients → plant consumption
- ✅ **Complex aging** - age-based death thresholds that vary by size
- ✅ **Interactive UI** - ratatui with taxonomy panel (press 't')
- ✅ **CLI simulation mode** - headless runs with file output
- ✅ **Size inheritance** - offspring inherit parent size with variation
- ✅ **Water ecosystem** - plants near water grow faster and age slower
- ✅ **Terrain generation** - hills and valleys create diverse microhabitats
- ✅ **Smart creature AI** - pillbugs actively seek food sources
- ✅ **Natural decomposition** - aging plants shed parts creating debris

## Ecosystem Mechanics (DETAILED)
- **Plants**: Stems (structural), Leaves (photosynthesize), Buds (develop into stems/flowers/leaves), Flowers (reproduce)
- **Size effects**: Large = longer life/slower reproduction, Small = shorter life/faster reproduction  
- **Pillbugs**: Head (eats), Body (storage), Legs (movement) - size affects eating success & movement speed
- **Nutrient cycle**: Death → Nutrients → Plant consumption → Growth → Reproduction → Death
- **Support physics**: Plants need structural support or they fall

## High Priority TODO 🔥 (COMPLETED!)
- [x] **Fix compiler warnings** - ✅ Replaced inherent to_string with Display trait, fixed Vec parameters
- [x] **Improve pillbug coordination** - ✅ Body/legs now follow head movement in coordinated chain
- [x] **Enhanced plant withering** - ✅ Multi-stage decay: living → withered → nutrients
- [x] **Better plant physics** - ✅ Size-based stability system with support strength calculations

## Recent Major Enhancements 🚀 (JUST ADDED!)
- [x] **Hydrology System** - Water proximity boosts plant growth rate by 50% and slows aging
- [x] **Procedural Terrain** - Sine wave hill/valley generation with realistic water pooling in valleys
- [x] **Intelligent Foraging** - Pillbugs scan 3-tile radius for food and move strategically toward plants
- [x] **Seasonal Plant Shedding** - Aging stems drop leaves/flowers/buds based on age and size
- [x] **Ecosystem Stability** - Enhanced nutrient cycling with more realistic decomposition patterns

## Medium Priority 📋 (COMPLETED!)
- [x] **Water benefits** - ✅ plants near water grow 50% faster with slower aging
- [x] **Terrain variety** - ✅ hills, valleys using sine wave terrain generation  
- [x] **Smart pillbug AI** - ✅ food-seeking behavior with 3-tile detection radius
- [x] **Plant decomposition** - ✅ aging plants shed parts (leaves, flowers, buds)
- [ ] **Visual improvements** - better size differentiation, animations
- [ ] **Performance optimization** - profile heavy simulation loops

## Future Ideas 💡
- [ ] **Multiple species** - different plant/bug types with unique behaviors  
- [ ] **Seasonal cycles** - longer term environmental changes
- [ ] **Disease systems** - spreading conditions affecting populations
- [ ] **Save/load state** - pause/resume simulations
- [ ] **Statistics dashboard** - population tracking, ecosystem health metrics