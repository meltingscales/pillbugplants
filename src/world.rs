use std::fmt;
use rand::{Rng, seq::SliceRandom};
use crate::types::{TileType, Size, random_size};

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
                            
                            // Plant growth - grows leaves, buds, and extends
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
                        
                        if new_age > 30 && rng.gen_bool(0.1 * growth_rate as f64) {
                            // Bud matures into flower
                            new_tiles[y][x] = TileType::PlantFlower(0, size);
                        } else if new_age > 50 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantBud(new_age, size);
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
                    TileType::PillbugHead(age, size) => {
                        pillbug_heads.push((x, y, size, age));
                        let mut new_age = age.saturating_add(1);
                        let mut well_fed = false;
                        
                        // Eating behavior - pillbugs eat plants and nutrients
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    match self.tiles[ny][nx] {
                                        TileType::PlantLeaf(_, _) | TileType::PlantWithered(_, _) => {
                                            if rng.gen_bool(0.2) {
                                                new_tiles[ny][nx] = TileType::Empty;
                                                new_age = new_age.saturating_sub(5);
                                                well_fed = true;
                                            }
                                        }
                                        TileType::Nutrient => {
                                            if rng.gen_bool(0.3) {
                                                new_tiles[ny][nx] = TileType::Empty;
                                                new_age = new_age.saturating_sub(3);
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
    
    fn move_pillbug(&self, new_tiles: &mut Vec<Vec<TileType>>, x: usize, y: usize, size: Size, age: u8) {
        let mut rng = rand::thread_rng();
        
        // Find connected body parts (should be adjacent)
        let mut segments = vec![(x, y, TileType::PillbugHead(age, size))];
        
        // Look for body segments adjacent to head
        for (dx, dy) in &[(0, 1), (1, 0), (-1, 0), (0, -1)] {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            if nx < self.width && ny < self.height {
                if let TileType::PillbugBody(b_age, b_size) = self.tiles[ny][nx] {
                    if b_size == size {  // Same bug
                        segments.push((nx, ny, TileType::PillbugBody(b_age, b_size)));
                        
                        // Look for legs adjacent to body
                        for (dx2, dy2) in &[(0, 1), (1, 0), (-1, 0), (0, -1)] {
                            let lx = (nx as i32 + dx2) as usize;
                            let ly = (ny as i32 + dy2) as usize;
                            if lx < self.width && ly < self.height {
                                if let TileType::PillbugLegs(l_age, l_size) = self.tiles[ly][lx] {
                                    if l_size == size {
                                        segments.push((lx, ly, TileType::PillbugLegs(l_age, l_size)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Possible movement directions
        let moves = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let (dx, dy) = moves.choose(&mut rng).unwrap();
        
        // Check if movement is possible
        let new_x = (x as i32 + dx) as usize;
        let new_y = (y as i32 + dy) as usize;
        
        if new_x < self.width && new_y < self.height {
            // Check if all segments can move
            let mut can_move = true;
            let mut new_positions = Vec::new();
            
            for (seg_x, seg_y, _) in &segments {
                let new_seg_x = (*seg_x as i32 + dx) as usize;
                let new_seg_y = (*seg_y as i32 + dy) as usize;
                
                if new_seg_x >= self.width || new_seg_y >= self.height {
                    can_move = false;
                    break;
                }
                
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
        
        // Count existing entities
        let mut plant_count = 0;
        let mut pillbug_count = 0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::PlantStem(_, _) => plant_count += 1,
                    TileType::PillbugHead(_, _) => pillbug_count += 1,
                    _ => {}
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