pub mod physics;

use std::collections::HashSet;
use std::time::{Duration, Instant};
use rand::{Rng, seq::SliceRandom, prelude::IteratorRandom};

use crate::types::{TileType, Size, random_size, MovementStrategy, Season, Biome, random_biome};
pub use physics::{SeedProjectile, TileChange};

// Re-export the main World struct and related types
pub use world_impl::*;

mod world_impl {
    use super::*;

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

    pub struct World {
        pub tiles: Vec<Vec<TileType>>,
        pub biome_map: Vec<Vec<Biome>>, // Biome information for each region
        pub width: usize,
        pub height: usize,
        pub tick: usize,
        pub day_cycle: f32,        // 0.0 to 2π
        pub rain_intensity: f32,   // 0.0 to 1.0
        pub season_cycle: f32,     // 0.0 to 1.0, represents progression through year
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
            let mut world = World {
                tiles: vec![vec![TileType::Empty; width]; height],
                biome_map: vec![vec![Biome::Grassland; width]; height],
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
            
            // Generate initial world will be moved to generation module
            world.generate_biome_map();
            world.generate_initial_world();
            world
        }

        // Placeholder methods - these will be moved to appropriate modules
        fn generate_biome_map(&mut self) {
            // Temporary stub - will be moved to generation module
        }
        
        fn generate_initial_world(&mut self) {
            // Temporary stub - will be moved to generation module  
        }

        pub fn update(&mut self) {
            // This is where we'll call all the modularized systems
            // The actual implementation will remain in world.rs for now
            // but will gradually be moved to modules
        }

        pub fn is_day(&self) -> bool {
            self.day_cycle.sin() > 0.0
        }
        
        pub fn get_projectile_count(&self) -> usize {
            self.seed_projectiles.len()
        }
        
        // More methods will be added as we continue modularization...
    }
}