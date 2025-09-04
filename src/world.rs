use std::fmt;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use rand::{Rng, seq::SliceRandom, prelude::IteratorRandom};
use crate::types::{TileType, Size, random_size, MovementStrategy, Season, Biome, random_biome};

// Optimization: Track tile changes without full array clones
#[derive(Debug)]
struct TileChange {
    x: usize,
    y: usize,
    old_tile: TileType,
    new_tile: TileType,
}

impl TileChange {
    fn new(x: usize, y: usize, old_tile: TileType, new_tile: TileType) -> Self {
        TileChange { x, y, old_tile, new_tile }
    }
}

// Ecosystem health and diversity statistics
#[derive(Debug)]
pub struct EcosystemStats {
    pub total_plants: usize,
    pub total_pillbugs: usize,
    pub water_coverage: usize,
    pub nutrient_count: usize,
    pub plant_health_ratio: f32,  // 0.0-1.0, higher means more healthy plants
    pub biome_diversity: usize,   // Number of different biomes present
}

// Seed with velocity for projectile motion
#[derive(Debug, Clone)]
struct SeedProjectile {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    seed_type: TileType, // The actual seed tile type
    age: u8,
    bounce_count: u8,    // How many times it has bounced
}

// Performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_update_time: Duration,
    pub physics_time: Duration,
    pub gravity_time: Duration,
    pub projectiles_time: Duration,
    pub wind_time: Duration,
    pub plant_support_time: Duration,
    pub nutrient_diffusion_time: Duration,
    pub life_update_time: Duration,
    pub spawn_entities_time: Duration,
    pub ticks_per_second: f64,
    pub frame_times: Vec<Duration>, // Last 60 frame times for averaging
}

