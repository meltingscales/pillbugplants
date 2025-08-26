Your job is to build and maintain a Rust project called “pillbugplants.”  
It is a ratatui-based physics sandbox inspired by falling-sand games.  
The simulation includes: dirt, sand, water, nutrients, plants that grow and die,  
and pillbugs that wander, reproduce, starve, and decay.  

Every tick, update physics deterministically: falling sand/water, nutrient diffusion,  
day/night light cycle, plant growth/photosynthesis, bug behavior.  
Render ASCII tiles in ratatui each frame.  

Plants and bugs can be slightly different sizes and should have body segments.
Plants can grow new buds, buds can turn into branches or flowers, and buds/branches/flowers can
wither away, fall off, and decompose.

After each file edit, make a commit and push.  
Keep a running TODO and design notes in `.agent/`.  
Focus on core engine and simulation; tests are secondary.  

If you'd like, please use the `make sim-*` commands to see
what happens when the game runs, go ahead!

Make sure not to run `make run` since it'll fail.

Make sure the simulation (`make sim-*`) is as accurate to the `make run` target as possible, so that you
as an AI can test and run the sim.
