use rand::Rng;
use crate::types::{TileType, Size, random_size};
use crate::world::World;

impl World {
    /// Process plant aging and lifecycle - extracted from update_life
    pub fn update_plant_lifecycle(&mut self, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
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
                                * size.growth_rate_multiplier() 
                                * biome.plant_growth_modifier();
                            
                            let growth_chance: f32 = 0.15 * seasonal_growth_rate;
                            
                            // Vertical growth (stem extension)
                            if y > 0 && rng.gen_bool((growth_chance * 0.3).min(1.0) as f64) && new_tiles[y - 1][x] == TileType::Empty {
                                new_tiles[y - 1][x] = TileType::PlantStem(0, size);
                            }
                            
                            // Lateral growth (buds for leaves and flowers)
                            if rng.gen_bool((growth_chance * 0.4).min(1.0) as f64) {
                                let lateral_directions = [(x.saturating_sub(1), y), (x.saturating_add(1), y)];
                                if let Some((bx, by)) = lateral_directions.iter().find(|(bx, by)| {
                                    *bx < self.width && *by < self.height && new_tiles[*by][*bx] == TileType::Empty
                                }) {
                                    new_tiles[*by][*bx] = TileType::PlantBud(0, size);
                                }
                            }
                            
                            // Root growth downward to find nutrients
                            if y + 1 < self.height && rng.gen_bool((growth_chance * 0.6).min(1.0) as f64) {
                                if matches!(new_tiles[y + 1][x], TileType::Empty | TileType::Dirt | TileType::Sand) && rng.gen_bool(0.5) {
                                    new_tiles[y + 1][x] = TileType::PlantRoot(0, size);
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
                            
                            // Only photosynthesize during day
                            if self.is_day() && rng.gen_bool(0.3) {
                                for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                                        new_tiles[ny][nx] = TileType::Nutrient;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // More plant types would continue here...
                    _ => {}
                }
            }
        }
    }
}