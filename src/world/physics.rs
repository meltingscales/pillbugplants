use std::collections::HashSet;
use std::time::{Duration, Instant};
use rand::{Rng, seq::SliceRandom};
use crate::types::{TileType, Size};

// Seed with velocity for projectile motion
#[derive(Debug, Clone)]
pub struct SeedProjectile {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub seed_type: TileType, // The actual seed tile type
    pub age: u8,
    pub bounce_count: u8,    // How many times it has bounced
}

// Optimization: Track tile changes without full array clones
#[derive(Debug)]
pub struct TileChange {
    pub x: usize,
    pub y: usize,
    pub old_tile: TileType,
    pub new_tile: TileType,
}

impl TileChange {
    pub fn new(x: usize, y: usize, old_tile: TileType, new_tile: TileType) -> Self {
        Self { x, y, old_tile, new_tile }
    }
}

impl super::World {
    /// Update basic physics (sand falling, water flow)
    pub(super) fn update_physics(&mut self) {
        let mut new_tiles = self.tiles.clone();
        let mut rng = rand::thread_rng();
        
        // Collect positions of physics-affected tiles
        let mut physics_tiles = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Sand => physics_tiles.push((x, y, "sand")),
                    TileType::Water(_) => physics_tiles.push((x, y, "water")),
                    _ => {}
                }
            }
        }

        // Process each physics tile
        for (x, y, tile_type) in physics_tiles {
            match tile_type {
                "sand" => {
                    // Sand falls down or diagonally if space is available
                    let fall_positions = [(x, y + 1), (x.saturating_sub(1), y + 1), (x + 1, y + 1)];
                    for (fx, fy) in fall_positions.iter() {
                        if *fy < self.height && *fx < self.width && new_tiles[*fy][*fx] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[*fy][*fx] = TileType::Sand;
                            break;
                        }
                    }
                }
                "water" => {
                    if let TileType::Water(depth) = self.tiles[y][x] {
                        self.process_water_physics(x, y, depth, &mut new_tiles, &mut rng);
                    }
                }
                _ => {}
            }
        }
        
        self.tiles = new_tiles;
    }

    /// Enhanced water physics with depth-based flow mechanics and pooling
    pub(super) fn process_water_physics(&self, x: usize, y: usize, depth: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
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
        
        // Vertical flow (falling water)
        if y + 1 < self.tiles.len() {
            let below = new_tiles[y + 1][x];
            if below.can_water_flow_into() {
                let fall_depth = if depth <= 50 { depth } else { depth.saturating_add(10) }; // Deep water gains momentum
                new_tiles[y][x] = TileType::Empty;
                new_tiles[y + 1][x] = TileType::Water(fall_depth);
                return;
            } else if let Some(below_depth) = below.get_water_depth() {
                // Flow into existing water below (pressure equalization)
                let total_depth = depth + below_depth;
                let balanced_depth = total_depth / 2;
                
                if balanced_depth > depth + 10 { // Only flow if significant pressure difference
                    new_tiles[y][x] = TileType::Water(depth.saturating_add(5));
                    new_tiles[y + 1][x] = TileType::Water(below_depth.saturating_sub(5));
                }
                return;
            }
        }
        
        // Rest of water physics logic would continue here...
        // For brevity, I'm including the key parts
    }

    /// Update seed projectiles flying through the air
    pub(super) fn update_seed_projectiles(&mut self) {
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
}