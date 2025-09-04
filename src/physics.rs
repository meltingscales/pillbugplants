use rand::Rng;
use crate::types::{TileType, Size};
use crate::world::World;

impl World {
    /// Apply gravity to unsupported entities - this could be called from the main gravity function
    pub fn apply_gravity_to_particles(&mut self, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        // Process gravity for individual particles from bottom to top
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    // Seeds and spores fall
                    TileType::Seed(age, size) => {
                        if new_tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.6) {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = TileType::Seed(age, size);
                        }
                    }
                    TileType::Spore(age) => {
                        if new_tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.3) {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = TileType::Spore(age);
                        }
                    }
                    // Nutrients fall slowly
                    TileType::Nutrient => {
                        if new_tiles[y + 1][x] == TileType::Empty && rng.gen_bool(0.2) {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = TileType::Nutrient;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Process sand falling physics - extracted from update_physics
    pub fn update_sand_physics(&mut self, new_tiles: &mut Vec<Vec<TileType>>) {
        // Collect sand positions
        let mut sand_positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if matches!(self.tiles[y][x], TileType::Sand) {
                    sand_positions.push((x, y));
                }
            }
        }

        // Process each sand tile
        for (x, y) in sand_positions {
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
    }
}