# TODO - Pillbug Plants

## Current Status ✅
- ✅ Basic ratatui-based physics sandbox implemented
- ✅ Falling sand/water physics working
- ✅ Day/night cycle implemented
- ✅ Rain system (spawns during night, variable intensity)
- ✅ Plant/pillbug gravity with 8-directional support checking
- ✅ Nutrient diffusion system
- ✅ Plant aging, reproduction, starvation, death, and decomposition
- ✅ Pillbug aging, reproduction, eating, starvation, death, and decomposition
- ✅ Age-based visual fading (older entities get darker)
- ✅ ASCII rendering with colored tiles
- ✅ Dynamic world sizing based on terminal dimensions

## Ecosystem Mechanics
- Plants consume nutrients to slow aging and reproduce during daylight
- Plants die at age 200 and decompose into nutrients
- Pillbugs eat plants (converting them to nutrients) to slow aging
- Pillbugs reproduce when well-fed and die at age 180
- All deaths create nutrients, maintaining ecosystem balance
- Rain provides periodic water influx

## Next Iterations (Future)
- [ ] More complex plant structures (segments, buds, branches, flowers)
- [ ] Enhanced photosynthesis mechanics (light-based growth rates)
- [ ] Different plant/pillbug species
- [ ] Seasonal cycles affecting behavior
- [ ] More realistic physics (velocity, momentum)
- [ ] Save/load world state