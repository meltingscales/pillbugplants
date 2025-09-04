use std::fmt;
use rand::{Rng, seq::SliceRandom};
use crate::types::{TileType, Size, random_size, MovementStrategy};

pub struct World {
    pub tiles: Vec<Vec<TileType>>,
    pub width: usize,
    pub height: usize,
    pub tick: u64,
    pub day_cycle: f32,
    pub rain_intensity: f32,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![TileType::Empty; width]; height];
        let mut world = World {
            tiles,
            width,
            height,
            tick: 0,
            day_cycle: 0.0,
            rain_intensity: 0.0,
        };
        
        world.generate_initial_world();
        world
    }
    
    pub fn update(&mut self) {
        self.tick += 1;
        self.day_cycle = (self.tick as f32 * 0.01) % (2.0 * std::f32::consts::PI);
        
        // Rain cycle - more likely during night
        let mut rng = rand::thread_rng();
        if self.day_cycle.sin() < -0.3 && rng.gen_bool(0.05) {
            self.rain_intensity = rng.gen_range(0.1..0.8);
        } else if rng.gen_bool(0.02) {
            self.rain_intensity *= 0.95; // Rain gradually stops
        }
        
        self.spawn_rain();
        self.update_physics();
        self.check_plant_support();
        self.diffuse_nutrients();
        self.update_life();
        self.spawn_entities();
    }
    
    pub fn is_day(&self) -> bool {
        self.day_cycle.sin() > 0.0
    }
    
    // Simplified stub implementations - these would be expanded from the original
    fn generate_initial_world(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Create varied terrain with dirt and sand
        for y in (self.height - 10)..self.height {
            for x in 0..self.width {
                let depth = self.height - y;
                if depth <= 2 {
                    // Top layers have mix of dirt and sand
                    if rng.gen_bool(0.3) {
                        self.tiles[y][x] = TileType::Sand;
                    } else if rng.gen_bool(0.7) {
                        self.tiles[y][x] = TileType::Dirt;
                    }
                } else if depth <= 5 {
                    // Middle layers mostly dirt
                    if rng.gen_bool(0.85) {
                        self.tiles[y][x] = TileType::Dirt;
                    } else if rng.gen_bool(0.5) {
                        self.tiles[y][x] = TileType::Sand;
                    }
                } else {
                    // Deep layers all dirt
                    if rng.gen_bool(0.95) {
                        self.tiles[y][x] = TileType::Dirt;
                    }
                }
            }
        }
        
        // Add some sand dunes/piles
        for _ in 0..3 {
            let x = rng.gen_range(5..self.width - 5);
            let y = self.height - 11;
            for dx in -2..=2 {
                for dy in 0..=1 {
                    let nx = (x as i32 + dx) as usize;
                    let ny = y + dy;
                    if nx < self.width && ny < self.height && rng.gen_bool(0.6) {
                        self.tiles[ny][nx] = TileType::Sand;
                    }
                }
            }
        }
        
        // Add a few initial plants
        for _ in 0..3 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 12..self.height - 3);
            if self.tiles[y][x] == TileType::Empty {
                let size = random_size(&mut rng);
                self.tiles[y][x] = TileType::PlantStem(10, size);
            }
        }
        
        // Add nutrients scattered around
        for _ in 0..5 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 15..self.height - 2);
            if self.tiles[y][x] == TileType::Empty {
                self.tiles[y][x] = TileType::Nutrient;
            }
        }
        
        // Add a few initial pillbugs with full body segments
        for _ in 0..2 {
            let x = rng.gen_range(2..self.width - 2);
            let y = rng.gen_range(self.height - 12..self.height - 2);
            if self.tiles[y][x] == TileType::Empty {
                let size = random_size(&mut rng);
                self.spawn_pillbug(x, y, size, 20);
            }
        }
    }
    
    fn spawn_rain(&mut self) {
        if self.rain_intensity > 0.1 {
            let mut rng = rand::thread_rng();
            let drops = (self.rain_intensity * self.width as f32 * 0.1) as usize;
            for _ in 0..drops {
                let x = rng.gen_range(0..self.width);
                if self.tiles[0][x] == TileType::Empty {
                    self.tiles[0][x] = TileType::Water;
                }
            }
        }
    }
    
    fn update_physics(&mut self) {
        let mut new_tiles = self.tiles.clone();
        let mut rng = rand::thread_rng();
        
        // Process physics from bottom to top for proper stacking
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Sand => {
                        // Sand falls straight down or diagonally to form piles
                        if new_tiles[y + 1][x] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = TileType::Sand;
                        } else if !matches!(new_tiles[y + 1][x], TileType::Empty | TileType::Water) {
                            // Try to slide diagonally if blocked
                            // Randomly choose left or right first for natural piling
                            let directions = if rng.gen_bool(0.5) {
                                vec![(-1, 1), (1, 1)]
                            } else {
                                vec![(1, 1), (-1, 1)]
                            };
                            
                            for (dx, dy) in directions {
                                let nx = (x as i32 + dx) as usize;
                                let ny = y + dy;
                                if nx < self.width && ny < self.height {
                                    if new_tiles[ny][nx] == TileType::Empty {
                                        new_tiles[y][x] = TileType::Empty;
                                        new_tiles[ny][nx] = TileType::Sand;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    TileType::Water => {
                        // Water flows down and spreads sideways
                        if new_tiles[y + 1][x] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = TileType::Water;
                        } else if !matches!(new_tiles[y + 1][x], TileType::Empty) {
                            // Water spreads sideways when blocked
                            // Try diagonal flow first
                            let directions = if rng.gen_bool(0.5) {
                                vec![(-1, 1), (1, 1), (-1, 0), (1, 0)]
                            } else {
                                vec![(1, 1), (-1, 1), (1, 0), (-1, 0)]
                            };
                            
                            for (dx, dy) in directions {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    if new_tiles[ny][nx] == TileType::Empty {
                                        // Higher chance to flow sideways for water
                                        if dy == 0 && rng.gen_bool(0.7) || dy == 1 {
                                            new_tiles[y][x] = TileType::Empty;
                                            new_tiles[ny][nx] = TileType::Water;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn check_plant_support(&mut self) {
        let mut new_tiles = self.tiles.clone();
        let mut rng = rand::thread_rng();
        
        // Check plant parts from top to bottom
        for y in 0..self.height - 1 {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::PlantLeaf(_, size) | TileType::PlantBud(_, size) | 
                    TileType::PlantBranch(_, size) | TileType::PlantFlower(_, size) => {
                        // Check for support in 8 directions
                        let mut has_support = false;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 { continue; }
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    match self.tiles[ny][nx] {
                                        TileType::PlantStem(_, _) | TileType::PlantBranch(_, _) | TileType::PlantRoot(_, _) | TileType::Dirt => {
                                            has_support = true;
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            if has_support { break; }
                        }
                        
                        // If no support, it falls or withers
                        if !has_support {
                            if rng.gen_bool(0.3) {
                                // Falls down if space below
                                if y + 1 < self.height && new_tiles[y + 1][x] == TileType::Empty {
                                    new_tiles[y + 1][x] = self.tiles[y][x];
                                    new_tiles[y][x] = TileType::Empty;
                                } else {
                                    // Withers if can't fall
                                    new_tiles[y][x] = TileType::PlantWithered(0, size);
                                }
                            }
                        }
                    }
                    TileType::PlantStem(age, size) => {
                        // Stems need support from below or adjacent stems
                        let mut has_support = false;
                        
                        // Check below
                        if y + 1 < self.height {
                            match self.tiles[y + 1][x] {
                                TileType::PlantStem(_, _) | TileType::PlantBranch(_, _) | TileType::PlantRoot(_, _) | TileType::Dirt | TileType::Sand => {
                                    has_support = true;
                                }
                                _ => {}
                            }
                        } else {
                            has_support = true; // Bottom row
                        }
                        
                        // Check adjacent for other stems
                        if !has_support {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                if nx < self.width {
                                    if let TileType::PlantStem(other_age, _) = self.tiles[y][nx] {
                                        if other_age > age {  // Older stems provide support
                                            has_support = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Unsupported stems fall or break
                        if !has_support && rng.gen_bool(0.2) {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn diffuse_nutrients(&mut self) {
        // Nutrients spread slowly
        let mut rng = rand::thread_rng();
        let mut new_tiles = self.tiles.clone();
        
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.tiles[y][x] == TileType::Nutrient && rng.gen_bool(0.1) {
                    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if self.tiles[ny][nx] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[ny][nx] = TileType::Nutrient;
                        }
                    }
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn update_life(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_tiles = self.tiles.clone();
        
        // Track pillbug segments for coordinated movement
        let mut pillbug_heads: Vec<(usize, usize, Size, u8)> = Vec::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::PlantStem(age, size) => {
                        let new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        
                        if new_age > (100.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantStem(new_age, size);
                            
                            // Plant growth - grows leaves, buds, roots, and extends
                            if rng.gen_bool(0.1 * growth_rate as f64) {
                                // Try to grow upward (extend stem)
                                if y > 0 && self.tiles[y - 1][x] == TileType::Empty && rng.gen_bool(0.3) {
                                    new_tiles[y - 1][x] = TileType::PlantStem(0, size);
                                }
                                // Grow leaves to the sides
                                else if x > 0 && self.tiles[y][x - 1] == TileType::Empty && rng.gen_bool(0.4) {
                                    new_tiles[y][x - 1] = TileType::PlantLeaf(0, size);
                                } else if x < self.width - 1 && self.tiles[y][x + 1] == TileType::Empty && rng.gen_bool(0.4) {
                                    new_tiles[y][x + 1] = TileType::PlantLeaf(0, size);
                                }
                                // Grow roots downward for nutrient absorption
                                else if y < self.height - 1 && matches!(self.tiles[y + 1][x], TileType::Empty | TileType::Dirt | TileType::Sand) && rng.gen_bool(0.5) {
                                    new_tiles[y + 1][x] = TileType::PlantRoot(0, size);
                                }
                                // Grow buds that will become flowers
                                else if y > 0 && self.tiles[y - 1][x] == TileType::Empty && rng.gen_bool(0.2) {
                                    new_tiles[y - 1][x] = TileType::PlantBud(0, size);
                                }
                            }
                        }
                    }
                    TileType::PlantLeaf(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > (50.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantLeaf(new_age, size);
                        }
                    }
                    TileType::PlantBud(age, size) => {
                        let new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        
                        if new_age > 25 && rng.gen_bool(0.15 * growth_rate as f64) {
                            // Bud can mature into branch or flower
                            if rng.gen_bool(0.6) {
                                // 60% chance to become a branch for Y-shaped growth
                                new_tiles[y][x] = TileType::PlantBranch(0, size);
                            } else {
                                // 40% chance to become flower for reproduction
                                new_tiles[y][x] = TileType::PlantFlower(0, size);
                            }
                        } else if new_age > 50 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantBud(new_age, size);
                        }
                    }
                    TileType::PlantBranch(age, size) => {
                        let new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        
                        if new_age > (100.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantBranch(new_age, size);
                            
                            // Branches grow diagonally and can spawn leaves/buds
                            if rng.gen_bool(0.08 * growth_rate as f64) {
                                // Diagonal growth patterns for Y-shaped branching
                                let directions = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
                                if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                                        if rng.gen_bool(0.7) {
                                            // Extend the branch diagonally
                                            new_tiles[ny][nx] = TileType::PlantBranch(0, size);
                                        } else if rng.gen_bool(0.6) {
                                            // Grow a leaf on the branch
                                            new_tiles[ny][nx] = TileType::PlantLeaf(0, size);
                                        } else {
                                            // Grow a bud for further branching
                                            new_tiles[ny][nx] = TileType::PlantBud(0, size);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    TileType::PlantFlower(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > (80.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantFlower(new_age, size);
                            
                            // Flowers spread seeds
                            if rng.gen_bool(0.05 * size.growth_rate_multiplier() as f64) {
                                let spread_distance = match size {
                                    Size::Small => 3,
                                    Size::Medium => 5,
                                    Size::Large => 7,
                                };
                                let dx = rng.gen_range(-(spread_distance as i32)..=spread_distance);
                                let dy = rng.gen_range(0..spread_distance);
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy as i32) as usize;
                                if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                                    let new_size = if rng.gen_bool(0.7) { size } else { random_size(&mut rng) };
                                    new_tiles[ny][nx] = TileType::PlantStem(0, new_size);
                                }
                            }
                        }
                    }
                    TileType::PlantWithered(age, size) => {
                        let new_age = age.saturating_add(2);
                        if new_age > 30 {
                            new_tiles[y][x] = TileType::Nutrient;
                        } else {
                            new_tiles[y][x] = TileType::PlantWithered(new_age, size);
                        }
                    }
                    TileType::PlantRoot(age, size) => {
                        let new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        
                        if new_age > (200.0 * size.lifespan_multiplier()) as u8 {
                            // Old roots wither and become nutrients
                            new_tiles[y][x] = TileType::Nutrient;
                        } else {
                            new_tiles[y][x] = TileType::PlantRoot(new_age, size);
                            
                            // Roots actively absorb nearby nutrients
                            let absorption_range = match size {
                                Size::Small => 1,
                                Size::Medium => 2,
                                Size::Large => 3,
                            };
                            
                            for dy in -(absorption_range as i32)..=(absorption_range as i32) {
                                for dx in -(absorption_range as i32)..=(absorption_range as i32) {
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    if nx < self.width && ny < self.height {
                                        if self.tiles[ny][nx] == TileType::Nutrient && rng.gen_bool(0.3 * growth_rate as f64) {
                                            // Absorb nutrients and potentially extend root network
                                            new_tiles[ny][nx] = TileType::Empty;
                                            
                                            // Chance to grow new root toward absorbed nutrient
                                            if rng.gen_bool(0.4) {
                                                let steps_x = if dx > 0 { 1 } else if dx < 0 { -1 } else { 0 };
                                                let steps_y = if dy > 0 { 1 } else if dy < 0 { -1 } else { 0 };
                                                let extend_x = (x as i32 + steps_x) as usize;
                                                let extend_y = (y as i32 + steps_y) as usize;
                                                
                                                if extend_x < self.width && extend_y < self.height 
                                                    && matches!(new_tiles[extend_y][extend_x], TileType::Empty | TileType::Dirt | TileType::Sand) {
                                                    new_tiles[extend_y][extend_x] = TileType::PlantRoot(0, size);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    TileType::PillbugHead(age, size) => {
                        pillbug_heads.push((x, y, size, age));
                        let mut new_age = age.saturating_add(1);
                        let mut well_fed = false;
                        
                        // Size-based eating behavior - efficiency depends on pillbug and food size
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    match self.tiles[ny][nx] {
                                        TileType::PlantLeaf(_, food_size) | TileType::PlantWithered(_, food_size) => {
                                            let eating_efficiency = self.calculate_eating_efficiency(size, food_size);
                                            if rng.gen_bool(eating_efficiency) {
                                                new_tiles[ny][nx] = TileType::Empty;
                                                // Nutrition gained depends on food size
                                                let nutrition = match food_size {
                                                    Size::Small => 3,
                                                    Size::Medium => 5,
                                                    Size::Large => 8,
                                                };
                                                new_age = new_age.saturating_sub(nutrition);
                                                well_fed = true;
                                            }
                                        }
                                        TileType::PlantBranch(_, food_size) => {
                                            // Branches are harder to eat but more nutritious
                                            let eating_efficiency = self.calculate_eating_efficiency(size, food_size) * 0.7;
                                            if rng.gen_bool(eating_efficiency) {
                                                new_tiles[ny][nx] = TileType::Empty;
                                                let nutrition = match food_size {
                                                    Size::Small => 4,
                                                    Size::Medium => 6,
                                                    Size::Large => 10,
                                                };
                                                new_age = new_age.saturating_sub(nutrition);
                                                well_fed = true;
                                            }
                                        }
                                        TileType::Nutrient => {
                                            // Nutrients are always easy to consume regardless of pillbug size
                                            if rng.gen_bool(0.4) {
                                                new_tiles[ny][nx] = TileType::Empty;
                                                new_age = new_age.saturating_sub(4);
                                                well_fed = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        
                        // Reproduction - well-fed mature pillbugs reproduce
                        if well_fed && age > 30 && age < 100 && rng.gen_bool(0.05 * size.growth_rate_multiplier() as f64) {
                            // Try to spawn baby pillbug nearby
                            for _ in 0..5 {  // Try 5 times to find a spot
                                let spawn_x = (x as i32 + rng.gen_range(-3..=3)).clamp(2, self.width as i32 - 3) as usize;
                                let spawn_y = (y as i32 + rng.gen_range(-2..=2)).clamp(0, self.height as i32 - 1) as usize;
                                
                                if new_tiles[spawn_y][spawn_x] == TileType::Empty {
                                    // Baby inherits size with chance of variation
                                    let baby_size = if rng.gen_bool(0.8) { size } else { random_size(&mut rng) };
                                    // Spawn baby pillbug (just head for now, body will grow)
                                    new_tiles[spawn_y][spawn_x] = TileType::PillbugHead(0, baby_size);
                                    break;
                                }
                            }
                        }
                        
                        if new_age > (150.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PillbugHead(new_age, size);
                        }
                    }
                    TileType::PillbugBody(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > (150.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PillbugBody(new_age, size);
                        }
                    }
                    TileType::PillbugLegs(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > (150.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PillbugLegs(new_age, size);
                        }
                    }
                    TileType::PillbugDecaying(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > 20 {
                            new_tiles[y][x] = TileType::Nutrient;
                        } else {
                            new_tiles[y][x] = TileType::PillbugDecaying(new_age, size);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Move pillbugs (heads control movement) and grow baby segments
        for (x, y, size, age) in pillbug_heads {
            // Baby pillbugs grow body segments as they mature
            if age == 10 {
                // Grow body segment
                for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                        new_tiles[ny][nx] = TileType::PillbugBody(age, size);
                        break;
                    }
                }
            } else if age == 20 {
                // Grow legs segment
                // Find the body segment first
                for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let bx = (x as i32 + dx) as usize;
                    let by = (y as i32 + dy) as usize;
                    if bx < self.width && by < self.height {
                        if let TileType::PillbugBody(_, b_size) = new_tiles[by][bx] {
                            if b_size == size {
                                // Try to add legs next to body
                                for (dx2, dy2) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                    let lx = (bx as i32 + dx2) as usize;
                                    let ly = (by as i32 + dy2) as usize;
                                    if lx < self.width && ly < self.height && new_tiles[ly][lx] == TileType::Empty {
                                        // Make sure it's not next to the head
                                        if lx != x || ly != y {
                                            new_tiles[ly][lx] = TileType::PillbugLegs(age, size);
                                            break;
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            
            if rng.gen_bool(0.3) {  // 30% chance to move each tick
                let movement_speed = match size {
                    Size::Small => 0.5,   // Small bugs move more often
                    Size::Medium => 0.3,
                    Size::Large => 0.2,   // Large bugs move slower
                };
                
                if rng.gen_bool(movement_speed) {
                    self.move_pillbug(&mut new_tiles, x, y, size, age);
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn calculate_eating_efficiency(&self, pillbug_size: Size, food_size: Size) -> f64 {
        // Base efficiency based on size matching
        let base_efficiency = match (pillbug_size, food_size) {
            // Perfect size matches are most efficient
            (Size::Small, Size::Small) => 0.35,
            (Size::Medium, Size::Medium) => 0.30,
            (Size::Large, Size::Large) => 0.25,
            
            // Large pillbugs can handle smaller food efficiently
            (Size::Large, Size::Medium) => 0.30,
            (Size::Large, Size::Small) => 0.40,
            (Size::Medium, Size::Small) => 0.35,
            
            // Smaller pillbugs struggle with larger food
            (Size::Small, Size::Medium) => 0.15,
            (Size::Small, Size::Large) => 0.05,
            (Size::Medium, Size::Large) => 0.20,
        };
        
        base_efficiency
    }
    
    fn determine_movement_strategy(&self, x: usize, y: usize, size: Size, age: u8) -> MovementStrategy {
        let mut rng = rand::thread_rng();
        
        // Young pillbugs are more exploratory
        if age < 20 {
            return MovementStrategy::Explore;
        }
        
        // Older pillbugs rest more
        if age > 120 {
            return if rng.gen_bool(0.6) { MovementStrategy::Rest } else { MovementStrategy::Explore };
        }
        
        let search_radius = match size {
            Size::Small => 3,
            Size::Medium => 4,
            Size::Large => 5,
        };
        
        // Look for food, social targets, and dangers in the area
        let mut food_positions = Vec::new();
        let mut pillbug_positions = Vec::new();
        let mut danger_positions = Vec::new();
        
        for dy in -(search_radius as i32)..=(search_radius as i32) {
            for dx in -(search_radius as i32)..=(search_radius as i32) {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    let tile = self.tiles[ny][nx];
                    
                    // Check for food using utility method
                    if tile.is_plant() || matches!(tile, TileType::Nutrient) {
                        // Only count living/withering plants as food
                        match tile {
                            TileType::PlantLeaf(_, _) | TileType::PlantWithered(_, _) | TileType::Nutrient => {
                                food_positions.push((dx, dy));
                            },
                            _ => {}
                        }
                    }
                    
                    // Check for social interactions
                    if let TileType::PillbugHead(_, other_size) = tile {
                        if other_size == size && !(dx == 0 && dy == 0) {
                            pillbug_positions.push((dx, dy));
                        }
                    }
                    
                    // Detect dangers - larger pillbugs, unstable areas, deep water
                    match tile {
                        TileType::PillbugHead(_, other_size) if other_size as u8 > size as u8 => {
                            // Larger pillbugs are threatening
                            danger_positions.push((dx, dy));
                        },
                        TileType::Water => {
                            // Standing water is dangerous
                            if dy > 0 {  // Water below is especially dangerous
                                danger_positions.push((dx, dy));
                            }
                        },
                        _ => {
                            // Check for unstable areas (floating sand)
                            if matches!(tile, TileType::Sand) {
                                // Check if sand has support
                                if ny + 1 < self.height && matches!(self.tiles[ny + 1][nx], TileType::Empty | TileType::Water) {
                                    danger_positions.push((dx, dy));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Priority: Avoid Danger > Food > Social > Explore
        if !danger_positions.is_empty() {
            // Find closest danger and move away from it
            let closest_danger = danger_positions.iter()
                .min_by_key(|(dx, dy)| dx.abs() + dy.abs())
                .unwrap();
            
            // Move in opposite direction
            let dir_x = if closest_danger.0 > 0 { -1 } else if closest_danger.0 < 0 { 1 } else { 0 };
            let dir_y = if closest_danger.1 > 0 { -1 } else if closest_danger.1 < 0 { 1 } else { 0 };
            
            MovementStrategy::Avoid((dir_x, dir_y))
        } else if !food_positions.is_empty() {
            // Find closest food
            let closest_food = food_positions.iter()
                .min_by_key(|(dx, dy)| dx.abs() + dy.abs())
                .unwrap();
            
            // Convert to unit direction
            let dir_x = if closest_food.0 > 0 { 1 } else if closest_food.0 < 0 { -1 } else { 0 };
            let dir_y = if closest_food.1 > 0 { 1 } else if closest_food.1 < 0 { -1 } else { 0 };
            
            MovementStrategy::SeekFood((dir_x, dir_y))
        } else if !pillbug_positions.is_empty() && rng.gen_bool(0.3) {
            // Sometimes seek social interaction
            let closest_pillbug = pillbug_positions.iter()
                .min_by_key(|(dx, dy)| dx.abs() + dy.abs())
                .unwrap();
            
            let dir_x = if closest_pillbug.0 > 0 { 1 } else if closest_pillbug.0 < 0 { -1 } else { 0 };
            let dir_y = if closest_pillbug.1 > 0 { 1 } else if closest_pillbug.1 < 0 { -1 } else { 0 };
            
            MovementStrategy::Social((dir_x, dir_y))
        } else {
            // Default to exploration or rest
            if rng.gen_bool(0.7) { MovementStrategy::Explore } else { MovementStrategy::Rest }
        }
    }
    
    fn move_pillbug(&self, new_tiles: &mut Vec<Vec<TileType>>, x: usize, y: usize, size: Size, age: u8) {
        let mut rng = rand::thread_rng();
        
        // Find connected body parts (should be adjacent)
        let mut segments = vec![(x, y, TileType::PillbugHead(age, size))];
        
        // Look for body segments adjacent to head using utility methods
        for (dx, dy) in &[(0, 1), (1, 0), (-1, 0), (0, -1)] {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            if nx < self.width && ny < self.height {
                let tile = self.tiles[ny][nx];
                // Use is_pillbug utility to check if it's a pillbug part
                if tile.is_pillbug() {
                    if let TileType::PillbugBody(_b_age, b_size) = tile {
                        if b_size == size {  // Same bug
                            segments.push((nx, ny, tile));
                            
                            // Look for legs adjacent to body
                            for (dx2, dy2) in &[(0, 1), (1, 0), (-1, 0), (0, -1)] {
                                let lx = (nx as i32 + dx2) as usize;
                                let ly = (ny as i32 + dy2) as usize;
                                if lx < self.width && ly < self.height {
                                    let leg_tile = self.tiles[ly][lx];
                                    if let TileType::PillbugLegs(_l_age, l_size) = leg_tile {
                                        if l_size == size && leg_tile.get_size() == Some(size) {
                                            segments.push((lx, ly, leg_tile));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Use movement strategy to determine direction
        let strategy = self.determine_movement_strategy(x, y, size, age);
        let (dx, dy) = strategy.get_movement_vector(&mut rng);
        
        // Skip movement if strategy says not to move
        if !strategy.should_move(&mut rng) {
            return;
        }
        
        // Check if movement is possible
        if dx == 0 && dy == 0 {
            return;  // No movement
        }
        
        let new_x = x as i32 + dx;
        let new_y = y as i32 + dy;
        
        if new_x >= 0 && new_x < self.width as i32 && new_y >= 0 && new_y < self.height as i32 {
            // Check if all segments can move
            let mut can_move = true;
            let mut new_positions = Vec::new();
            
            for (seg_x, seg_y, _) in &segments {
                let new_seg_x = *seg_x as i32 + dx;
                let new_seg_y = *seg_y as i32 + dy;
                
                if new_seg_x < 0 || new_seg_x >= self.width as i32 || new_seg_y < 0 || new_seg_y >= self.height as i32 {
                    can_move = false;
                    break;
                }
                
                let new_seg_x = new_seg_x as usize;
                let new_seg_y = new_seg_y as usize;
                
                // Check if destination is empty or will be vacated by another segment
                let dest_tile = new_tiles[new_seg_y][new_seg_x];
                if !matches!(dest_tile, TileType::Empty | TileType::Nutrient) {
                    // Check if it's occupied by another segment of the same bug
                    let occupied_by_self = segments.iter().any(|(sx, sy, _)| *sx == new_seg_x && *sy == new_seg_y);
                    if !occupied_by_self {
                        can_move = false;
                        break;
                    }
                }
                
                new_positions.push((new_seg_x, new_seg_y));
            }
            
            if can_move {
                // Clear old positions
                for (seg_x, seg_y, _) in &segments {
                    new_tiles[*seg_y][*seg_x] = TileType::Empty;
                }
                
                // Place segments in new positions
                for (i, (new_seg_x, new_seg_y)) in new_positions.iter().enumerate() {
                    new_tiles[*new_seg_y][*new_seg_x] = segments[i].2;
                }
            }
        }
    }
    
    fn spawn_pillbug(&mut self, x: usize, y: usize, size: Size, age: u8) {
        // Spawn a multi-segment pillbug (head-body-legs pattern)
        self.tiles[y][x] = TileType::PillbugHead(age, size);
        
        // Try to spawn body segment
        if x + 1 < self.width && self.tiles[y][x + 1] == TileType::Empty {
            self.tiles[y][x + 1] = TileType::PillbugBody(age, size);
            
            // Try to spawn legs segment
            if x + 2 < self.width && self.tiles[y][x + 2] == TileType::Empty {
                self.tiles[y][x + 2] = TileType::PillbugLegs(age, size);
            }
        } else if x > 0 && self.tiles[y][x - 1] == TileType::Empty {
            // Try the other direction
            self.tiles[y][x - 1] = TileType::PillbugBody(age, size);
            
            if x > 1 && self.tiles[y][x - 2] == TileType::Empty {
                self.tiles[y][x - 2] = TileType::PillbugLegs(age, size);
            }
        }
    }
    
    fn spawn_entities(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Count existing entities using utility methods
        let mut plant_count = 0;
        let mut pillbug_count = 0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.tiles[y][x];
                // Count plant stems as primary plant entities
                if matches!(tile, TileType::PlantStem(_, _)) {
                    plant_count += 1;
                }
                // Count pillbug heads as primary pillbug entities
                if matches!(tile, TileType::PillbugHead(_, _)) {
                    pillbug_count += 1;
                }
            }
        }
        
        // Spawn new entities if needed
        if plant_count < 2 {
            for _ in 0..(3 - plant_count) {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..5);
                if self.tiles[y][x] == TileType::Empty {
                    let size = random_size(&mut rng);
                    self.tiles[y][x] = TileType::PlantStem(5, size);
                }
            }
        }
        
        if pillbug_count < 1 {
            for _ in 0..(2 - pillbug_count) {
                let x = rng.gen_range(2..self.width.saturating_sub(2).max(3));
                let y = rng.gen_range(0..self.height.saturating_sub(2));
                if self.tiles[y][x] == TileType::Empty {
                    let size = random_size(&mut rng);
                    self.spawn_pillbug(x, y, size, 10);
                }
            }
        }
    }
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.tiles[y][x].to_char())?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Tick: {}", self.tick)?;
        writeln!(f, "Day/Night: {}", if self.is_day() { "Day" } else { "Night" })?;
        writeln!(f, "Rain intensity: {:.2}", self.rain_intensity)?;
        Ok(())
    }
}