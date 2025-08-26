# pillbugplants TODO

## Current Status ✅ (UPDATED)
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

## Ecosystem Mechanics (DETAILED)
- **Plants**: Stems (structural), Leaves (photosynthesize), Buds (develop into stems/flowers/leaves), Flowers (reproduce)
- **Size effects**: Large = longer life/slower reproduction, Small = shorter life/faster reproduction  
- **Pillbugs**: Head (eats), Body (storage), Legs (movement) - size affects eating success & movement speed
- **Nutrient cycle**: Death → Nutrients → Plant consumption → Growth → Reproduction → Death
- **Support physics**: Plants need structural support or they fall

## High Priority TODO 🔥
- [ ] **Fix compiler warnings** - remove unused `pillbug_age` and `get_age` methods
- [ ] **Improve pillbug coordination** - body/legs should follow head movement better
- [ ] **Enhanced plant withering** - gradual decomposition rather than instant death
- [ ] **Better plant physics** - more realistic attachment/falling for different plant parts

## Medium Priority 📋  
- [ ] **Water benefits** - plants near water should grow better
- [ ] **Terrain variety** - hills, valleys, different soil types
- [ ] **Visual improvements** - better size differentiation, animations
- [ ] **Performance optimization** - profile heavy simulation loops

## Future Ideas 💡
- [ ] **Multiple species** - different plant/bug types with unique behaviors  
- [ ] **Seasonal cycles** - longer term environmental changes
- [ ] **Disease systems** - spreading conditions affecting populations
- [ ] **Save/load state** - pause/resume simulations
- [ ] **Statistics dashboard** - population tracking, ecosystem health metrics