pub struct World {
    pub tiles: Vec<Vec<TileType>>,
    pub biome_map: Vec<Vec<Biome>>, // Biome information for each region
    pub width: usize,
    pub height: usize,
    pub tick: u64,
    pub day_cycle: f32,
    pub rain_intensity: f32,
    pub season_cycle: f32,     // 0.0 = Spring, 0.25 = Summer, 0.5 = Fall, 0.75 = Winter
    pub temperature: f32,      // -1.0 to 1.0, affects growth rates
    pub humidity: f32,         // 0.0 to 1.0, affects rain and plant growth
    pub wind_direction: f32,   // 0.0 to 2π, direction of wind in radians
    pub wind_strength: f32,    // 0.0 to 1.0, strength of wind
    // Performance optimization: reuse buffers to reduce allocations
    tile_changes: Vec<TileChange>,
    // Seed projectiles in flight
    seed_projectiles: Vec<SeedProjectile>,
    // Performance monitoring
    pub performance: PerformanceMetrics,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![TileType::Empty; width]; height];
        let biome_map = vec![vec![Biome::Grassland; width]; height]; // Initialize with default biome
        let mut world = World {
            tiles,
            biome_map,
            width,
            height,
            tick: 0,
            day_cycle: 0.0,
            rain_intensity: 0.0,
            season_cycle: 0.0,   // Start in spring
            temperature: 0.3,    // Mild spring temperature
            humidity: 0.5,       // Moderate humidity
            wind_direction: 0.0, // Start with easterly wind
            wind_strength: 0.3,  // Moderate wind strength
            tile_changes: Vec::with_capacity(1000), // Pre-allocate for common case
            seed_projectiles: Vec::new(), // Start with no flying seeds
            performance: PerformanceMetrics {
                total_update_time: Duration::new(0, 0),
                physics_time: Duration::new(0, 0),
                gravity_time: Duration::new(0, 0),
                projectiles_time: Duration::new(0, 0),
                wind_time: Duration::new(0, 0),
                plant_support_time: Duration::new(0, 0),
                nutrient_diffusion_time: Duration::new(0, 0),
                life_update_time: Duration::new(0, 0),
                spawn_entities_time: Duration::new(0, 0),
                ticks_per_second: 0.0,
                frame_times: Vec::with_capacity(60),
            },
        };
        
        world.generate_biome_map();
        world.generate_initial_world();
        world
    }
    
    pub fn update(&mut self) {
        self.tick += 1;
        self.day_cycle = (self.tick as f32 * 0.01) % (2.0 * std::f32::consts::PI);
        
        // Seasonal cycle - complete season change every ~1600 ticks
        self.season_cycle = (self.tick as f32 * 0.001) % 1.0;
        
        // Update seasonal weather parameters
        self.update_seasonal_weather();
        
        // Rain cycle - affected by season and humidity
        let mut rng = rand::thread_rng();
        let base_rain_chance = 0.05 * self.humidity;
        let seasonal_rain_modifier = match self.get_current_season() {
            Season::Spring => 1.5,  // Rainy season
            Season::Summer => 0.7,  // Drier season
            Season::Fall => 1.3,    // Return of rains
            Season::Winter => 0.5,  // Cold, less rain
        };
        
        // Rain more likely during night and based on seasonal patterns
        if self.day_cycle.sin() < -0.3 && rng.gen_bool((base_rain_chance * seasonal_rain_modifier).min(1.0) as f64) {
            self.rain_intensity = rng.gen_range(0.1..(0.8 * self.humidity));
        } else if rng.gen_bool(0.02) {
            self.rain_intensity *= 0.95; // Rain gradually stops
        }
        
        // Timed system updates with performance profiling
        let update_start = Instant::now();
        
        self.spawn_rain();
        
        let physics_start = Instant::now();
        self.update_physics();
        self.performance.physics_time = physics_start.elapsed();
        
        let gravity_start = Instant::now();
        self.apply_gravity();
        self.performance.gravity_time = gravity_start.elapsed();
        
        let projectiles_start = Instant::now();
        self.update_seed_projectiles();
        self.performance.projectiles_time = projectiles_start.elapsed();
        
        let wind_start = Instant::now();
        self.process_wind_effects();
        self.performance.wind_time = wind_start.elapsed();
        
        let support_start = Instant::now();
        self.check_plant_support();
        self.performance.plant_support_time = support_start.elapsed();
        
        let diffusion_start = Instant::now();
        self.diffuse_nutrients();
        self.performance.nutrient_diffusion_time = diffusion_start.elapsed();
        
        let life_start = Instant::now();
        self.update_life();
        self.performance.life_update_time = life_start.elapsed();
        
        let spawn_start = Instant::now();
        self.spawn_entities();
        self.performance.spawn_entities_time = spawn_start.elapsed();
        
        // Calculate total update time and performance metrics
        self.performance.total_update_time = update_start.elapsed();
        
        // Maintain rolling average of frame times (last 60 frames)
        if self.performance.frame_times.len() >= 60 {
            self.performance.frame_times.remove(0);
        }
        self.performance.frame_times.push(self.performance.total_update_time);
        
        // Calculate TPS based on average frame time
        if !self.performance.frame_times.is_empty() {
            let avg_frame_time: Duration = self.performance.frame_times.iter().sum::<Duration>() / self.performance.frame_times.len() as u32;
            self.performance.ticks_per_second = if avg_frame_time.as_secs_f64() > 0.0 {
                1.0 / avg_frame_time.as_secs_f64()
            } else {
                0.0
            };
        }
    }
    
    pub fn is_day(&self) -> bool {
        self.day_cycle.sin() > 0.0
    }
    
    pub fn get_projectile_count(&self) -> usize {
        self.seed_projectiles.len()
    }
    
    pub fn get_current_season(&self) -> Season {
        match (self.season_cycle * 4.0) as u32 % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Fall,
            _ => Season::Winter,
        }
    }
    
    pub fn get_season_name(&self) -> &'static str {
        match self.get_current_season() {
            Season::Spring => "Spring",
            Season::Summer => "Summer", 
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        }
    }
    
    fn update_seasonal_weather(&mut self) {
        // Calculate target temperature and humidity based on season
        let (target_temp, target_humidity) = match self.get_current_season() {
            Season::Spring => (0.3, 0.7),   // Mild and moist
            Season::Summer => (0.8, 0.3),   // Hot and dry
            Season::Fall => (0.1, 0.6),     // Cool and moderately moist
            Season::Winter => (-0.5, 0.4),  // Cold and variable
        };
        
        // Add some seasonal variation using sine waves
        let season_progress = (self.season_cycle * 4.0) % 1.0; // Progress within current season
        let temp_variation = (season_progress * 2.0 * std::f32::consts::PI).sin() * 0.2;
        let humidity_variation = ((season_progress + 0.5) * 2.0 * std::f32::consts::PI).sin() * 0.15;
        
        // Gradually adjust temperature and humidity toward targets
        let target_temp_with_var = (target_temp + temp_variation).clamp(-1.0, 1.0);
        let target_humidity_with_var = (target_humidity + humidity_variation).clamp(0.1, 1.0);
        
        self.temperature += (target_temp_with_var - self.temperature) * 0.02; // Slow change
        self.humidity += (target_humidity_with_var - self.humidity) * 0.03;   // Slightly faster change
        
        // Clamp values to valid ranges
        self.temperature = self.temperature.clamp(-1.0, 1.0);
        self.humidity = self.humidity.clamp(0.1, 1.0);
        
        // Update wind patterns - varies by season and has some random variation
        let target_wind_direction = match self.get_current_season() {
            Season::Spring => 0.5,      // Easterly winds (spring breezes)
            Season::Summer => 1.5,      // Southerly winds (hot air rising)
            Season::Fall => 4.0,        // Westerly winds (storm systems)
            Season::Winter => 2.5,      // Northerly winds (cold fronts)
        };
        
        let target_wind_strength = match self.get_current_season() {
            Season::Spring => 0.4 + self.humidity * 0.3,  // Variable spring winds
            Season::Summer => 0.2 + (1.0 - self.humidity) * 0.4, // Hot, dry winds
            Season::Fall => 0.6 + self.rain_intensity * 0.4,     // Storm-driven winds
            Season::Winter => 0.5 + (1.0 + self.temperature) * 0.2, // Cold winds
        };
        
        // Add some natural variation
        let wind_dir_variation = ((self.tick as f32 * 0.003).sin() + (self.tick as f32 * 0.007).cos()) * 0.5;
        let wind_str_variation = ((self.tick as f32 * 0.005).sin()) * 0.1;
        
        // Gradually adjust wind toward targets
        let target_dir_with_var = (target_wind_direction + wind_dir_variation) % (2.0 * std::f32::consts::PI);
        let target_str_with_var = (target_wind_strength + wind_str_variation).clamp(0.0, 1.0);
        
        self.wind_direction += (target_dir_with_var - self.wind_direction) * 0.05; // Slow change
        self.wind_strength += (target_str_with_var - self.wind_strength) * 0.08;   // Slightly faster
        
        self.wind_direction = self.wind_direction % (2.0 * std::f32::consts::PI);
        self.wind_strength = self.wind_strength.clamp(0.0, 1.0);
    }
    
    pub fn get_seasonal_growth_modifier(&self) -> f32 {
        // Base seasonal multipliers
        let season_multiplier = match self.get_current_season() {
            Season::Spring => 1.4,  // Peak growth season
            Season::Summer => 0.8,  // Slower growth due to heat/drought
            Season::Fall => 1.1,    // Second growth period
            Season::Winter => 0.3,  // Minimal growth
        };
        
        // Temperature effects (optimal around 0.2-0.4)
        let temp_multiplier = if self.temperature > 0.6 {
            0.6 // Too hot, growth slows
        } else if self.temperature < -0.3 {
            0.2 // Too cold, growth nearly stops
        } else {
            1.0 + (0.3 - (self.temperature - 0.3).abs()) * 0.5 // Optimal range bonus
        };
        
        // Humidity effects (plants need moisture)
        let humidity_multiplier = 0.5 + self.humidity * 0.8; // 0.5 to 1.3 range
        
        season_multiplier * temp_multiplier * humidity_multiplier
    }
    
    /// Generate biome map using regions and noise-like patterns
    fn generate_biome_map(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Divide world into regions and assign biomes
        let region_size = 8; // Each biome region is roughly 8x8 tiles
        
        for ry in 0..(self.height / region_size + 1) {
            for rx in 0..(self.width / region_size + 1) {
                let biome = random_biome(&mut rng);
                
                // Fill region with this biome, with some variation at edges
                for y in (ry * region_size)..((ry + 1) * region_size).min(self.height) {
                    for x in (rx * region_size)..((rx + 1) * region_size).min(self.width) {
                        // Add some fuzzy edges between biomes
                        let distance_from_center = ((x % region_size) as f32 - region_size as f32 / 2.0).abs()
                            + ((y % region_size) as f32 - region_size as f32 / 2.0).abs();
                        
                        if distance_from_center < region_size as f32 * 0.3 || rng.gen_bool(0.7) {
                            self.biome_map[y][x] = biome;
                        } else if rng.gen_bool(0.5) {
                            // Sometimes blend with neighboring biomes
                            self.biome_map[y][x] = random_biome(&mut rng);
                        }
                    }
                }
            }
        }
    }

    /// Get biome at a specific coordinate
    pub fn get_biome_at(&self, x: usize, y: usize) -> Biome {
        if x < self.width && y < self.height {
            self.biome_map[y][x]
        } else {
            Biome::Grassland // Default fallback
        }
    }

    // Simplified stub implementations - these would be expanded from the original
    fn generate_initial_world(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Create varied terrain with dirt and sand based on biome preferences
        for y in (self.height - 10)..self.height {
            for x in 0..self.width {
                let biome = self.get_biome_at(x, y);
                let (dirt_pref, sand_pref) = biome.get_terrain_preferences();
                let depth = self.height - y;
                
                if depth <= 2 {
                    // Top layers influenced by biome
                    if rng.gen_bool(sand_pref as f64) {
                        self.tiles[y][x] = TileType::Sand;
                    } else if rng.gen_bool(dirt_pref as f64) {
                        self.tiles[y][x] = TileType::Dirt;
                    }
                } else if depth <= 5 {
                    // Middle layers mostly follow biome preferences but favor dirt
                    let dirt_chance = (dirt_pref * 0.85 + 0.15).min(0.95);
                    let sand_chance = sand_pref * 0.5;
                    
                    if rng.gen_bool(dirt_chance as f64) {
                        self.tiles[y][x] = TileType::Dirt;
                    } else if rng.gen_bool(sand_chance as f64) {
                        self.tiles[y][x] = TileType::Sand;
                    }
                } else {
                    // Deep layers mostly dirt but still biome-influenced
                    let dirt_chance = (dirt_pref * 0.1 + 0.85).min(0.98);
                    if rng.gen_bool(dirt_chance as f64) {
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
        
        // Add initial plants based on biome preferences
        let base_plant_count = 8; // More plants than before to show biome differences
        for _ in 0..base_plant_count {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 12..self.height - 3);
            if self.tiles[y][x] == TileType::Empty {
                let biome = self.get_biome_at(x, y);
                let plant_chance = biome.plant_growth_modifier() * 0.6; // Base 60% chance
                
                if rng.gen_bool(plant_chance as f64) {
                    let size = random_size(&mut rng);
                    self.tiles[y][x] = TileType::PlantStem(10, size);
                    
                    // In Woodland biomes, sometimes add immediate roots
                    if biome == Biome::Woodland && rng.gen_bool(0.4) {
                        if y + 1 < self.height && self.tiles[y + 1][x] != TileType::Empty {
                            self.tiles[y + 1][x] = TileType::PlantRoot(5, size);
                        }
                    }
                }
            }
        }
        
        // Add nutrients based on biome richness
        let base_nutrient_count = 10;
        for _ in 0..base_nutrient_count {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 15..self.height - 2);
            if self.tiles[y][x] == TileType::Empty {
                let biome = self.get_biome_at(x, y);
                let nutrient_chance = biome.nutrient_modifier() * 0.5; // Base 50% chance
                
                if rng.gen_bool(nutrient_chance as f64) {
                    self.tiles[y][x] = TileType::Nutrient;
                }
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
                    // Check biome for rain accumulation bonus
                    let biome = self.get_biome_at(x, 0);
                    let accumulation_bonus = biome.rain_accumulation_bonus();
                    
                    // Higher chance for rain to "stick" in wetlands, lower in drylands
                    if rng.gen_bool((accumulation_bonus * 0.8).min(1.0) as f64) {
                        // Rain starts with moderate depth
                        let rain_depth = (50.0 + self.rain_intensity * 100.0) as u8;
                        self.tiles[0][x] = TileType::Water(rain_depth);
                    }
                }
            }
        }
    }
    
    // Performance optimization: Apply tile changes efficiently without full clones
    fn apply_tile_changes(&mut self) {
        for change in self.tile_changes.drain(..) {
            if change.x < self.width && change.y < self.height {
                self.tiles[change.y][change.x] = change.new_tile;
            }
        }
    }
    
    // Helper to queue a tile change for later application
    fn queue_tile_change(&mut self, x: usize, y: usize, new_tile: TileType) {
        if x < self.width && y < self.height {
            let old_tile = self.tiles[y][x];
            if old_tile != new_tile {
                self.tile_changes.push(TileChange::new(x, y, old_tile, new_tile));
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
                        } else if new_tiles[y + 1][x].blocks_water() {
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
                    TileType::Water(depth) => {
                        self.process_water_physics(x, y, depth, &mut new_tiles, &mut rng);
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    /// Update seed projectiles flying through the air
    fn update_seed_projectiles(&mut self) {
        let mut i = 0;
        
        // Process each projectile
        while i < self.seed_projectiles.len() {
            let mut projectile = self.seed_projectiles[i].clone();
            
            // Apply gravity
            projectile.velocity_y += 0.2; // Gravity acceleration
            
            // Apply wind effects
            let wind_x = self.wind_direction.cos() * self.wind_strength * 0.3;
            let wind_y = self.wind_direction.sin() * self.wind_strength * 0.3;
            
            // Wind affects lighter seeds more
            if let TileType::Seed(_, size) = projectile.seed_type {
                let wind_susceptibility = match size {
                    Size::Small => 1.0,
                    Size::Medium => 0.7,
                    Size::Large => 0.4,
                };
                projectile.velocity_x += wind_x * wind_susceptibility;
                projectile.velocity_y += wind_y * wind_susceptibility;
            }
            
            // Update position
            projectile.x += projectile.velocity_x;
            projectile.y += projectile.velocity_y;
            
            // Check bounds
            if projectile.x < 0.0 || projectile.x >= self.width as f32 || 
               projectile.y < 0.0 || projectile.y >= self.height as f32 {
                // Remove projectile that went out of bounds
                self.seed_projectiles.remove(i);
                continue;
            }
            
            let tile_x = projectile.x.floor() as usize;
            let tile_y = projectile.y.floor() as usize;
            
            // Check for collision
            match self.tiles[tile_y][tile_x] {
                TileType::Empty => {
                    // Continue flying
                    self.seed_projectiles[i] = projectile;
                    i += 1;
                }
                TileType::Water(_) => {
                    // Seed lands in water, stops moving but stays alive
                    self.tiles[tile_y][tile_x] = projectile.seed_type;
                    self.seed_projectiles.remove(i);
                }
                _ => {
                    // Hit solid object - try to bounce or stop
                    if projectile.bounce_count < 2 && projectile.velocity_y > 1.0 {
                        // Bounce with reduced velocity
                        projectile.velocity_y = -projectile.velocity_y * 0.4;
                        projectile.velocity_x *= 0.7;
                        projectile.bounce_count += 1;
                        
                        // Move slightly away from collision point
                        if projectile.velocity_y > 0.0 {
                            projectile.y = tile_y as f32 + 1.1;
                        } else {
                            projectile.y = tile_y as f32 - 0.1;
                        }
                        
                        self.seed_projectiles[i] = projectile;
                        i += 1;
                    } else {
                        // Find empty adjacent space to land
                        let adjacent_positions = [
                            (tile_x, tile_y.saturating_sub(1)),
                            (tile_x.saturating_sub(1), tile_y),
                            (tile_x.saturating_add(1).min(self.width - 1), tile_y),
                            (tile_x, tile_y.saturating_add(1).min(self.height - 1)),
                        ];
                        
                        let mut landed = false;
                        for (ax, ay) in adjacent_positions.iter() {
                            if self.tiles[*ay][*ax] == TileType::Empty {
                                self.tiles[*ay][*ax] = projectile.seed_type;
                                landed = true;
                                break;
                            }
                        }
                        
                        if !landed {
                            // No space to land, seed is destroyed
                            // Could become nutrient instead if we want
                        }
                        
                        self.seed_projectiles.remove(i);
                    }
                }
            }
        }
    }
    
    /// Apply gravity to unsupported entities (pillbugs and loose objects) - OPTIMIZED
    fn apply_gravity(&mut self) {
        let mut rng = rand::thread_rng();
        let mut processed_positions = HashSet::new();
        
        // OPTIMIZATION: Collect potentially unstable entities first, skip others entirely  
        let mut unstable_entities = Vec::new();
        let underground_threshold = self.height.saturating_sub(self.height / 4); // Bottom 25% of world
        
        for y in 0..self.height.saturating_sub(1) {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    tile if tile.is_pillbug() => {
                        // Quick stability check - if directly supported, skip expensive group analysis
                        if y + 1 < self.height {
                            let below = self.tiles[y + 1][x];
                            if below.can_support_plants() || below.is_plant() || below.is_pillbug() {
                                continue; // Obviously supported, skip
                            }
                        }
                        unstable_entities.push((x, y, "pillbug"));
                    }
                    tile if tile.is_plant() => {
                        // MAJOR OPTIMIZATION: Skip roots that are deep underground (bottom 25% of world)
                        if matches!(tile, TileType::PlantRoot(_, _)) && y >= underground_threshold {
                            continue; // Deep roots don't need gravity checks
                        }
                        
                        // Also skip roots buried in soil at any depth
                        if matches!(tile, TileType::PlantRoot(_, _)) && self.is_root_in_soil(x, y) {
                            continue;
                        }
                        
                        // Quick stability check for other plant parts
                        if y + 1 < self.height {
                            let below = self.tiles[y + 1][x];
                            if below.can_support_plants() || below.is_plant() {
                                continue; // Obviously supported, skip
                            }
                        }
                        unstable_entities.push((x, y, "plant"));
                    }
                    _ => {}
                }
            }
        }
        
        // OPTIMIZATION: Use tile change queue instead of full clone
        self.tile_changes.clear();
        
        // Process only potentially unstable entities
        for (x, y, entity_type) in unstable_entities {
            if processed_positions.contains(&(x, y)) {
                continue; // Already processed as part of a group
            }
            
            match entity_type {
                "pillbug" => {
                    let connected_segments = self.find_connected_pillbug_segments(x, y);
                    if self.is_pillbug_group_unsupported(&connected_segments) {
                        if self.can_move_group_down_simple(&connected_segments) {
                            // Queue moves instead of modifying directly
                            for (seg_x, seg_y, tile) in &connected_segments {
                                self.queue_tile_change(*seg_x, *seg_y, TileType::Empty);
                                self.queue_tile_change(*seg_x, seg_y + 1, *tile);
                            }
                            // Mark all segments as processed
                            for (seg_x, seg_y, _) in &connected_segments {
                                processed_positions.insert((*seg_x, *seg_y));
                            }
                        }
                    }
                }
                "plant" => {
                    let connected_plant_parts = self.find_connected_plant_parts(x, y);
                    if self.is_plant_group_unsupported(&connected_plant_parts) {
                        if self.can_move_group_down_simple(&connected_plant_parts) {
                            // Queue moves instead of modifying directly
                            for (part_x, part_y, tile) in &connected_plant_parts {
                                self.queue_tile_change(*part_x, *part_y, TileType::Empty);
                                self.queue_tile_change(*part_x, part_y + 1, *tile);
                            }
                            // Mark all parts as processed
                            for (part_x, part_y, _) in &connected_plant_parts {
                                processed_positions.insert((*part_x, *part_y));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        // OPTIMIZATION: Handle simple particle gravity using tile changes
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Seed(age, size) => {
                        if self.tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.6) {
                            self.queue_tile_change(x, y, TileType::Empty);
                            self.queue_tile_change(x, y + 1, TileType::Seed(age, size));
                        }
                    }
                    TileType::Spore(age) => {
                        if self.tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.3) {
                            self.queue_tile_change(x, y, TileType::Empty);
                            self.queue_tile_change(x, y + 1, TileType::Spore(age));
                        }
                    }
                    TileType::Nutrient => {
                        if self.tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.2) {
                            self.queue_tile_change(x, y, TileType::Empty);
                            self.queue_tile_change(x, y + 1, TileType::Nutrient);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Apply all gravity changes at once
        self.apply_tile_changes();
    }
    
    /// Check if a pillbug segment is completely unsupported (no solid ground, plants, or connected pillbug parts)
    fn is_pillbug_segment_unsupported(&self, x: usize, y: usize) -> bool {
        // Already at bottom - supported by world boundary
        if y >= self.height - 1 {
            return false;
        }
        
        // Check all 8 directions for support
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; } // Skip self
                
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                
                if nx < self.width && ny < self.height {
                    match self.tiles[ny][nx] {
                        // Solid support
                        TileType::Dirt | TileType::Sand => return false,
                        // Plant support
                        TileType::PlantStem(_, _) | TileType::PlantRoot(_, _) | TileType::PlantBranch(_, _) => return false,
                        // Other pillbug support (connected segments)
                        tile if tile.is_pillbug() => {
                            // Only count as support if the other segment is also supported or connected to something solid
                            if dy == 1 || self.has_solid_support_nearby(nx, ny) {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        true // No support found
    }
    
    /// Check if a position has solid support nearby (for connected pillbug segments)
    fn has_solid_support_nearby(&self, x: usize, y: usize) -> bool {
        // Bottom boundary is always solid
        if y >= self.height - 1 {
            return true;
        }
        
        // Check adjacent positions for solid support
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                
                if nx < self.width && ny < self.height {
                    match self.tiles[ny][nx] {
                        TileType::Dirt | TileType::Sand | TileType::PlantStem(_, _) | 
                        TileType::PlantRoot(_, _) | TileType::PlantBranch(_, _) => return true,
                        _ => {}
                    }
                }
            }
        }
        
        false
    }
    
    /// Check if a root is completely surrounded by soil (optimization for gravity)
    fn is_root_in_soil(&self, x: usize, y: usize) -> bool {
        // Check all 8 surrounding positions
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; } // Skip self
                
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                
                if nx < self.width && ny < self.height {
                    match self.tiles[ny][nx] {
                        // These tiles count as "soil" for root stability
                        TileType::Dirt | TileType::NutrientDirt(_) | TileType::Sand => {
                            // Good, surrounded by soil
                        }
                        TileType::PlantRoot(_, _) => {
                            // Other roots also provide stability
                        }
                        _ => {
                            // Empty space or other tiles - not completely buried
                            return false;
                        }
                    }
                } else {
                    // Edge of world - counts as not buried
                    return false;
                }
            }
        }
        
        true // Root is completely surrounded by soil/other roots
    }
    
    /// Find all connected pillbug segments starting from a given position
    fn find_connected_pillbug_segments(&self, start_x: usize, start_y: usize) -> Vec<(usize, usize, TileType)> {
        let mut connected = Vec::new();
        let mut visited = HashSet::new();
        let mut to_check = vec![(start_x, start_y)];
        
        while let Some((x, y)) = to_check.pop() {
            if visited.contains(&(x, y)) {
                continue;
            }
            visited.insert((x, y));
            
            let tile = self.tiles[y][x];
            if tile.is_pillbug() {
                connected.push((x, y, tile));
                
                // Check adjacent positions for more pillbug parts
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        
                        if nx < self.width && ny < self.height && !visited.contains(&(nx, ny)) {
                            let neighbor_tile = self.tiles[ny][nx];
                            if neighbor_tile.is_pillbug() {
                                // Check if sizes match (same pillbug)
                                if let (Some(size1), Some(size2)) = (tile.get_size(), neighbor_tile.get_size()) {
                                    if size1 == size2 {
                                        to_check.push((nx, ny));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        connected
    }
    
    /// Find all connected plant parts starting from a given position
    fn find_connected_plant_parts(&self, start_x: usize, start_y: usize) -> Vec<(usize, usize, TileType)> {
        let mut connected = Vec::new();
        let mut visited = HashSet::new();
        let mut to_check = vec![(start_x, start_y)];
        
        while let Some((x, y)) = to_check.pop() {
            if visited.contains(&(x, y)) {
                continue;
            }
            visited.insert((x, y));
            
            let tile = self.tiles[y][x];
            if tile.is_plant() {
                connected.push((x, y, tile));
                
                // Check adjacent positions for more plant parts
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        
                        if nx < self.width && ny < self.height && !visited.contains(&(nx, ny)) {
                            let neighbor_tile = self.tiles[ny][nx];
                            if neighbor_tile.is_plant() {
                                // Check if sizes match (same plant)
                                if let (Some(size1), Some(size2)) = (tile.get_size(), neighbor_tile.get_size()) {
                                    if size1 == size2 {
                                        to_check.push((nx, ny));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        connected
    }
    
    /// Check if an entire pillbug group is unsupported
    fn is_pillbug_group_unsupported(&self, segments: &[(usize, usize, TileType)]) -> bool {
        // If any segment has solid support, the entire group is supported
        for (x, y, _) in segments {
            if !self.is_pillbug_segment_unsupported(*x, *y) {
                return false;
            }
        }
        true
    }
    
    /// Check if an entire plant group is unsupported
    fn is_plant_group_unsupported(&self, parts: &[(usize, usize, TileType)]) -> bool {
        // Check if any part has solid support (dirt, sand, other solid ground)
        for (x, y, _) in parts {
            // Check all 8 directions for solid support
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    
                    let nx = (*x as i32 + dx) as usize;
                    let ny = (*y as i32 + dy) as usize;
                    
                    if nx < self.width && ny < self.height {
                        match self.tiles[ny][nx] {
                            TileType::Dirt | TileType::Sand => return false, // Solid support found
                            _ => {}
                        }
                    }
                }
            }
            
            // Also check if at world bottom
            if *y >= self.height - 1 {
                return false;
            }
        }
        true
    }
    
    /// Check if a group can move down (all spaces below are empty)
    fn can_move_group_down(&self, group: &[(usize, usize, TileType)], new_tiles: &Vec<Vec<TileType>>) -> bool {
        for (x, y, _) in group {
            // Check if the position below is available
            if *y + 1 >= self.height {
                return false; // Can't fall past bottom
            }
            
            let below_pos = (*x, *y + 1);
            let below_tile = new_tiles[below_pos.1][below_pos.0];
            
            // Position must be empty or will be vacated by another group member falling
            if below_tile != TileType::Empty {
                // Check if it's occupied by another member of the same group
                let occupied_by_group = group.iter().any(|(gx, gy, _)| *gx == below_pos.0 && *gy == below_pos.1);
                if !occupied_by_group {
                    return false;
                }
            }
        }
        true
    }
    
    /// Simple version that checks current tiles (optimized for gravity)
    fn can_move_group_down_simple(&self, group: &[(usize, usize, TileType)]) -> bool {
        for (x, y, _) in group {
            // Check if the position below is available
            if *y + 1 >= self.height {
                return false; // Can't fall past bottom
            }
            
            let below_tile = self.tiles[*y + 1][*x];
            
            // Position must be empty or will be vacated by another group member falling
            if below_tile != TileType::Empty {
                // Check if it's occupied by another member of the same group
                let occupied_by_group = group.iter().any(|(gx, gy, _)| *gx == *x && *gy == *y + 1);
                if !occupied_by_group {
                    return false;
                }
            }
        }
        true
    }
    
    /// Move a group down by one position
    fn move_group_down(&self, group: &[(usize, usize, TileType)], new_tiles: &mut Vec<Vec<TileType>>) {
        // First clear all current positions
        for (x, y, _) in group {
            new_tiles[*y][*x] = TileType::Empty;
        }
        
        // Then place all tiles in new positions
        for (x, y, tile) in group {
            new_tiles[*y + 1][*x] = *tile;
        }
    }
    
    /// Enhanced water physics with depth-based flow mechanics and pooling
    fn process_water_physics(&self, x: usize, y: usize, depth: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let biome = self.get_biome_at(x, y);
        let moisture_retention = biome.moisture_retention();
        
        // Water wetting earth - water can soak into dirt/sand instead of just piling up
        if depth <= 80 && rng.gen_bool(0.15) { // Moderate chance for light/medium water to soak in
            // Check if there's dirt or sand adjacent that can absorb water
            let absorption_positions = [
                (x, y.saturating_add(1).min(self.height - 1)), // Below
                (x.saturating_sub(1), y), (x.saturating_add(1).min(self.width - 1), y), // Sides
            ];
            
            for (ax, ay) in absorption_positions.iter() {
                if *ax < self.width && *ay < self.height {
                    match new_tiles[*ay][*ax] {
                        tile if tile.can_support_plants() => {
                            // Water soaks into the earth, reducing water depth
                            let absorption_amount = match depth {
                                0..=30 => depth, // Light water completely absorbed
                                31..=50 => 20 + rng.gen_range(0..15), // Partial absorption
                                _ => 10 + rng.gen_range(0..20), // Heavy water partially absorbed
                            };
                            
                            let remaining_depth = depth.saturating_sub(absorption_amount);
                            if remaining_depth > 10 {
                                new_tiles[y][x] = TileType::Water(remaining_depth);
                            } else {
                                new_tiles[y][x] = TileType::Empty; // Water fully absorbed
                            }
                            return; // Water absorbed, skip other physics
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Calculate evaporation based on depth, biome, and environmental conditions
        let base_evaporation = match depth {
            0..=30 => 0.08,   // Small droplets evaporate quickly
            31..=80 => 0.02,  // Normal water evaporation rate
            81..=150 => 0.01, // Deep water evaporates slowly
            _ => 0.005,       // Very deep water barely evaporates
        };
        
        let day_modifier = if self.is_day() { 1.5 } else { 0.8 };
        let temp_modifier = (self.temperature + 1.0) * 0.5; // 0.0 to 1.0 range
        let biome_modifier = 2.0 - moisture_retention; // 0.6 to 1.4 range
        let final_evaporation = base_evaporation * day_modifier * (0.5 + temp_modifier) * biome_modifier;
        
        // Small chance of evaporation, higher for shallow water
        if rng.gen_bool(final_evaporation.min(1.0) as f64) {
            if depth <= 30 {
                new_tiles[y][x] = TileType::Empty; // Complete evaporation
            } else {
                // Partial evaporation - reduce depth
                let new_depth = depth.saturating_sub(10 + rng.gen_range(0..10));
                if new_depth > 0 {
                    new_tiles[y][x] = TileType::Water(new_depth);
                } else {
                    new_tiles[y][x] = TileType::Empty;
                }
            }
            return;
        }
        
        // Enhanced flow physics with depth-based pressure
        if y + 1 < self.tiles.len() {
            let below = new_tiles[y + 1][x];
            
            match below {
                TileType::Empty => {
                    // Water falls with momentum - deeper water falls faster and harder
                    let fall_depth = if depth <= 50 { depth } else { depth.saturating_add(10) }; // Deep water gains momentum
                    new_tiles[y][x] = TileType::Empty;
                    new_tiles[y + 1][x] = TileType::Water(fall_depth.min(255));
                    return;
                }
                TileType::Water(below_depth) => {
                    // Water combines with water below, creating pressure
                    let combined_depth = below_depth.saturating_add(depth / 3); // Some water flows down
                    if combined_depth != below_depth {
                        let flow_amount = combined_depth - below_depth;
                        let remaining_depth = depth.saturating_sub(flow_amount);
                        new_tiles[y + 1][x] = TileType::Water(combined_depth.min(255));
                        if remaining_depth > 20 {
                            new_tiles[y][x] = TileType::Water(remaining_depth);
                        } else {
                            new_tiles[y][x] = TileType::Empty;
                        }
                    }
                }
                _ => {} // Blocked by solid material
            }
        }
        
        // Horizontal flow with pressure-driven mechanics
        let flow_pressure = depth as f32 / 255.0;
        let flow_chance = flow_pressure * 0.8; // Deeper water flows more readily
        
        // In wetlands, reduce flow to encourage pooling
        let biome_flow_resistance = match biome {
            Biome::Wetland => 0.3,   // Strong resistance to encourage pooling
            Biome::Woodland => 0.6,  // Some resistance under tree cover
            Biome::Grassland => 0.8, // Normal flow
            Biome::Drylands => 1.0,  // Flows away quickly
        };
        
        if rng.gen_bool((flow_chance * biome_flow_resistance) as f64) {
            // Find the best flow direction using elevation and existing water levels
            let mut flow_targets = Vec::new();
            
            // Check all adjacent positions for flow potential
            let directions = [(-1, 0), (1, 0), (-1, 1), (1, 1)]; // Horizontal and diagonal-down
            
            for (dx, dy) in directions.iter() {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < new_tiles.len() as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    let target_tile = new_tiles[ny][nx];
                    if target_tile.can_water_flow_into() {
                        let flow_priority = if *dy == 1 { 3 } else { 2 }; // Prefer diagonal flow downward
                        flow_targets.push((nx, ny, flow_priority, 0u8));
                    } else if let Some(target_depth) = target_tile.get_water_depth() {
                        // Flow into areas with lower water level
                        if target_depth < depth.saturating_sub(20) {
                            let flow_priority = if *dy == 1 { 2 } else { 1 }; // Lower priority than empty space
                            flow_targets.push((nx, ny, flow_priority, target_depth));
                        }
                    }
                }
            }
            
            // Sort by flow priority (higher priority first)
            flow_targets.sort_by_key(|&(_, _, priority, _)| std::cmp::Reverse(priority));
            
            if let Some((target_x, target_y, _, target_depth)) = flow_targets.first() {
                let flow_amount = if depth > 100 {
                    depth / 3 // Deep water flows more aggressively
                } else if depth > 50 {
                    depth / 4
                } else {
                    depth / 5 // Shallow water flows conservatively
                }.max(10);
                
                let remaining_depth = depth.saturating_sub(flow_amount);
                let new_target_depth = target_depth.saturating_add(flow_amount);
                
                // Update target position
                new_tiles[*target_y][*target_x] = TileType::Water(new_target_depth.min(255));
                
                // Update current position
                if remaining_depth > 10 {
                    new_tiles[y][x] = TileType::Water(remaining_depth);
                } else {
                    new_tiles[y][x] = TileType::Empty;
                }
            }
        }
    }
    
    /// Process wind effects on seeds, spores, light particles, and water droplets
    fn process_wind_effects(&mut self) {
        if self.wind_strength < 0.1 {
            return; // No significant wind
        }
        
        let mut new_tiles = self.tiles.clone();
        let mut rng = rand::thread_rng();
        
        // Calculate wind direction components
        let wind_x = self.wind_direction.cos();
        let wind_y = self.wind_direction.sin();
        
        // Process from top to bottom, left to right for consistent wind direction
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    tile if tile.is_wind_dispersible() || tile.is_light_particle() => {
                        self.process_wind_particle(x, y, tile, &mut new_tiles, &mut rng, wind_x, wind_y);
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    /// Process individual particle movement due to wind
    fn process_wind_particle(&self, x: usize, y: usize, particle: TileType, 
                           new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng, 
                           wind_x: f32, wind_y: f32) {
        // Check if this particle should be affected by wind
        let wind_susceptibility = match particle {
            TileType::Seed(_, Size::Small) => 0.9,    // Small seeds very susceptible
            TileType::Seed(_, Size::Medium) => 0.6,   // Medium seeds moderately susceptible
            TileType::Seed(_, Size::Large) => 0.3,    // Large seeds less susceptible
            TileType::Spore(_) => 1.0,                // Spores very light
            TileType::Nutrient => 0.4,                // Nutrients moderately affected
            TileType::Water(depth) if depth <= 30 => (30 - depth) as f32 / 30.0, // Light water droplets
            _ => return, // Not wind-affected
        };
        
        // Calculate movement probability based on wind strength and susceptibility
        let movement_chance = self.wind_strength * wind_susceptibility * 0.8;
        
        if !rng.gen_bool(movement_chance as f64) {
            return; // No movement this tick
        }
        
        // Calculate target position based on wind direction
        // Add some randomness to make wind dispersal more natural
        let random_x = rng.gen_range(-0.3..0.3);
        let random_y = rng.gen_range(-0.3..0.3);
        
        let target_x = x as f32 + wind_x * self.wind_strength * 2.0 + random_x;
        let target_y = y as f32 + wind_y * self.wind_strength * 2.0 + random_y;
        
        // Clamp to world bounds
        let target_x = target_x.round() as i32;
        let target_y = target_y.round() as i32;
        
        if target_x < 0 || target_x >= self.width as i32 || 
           target_y < 0 || target_y >= self.height as i32 {
            // Particle blown out of world - remove it
            new_tiles[y][x] = TileType::Empty;
            return;
        }
        
        let target_x = target_x as usize;
        let target_y = target_y as usize;
        
        // Check if target position is available
        match new_tiles[target_y][target_x] {
            TileType::Empty => {
                // Move particle to new location
                new_tiles[y][x] = TileType::Empty;
                new_tiles[target_y][target_x] = particle;
            }
            target_tile if target_tile.is_water() => {
                if let Some(depth) = target_tile.get_water_depth() {
                    if depth <= 50 {
                        // Light water can be displaced by wind particles
                        if particle.is_light_particle() {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[target_y][target_x] = particle;
                            
                            // Try to move the displaced water to adjacent positions
                            self.try_displace_water(target_x, target_y, target_tile, new_tiles, rng);
                        }
                    }
                }
            }
            _ => {
                // Target blocked, try adjacent positions
                let adjacent_positions = [
                    (target_x.saturating_sub(1), target_y),
                    (target_x.saturating_add(1).min(self.width - 1), target_y),
                    (target_x, target_y.saturating_sub(1)),
                    (target_x, target_y.saturating_add(1).min(self.height - 1)),
                ];
                
                for (adj_x, adj_y) in adjacent_positions.iter() {
                    if new_tiles[*adj_y][*adj_x] == TileType::Empty {
                        new_tiles[y][x] = TileType::Empty;
                        new_tiles[*adj_y][*adj_x] = particle;
                        return;
                    }
                }
                // No adjacent space available - particle stays put
            }
        }
    }
    
    /// Helper function to try displacing water when wind particles collide
    fn try_displace_water(&self, x: usize, y: usize, water: TileType, 
                         new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let directions = [(0, 1), (-1, 0), (1, 0), (0, -1)]; // Down, left, right, up priority
        
        if let Some((dx, dy)) = directions.iter().choose(rng) {
            let new_x = (x as i32 + dx) as usize;
            let new_y = (y as i32 + dy) as usize;
            
            if new_x < self.width && new_y < self.height && new_tiles[new_y][new_x] == TileType::Empty {
                new_tiles[new_y][new_x] = water;
                return;
            }
        }
        // If no space found, water evaporates due to wind dispersal
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
        // Nutrients spread slowly - optimized to avoid full array clone
        let mut rng = rand::thread_rng();
        
        // Collect nutrient positions first to avoid iterator conflicts
        let mut nutrient_positions = Vec::new();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.tiles[y][x] == TileType::Nutrient {
                    nutrient_positions.push((x, y));
                }
            }
        }
        
        // Process diffusion using change queue
        for (x, y) in nutrient_positions {
            if rng.gen_bool(0.1) {
                let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < self.width && ny < self.height {
                        match self.tiles[ny][nx] {
                            TileType::Empty => {
                                // Normal diffusion to empty space
                                self.queue_tile_change(x, y, TileType::Empty);
                                self.queue_tile_change(nx, ny, TileType::Nutrient);
                            }
                            TileType::Dirt if rng.gen_bool(0.3) => {
                                // Nutrients can absorb into dirt, creating nutrient dirt
                                self.queue_tile_change(x, y, TileType::Empty);
                                self.queue_tile_change(nx, ny, TileType::NutrientDirt(80)); // Medium nutrient level
                            }
                            TileType::NutrientDirt(existing_level) if rng.gen_bool(0.2) => {
                                // Add more nutrients to existing nutrient dirt
                                let new_level = existing_level.saturating_add(30);
                                self.queue_tile_change(x, y, TileType::Empty);
                                self.queue_tile_change(nx, ny, TileType::NutrientDirt(new_level));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Apply all changes at once
        self.apply_tile_changes();
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
                        let mut new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        
                        // Check for adjacent nutrients to absorb (extends life)
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height && rng.gen_bool(0.1) {
                                    if self.tiles[ny][nx] == TileType::Nutrient {
                                        new_tiles[ny][nx] = TileType::Empty;
                                        new_age = new_age.saturating_sub(15); // Absorbing nutrients extends life
                                        break;
                                    }
                                }
                            }
                        }
                        
                        if new_age > (100.0 * size.lifespan_multiplier()) as u8 {
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantStem(new_age, size);
                            
                            // Plant growth - affected by seasonal conditions and biome
                            let biome = self.get_biome_at(x, y);
                            let seasonal_growth_rate = self.get_seasonal_growth_modifier() 
                                * growth_rate 
                                * biome.plant_growth_modifier();
                            if rng.gen_bool((0.1 * seasonal_growth_rate).min(1.0) as f64) {
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
                        
                        let biome = self.get_biome_at(x, y);
                        let seasonal_growth_rate = self.get_seasonal_growth_modifier() 
                            * growth_rate 
                            * biome.plant_growth_modifier();
                        if new_age > 25 && rng.gen_bool((0.15 * seasonal_growth_rate).min(1.0) as f64) {
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
                            let biome = self.get_biome_at(x, y);
                            let seasonal_growth_rate = self.get_seasonal_growth_modifier() 
                                * growth_rate 
                                * biome.plant_growth_modifier();
                            if rng.gen_bool((0.08 * seasonal_growth_rate).min(1.0) as f64) {
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
                            
                            // Flowers produce seeds that can be dispersed by wind
                            let biome = self.get_biome_at(x, y);
                            let seasonal_growth_rate = self.get_seasonal_growth_modifier() 
                                * size.growth_rate_multiplier() 
                                * biome.plant_growth_modifier();
                            
                            // Higher chance during windy conditions for natural dispersal
                            let wind_boost = 1.0 + (self.wind_strength * 2.0);
                            let seed_chance = (0.08 * seasonal_growth_rate * wind_boost).min(1.0);
                            
                            if rng.gen_bool(seed_chance as f64) {
                                // Shoot seed with velocity instead of placing nearby
                                let seed_size = if rng.gen_bool(0.7) { size } else { random_size(&mut rng) };
                                
                                // Calculate shooting direction and velocity
                                let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                                
                                // Base velocity depends on flower size and wind
                                let base_velocity = match size {
                                    Size::Small => 1.5 + rng.gen_range(0.0..1.0),
                                    Size::Medium => 2.0 + rng.gen_range(0.0..1.5),
                                    Size::Large => 2.5 + rng.gen_range(0.0..2.0),
                                };
                                
                                // Wind can boost seed shooting velocity
                                let wind_boost = 1.0 + (self.wind_strength * 0.5);
                                let velocity = base_velocity * wind_boost;
                                
                                // Prefer upward/outward directions for better dispersal
                                let upward_bias = rng.gen_range(-0.5..0.0); // Slight upward bias
                                
                                let velocity_x = angle.cos() * velocity;
                                let velocity_y = (angle.sin() * velocity) + upward_bias;
                                
                                // Create seed projectile
                                let seed_projectile = SeedProjectile {
                                    x: x as f32 + 0.5, // Center of flower tile
                                    y: y as f32 + 0.5,
                                    velocity_x,
                                    velocity_y,
                                    seed_type: TileType::Seed(0, seed_size),
                                    age: 0,
                                    bounce_count: 0,
                                };
                                
                                self.seed_projectiles.push(seed_projectile);
                            }
                        }
                    }
                    TileType::PlantWithered(age, size) => {
                        let new_age = age.saturating_add(2);
                        if new_age > 30 {
                            new_tiles[y][x] = TileType::Nutrient;
                            
                            // Sometimes generate spores from decaying organic matter
                            if rng.gen_bool(0.1) && self.wind_strength > 0.2 {
                                // Try to place spore in nearby empty space
                                let spore_positions = [
                                    (x.saturating_sub(1), y), (x.saturating_add(1), y),
                                    (x, y.saturating_sub(1)), (x, y.saturating_add(1)),
                                ];
                                
                                if let Some((sx, sy)) = spore_positions.iter().choose(&mut rng) {
                                    if *sx < self.width && *sy < self.height && new_tiles[*sy][*sx] == TileType::Empty {
                                        new_tiles[*sy][*sx] = TileType::Spore(0);
                                    }
                                }
                            }
                        } else {
                            new_tiles[y][x] = TileType::PlantWithered(new_age, size);
                        }
                    }
                    TileType::PlantDiseased(age, size) => {
                        let new_age = age.saturating_add(1);
                        
                        if new_age > 60 {
                            // Disease kills the plant, turning it into withered plant
                            new_tiles[y][x] = TileType::PlantWithered(0, size);
                        } else {
                            new_tiles[y][x] = TileType::PlantDiseased(new_age, size);
                            
                            // Diseased plants actively spread spores when windy
                            if new_age > 10 && rng.gen_bool((0.05 + self.wind_strength * 0.1) as f64) {
                                // Generate spores that spread disease
                                let spore_positions = [
                                    (x.saturating_sub(1), y), (x.saturating_add(1), y),
                                    (x, y.saturating_sub(1)), (x, y.saturating_add(1)),
                                    (x.saturating_sub(1), y.saturating_sub(1)), (x.saturating_add(1), y.saturating_sub(1)),
                                ];
                                
                                if let Some((sx, sy)) = spore_positions.iter().choose(&mut rng) {
                                    if *sx < self.width && *sy < self.height && new_tiles[*sy][*sx] == TileType::Empty {
                                        new_tiles[*sy][*sx] = TileType::Spore(0);
                                    }
                                }
                            }
                            
                            // Disease spreads to nearby healthy plants
                            let spread_chance = 0.02 * (1.0 + new_age as f32 / 60.0); // Higher chance as disease progresses
                            for dy in -1i32..=1 {
                                for dx in -1i32..=1 {
                                    if dx == 0 && dy == 0 { continue; }
                                    
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    
                                    if nx < self.width && ny < self.height && rng.gen_bool(spread_chance as f64) {
                                        // Disease can infect healthy plant parts
                                        match self.tiles[ny][nx] {
                                            TileType::PlantLeaf(_leaf_age, leaf_size) |
                                            TileType::PlantBud(_leaf_age, leaf_size) |
                                            TileType::PlantBranch(_leaf_age, leaf_size) |
                                            TileType::PlantFlower(_leaf_age, leaf_size) => {
                                                new_tiles[ny][nx] = TileType::PlantDiseased(0, leaf_size);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    TileType::PlantRoot(age, size) => {
                        let mut new_age = age.saturating_add(1);
                        let growth_rate = size.growth_rate_multiplier();
                        let mut nutrients_absorbed = 0u8;
                        
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
                                    match self.tiles[ny][nx] {
                                        TileType::Nutrient if rng.gen_bool((0.3 * growth_rate).min(1.0) as f64) => {
                                            // Absorb free nutrients
                                            new_tiles[ny][nx] = TileType::Empty;
                                            nutrients_absorbed = nutrients_absorbed.saturating_add(20);
                                            
                                            // Chance to grow new root toward absorbed nutrient
                                            if rng.gen_bool(0.4) {
                                                let steps_x = if dx > 0 { 1 } else if dx < 0 { -1 } else { 0 };
                                                let steps_y = if dy > 0 { 1 } else if dy < 0 { -1 } else { 0 };
                                                let extend_x = (x as i32 + steps_x) as usize;
                                                let extend_y = (y as i32 + steps_y) as usize;
                                                
                                                if extend_x < self.width && extend_y < self.height 
                                                    && matches!(new_tiles[extend_y][extend_x], TileType::Empty) 
                                                    && new_tiles[extend_y][extend_x].can_support_plants() {
                                                    new_tiles[extend_y][extend_x] = TileType::PlantRoot(0, size);
                                                }
                                            }
                                        },
                                        TileType::NutrientDirt(nutrient_level) if rng.gen_bool((0.2 * growth_rate).min(1.0) as f64) => {
                                            // Absorb nutrients from nutrient-rich dirt
                                            let absorbed = (nutrient_level / 4).max(10); // Extract some nutrients
                                            let remaining = nutrient_level.saturating_sub(absorbed);
                                            nutrients_absorbed = nutrients_absorbed.saturating_add(absorbed);
                                            
                                            if remaining < 20 {
                                                // Nutrient dirt becomes regular dirt
                                                new_tiles[ny][nx] = TileType::Dirt;
                                            } else {
                                                new_tiles[ny][nx] = TileType::NutrientDirt(remaining);
                                            }
                                        },
                                        TileType::Dirt if rng.gen_bool(0.05) => {
                                            // Roots can merge with regular dirt, creating nutrient dirt
                                            new_tiles[ny][nx] = TileType::NutrientDirt(40); // Small amount of nutrients
                                            
                                            // Root extends into the dirt
                                            if rng.gen_bool(0.3) {
                                                new_tiles[ny][nx] = TileType::PlantRoot(0, size);
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                        
                        // Nutrients absorbed delay aging (reset some age)
                        if nutrients_absorbed > 0 {
                            let age_reduction = (nutrients_absorbed as f32 * 0.3) as u8; 
                            new_age = new_age.saturating_sub(age_reduction);
                        }
                        
                        if new_age > (200.0 * size.lifespan_multiplier()) as u8 {
                            // Old roots wither and become nutrients
                            new_tiles[y][x] = TileType::Nutrient;
                        } else {
                            new_tiles[y][x] = TileType::PlantRoot(new_age, size);
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
                                        TileType::PlantLeaf(_, food_size) | TileType::PlantWithered(_, food_size) | TileType::PlantDiseased(_, food_size) => {
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
                        if well_fed && age > 30 && age < 100 && rng.gen_bool((0.05 * size.growth_rate_multiplier()).min(1.0) as f64) {
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
            // Baby pillbugs grow body segments as they mature, but only if they're stable (not falling)
            let connected_segments = self.find_connected_pillbug_segments(x, y);
            let is_falling = self.is_pillbug_group_unsupported(&connected_segments);
            
            if !is_falling {
                if age == 10 {
                    // Grow body segment only if stable
                    for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                            new_tiles[ny][nx] = TileType::PillbugBody(age, size);
                            break;
                        }
                    }
                } else if age == 20 {
                    // Grow legs segment only if stable
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
        
        // Process seed aging, germination, and spore lifecycle
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Seed(age, size) => {
                        let new_age = age.saturating_add(1);
                        if new_age > 100 {
                            // Old seeds decay into nutrients
                            new_tiles[y][x] = TileType::Nutrient;
                        } else {
                            new_tiles[y][x] = TileType::Seed(new_age, size);
                            
                            // Seeds can germinate under good conditions
                            let biome = self.get_biome_at(x, y);
                            let seasonal_growth_rate = self.get_seasonal_growth_modifier() 
                                * size.growth_rate_multiplier() 
                                * biome.plant_growth_modifier();
                            
                            // Germination requires stable conditions (not too windy, good moisture)
                            let wind_penalty = 1.0 - (self.wind_strength * 0.5);
                            let germination_chance = (0.03 * seasonal_growth_rate * wind_penalty).min(1.0);
                            
                            if rng.gen_bool(germination_chance as f64) {
                                // Check if there's soil below for rooting
                                if y + 1 < self.height && matches!(new_tiles[y + 1][x], TileType::Dirt | TileType::Sand) {
                                    new_tiles[y][x] = TileType::PlantStem(0, size);
                                    // Add initial root
                                    if rng.gen_bool(0.7) {
                                        new_tiles[y + 1][x] = TileType::PlantRoot(0, size);
                                    }
                                }
                            }
                        }
                    }
                    TileType::Spore(age) => {
                        let new_age = age.saturating_add(1);
                        if new_age > 50 {
                            // Spores fade away
                            new_tiles[y][x] = TileType::Empty;
                        } else {
                            new_tiles[y][x] = TileType::Spore(new_age);
                            
                            // Spores can occasionally cause plant disease
                            if new_age > 20 && rng.gen_bool(0.02) {
                                // Look for nearby plants to infect
                                for dy in -1..=1 {
                                    for dx in -1..=1 {
                                        let nx = (x as i32 + dx) as usize;
                                        let ny = (y as i32 + dy) as usize;
                                        if nx < self.width && ny < self.height {
                                            if let TileType::PlantLeaf(plant_age, plant_size) 
                                            | TileType::PlantStem(plant_age, plant_size) 
                                            | TileType::PlantBranch(plant_age, plant_size) 
                                            | TileType::PlantFlower(plant_age, plant_size) = new_tiles[ny][nx] {
                                                // Only infect weakened (older) plants
                                                if plant_age > 30 && rng.gen_bool(0.3) {
                                                    new_tiles[ny][nx] = TileType::PlantDiseased(0, plant_size);
                                                    new_tiles[y][x] = TileType::Empty; // Spore consumed
                                                    break;
                                                }
                                            }
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
                            TileType::PlantLeaf(_, _) | TileType::PlantWithered(_, _) | TileType::PlantDiseased(_, _) | TileType::Nutrient => {
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
                        tile if tile.is_water() => {
                            // Standing water is dangerous
                            if dy > 0 {  // Water below is especially dangerous
                                danger_positions.push((dx, dy));
                            }
                        },
                        _ => {
                            // Check for unstable areas (floating sand)
                            if matches!(tile, TileType::Sand) {
                                // Check if sand has support
                                if ny + 1 < self.height && (self.tiles[ny + 1][nx] == TileType::Empty || self.tiles[ny + 1][nx].is_water()) {
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
        
        // Randomly introduce plant diseases (very rare)
        // Disease introduction is more likely in humid conditions and during certain seasons
        let base_disease_chance = 0.0005; // Realistic but observable disease chance
        let seasonal_disease_modifier = match self.get_current_season() {
            Season::Summer => 1.5,  // Hot humid summers increase disease risk
            Season::Fall => 1.2,    // Wet fall conditions favor disease
            Season::Winter => 0.3,  // Cold reduces most plant diseases  
            Season::Spring => 1.0,  // Normal disease pressure
        };
        let humidity_modifier = 1.0 + self.humidity; // Higher humidity increases disease risk
        let disease_chance = base_disease_chance * seasonal_disease_modifier * humidity_modifier;
        
        if rng.gen_bool(disease_chance as f64) {
            // Find a random healthy plant part to infect
            let mut attempts = 0;
            while attempts < 50 {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);
                
                match self.tiles[y][x] {
                    TileType::PlantLeaf(_age, size) |
                    TileType::PlantBud(_age, size) |
                    TileType::PlantBranch(_age, size) |
                    TileType::PlantFlower(_age, size) => {
                        // Introduce disease to this plant part
                        self.tiles[y][x] = TileType::PlantDiseased(0, size);
                        break;
                    }
                    _ => {}
                }
                attempts += 1;
            }
        }
    }
    
    // Calculate ecosystem statistics for monitoring
    pub fn calculate_ecosystem_stats(&self) -> EcosystemStats {
        let mut stats = EcosystemStats {
            total_plants: 0,
            total_pillbugs: 0,
            water_coverage: 0,
            nutrient_count: 0,
            plant_health_ratio: 0.0,
            biome_diversity: 0,
        };
        
        let mut healthy_plants = 0;
        let mut _diseased_plants = 0;
        let mut biome_types = HashSet::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    // Count plant parts
                    TileType::PlantStem(_, _) | TileType::PlantLeaf(_, _) | 
                    TileType::PlantBud(_, _) | TileType::PlantBranch(_, _) | 
                    TileType::PlantFlower(_, _) | TileType::PlantRoot(_, _) => {
                        stats.total_plants += 1;
                        healthy_plants += 1;
                    },
                    TileType::PlantWithered(_, _) | TileType::PlantDiseased(_, _) => {
                        stats.total_plants += 1;
                        _diseased_plants += 1;
                    },
                    
                    // Count pillbug parts
                    TileType::PillbugHead(_, _) | TileType::PillbugBody(_, _) | 
                    TileType::PillbugLegs(_, _) | TileType::PillbugDecaying(_, _) => {
                        stats.total_pillbugs += 1;
                    },
                    
                    // Count environmental elements
                    TileType::Water(_) => stats.water_coverage += 1,
                    TileType::Nutrient => stats.nutrient_count += 1,
                    
                    _ => {},
                }
                
                // Track biome diversity
                biome_types.insert(std::mem::discriminant(&self.biome_map[y][x]));
            }
        }
        
        // Calculate health ratio
        if stats.total_plants > 0 {
            stats.plant_health_ratio = healthy_plants as f32 / stats.total_plants as f32;
        }
        
        stats.biome_diversity = biome_types.len();
        stats
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
        writeln!(f, "Season: {} | Temperature: {:.1} | Humidity: {:.1}", 
                 self.get_season_name(), self.temperature, self.humidity)?;
        writeln!(f, "Rain intensity: {:.2} | Wind: {:.1} @ {:.0}°", 
                 self.rain_intensity, self.wind_strength, 
                 self.wind_direction * 180.0 / std::f32::consts::PI)?;
        
        // Add ecosystem statistics
        let stats = self.calculate_ecosystem_stats();
        writeln!(f, "Ecosystem: Plants:{} Pillbugs:{} Water:{} Nutrients:{}", 
                 stats.total_plants, stats.total_pillbugs, stats.water_coverage, stats.nutrient_count)?;
        writeln!(f, "Health:{:.1}% Biomes:{} ({}x{} world)", 
                 stats.plant_health_ratio * 100.0, stats.biome_diversity, self.width, self.height)?;
        Ok(())
    }
}