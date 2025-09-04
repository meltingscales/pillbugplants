use std::fmt;
use rand::{Rng, seq::SliceRandom};
use crate::types::{TileType, Size, MovementStrategy, random_size};

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
        
        // Simple ground layer
        for y in (self.height - 8)..self.height {
            for x in 0..self.width {
                if rng.gen_bool(0.8) {
                    self.tiles[y][x] = TileType::Dirt;
                }
            }
        }
        
        // Add a few initial plants
        for _ in 0..3 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 10..self.height - 3);
            if self.tiles[y][x] == TileType::Empty {
                let size = random_size(&mut rng);
                self.tiles[y][x] = TileType::PlantStem(10, size);
            }
        }
        
        // Add a few initial pillbugs
        for _ in 0..2 {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(self.height - 8..self.height);
            if self.tiles[y][x] == TileType::Empty {
                let size = random_size(&mut rng);
                self.tiles[y][x] = TileType::PillbugHead(20, size);
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
        
        // Simple gravity - water and sand fall
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Sand | TileType::Water => {
                        if self.tiles[y + 1][x] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = self.tiles[y][x];
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
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::PlantStem(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > 100 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantStem(new_age, size);
                            
                            // Simple growth
                            if rng.gen_bool(0.1) {
                                if y > 0 && self.tiles[y - 1][x] == TileType::Empty {
                                    new_tiles[y - 1][x] = TileType::PlantLeaf(0, size);
                                }
                            }
                        }
                    }
                    TileType::PlantLeaf(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > 50 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantLeaf(new_age, size);
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
                        let mut new_age = age.saturating_add(1);
                        
                        // Simple eating behavior
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    if matches!(self.tiles[ny][nx], TileType::PlantLeaf(_, _) | TileType::PlantWithered(_, _)) {
                                        if rng.gen_bool(0.2) {
                                            new_tiles[ny][nx] = TileType::Nutrient;
                                            new_age = new_age.saturating_sub(5);
                                        }
                                    }
                                }
                            }
                        }
                        
                        if new_age > 150 {
                            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PillbugHead(new_age, size);
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
        
        self.tiles = new_tiles;
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
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..5);
                if self.tiles[y][x] == TileType::Empty {
                    let size = random_size(&mut rng);
                    self.tiles[y][x] = TileType::PillbugHead(10, size);
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