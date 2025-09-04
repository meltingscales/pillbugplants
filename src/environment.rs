use rand::Rng;
use crate::types::{Season, Biome};
use crate::world::World;

impl World {
    /// Update seasonal weather parameters - extracted from update_seasonal_weather 
    pub fn update_seasonal_conditions(&mut self) {
        // Calculate target temperature and humidity based on season
        let (target_temp, target_humidity) = match self.get_current_season() {
            Season::Spring => (0.3, 0.7),   // Mild and moist
            Season::Summer => (0.8, 0.3),   // Hot and dry
            Season::Fall => (0.1, 0.6),     // Cool and moderately moist
            Season::Winter => (-0.5, 0.4),  // Cold and dry
        };
        
        // Smooth transition to seasonal targets
        let temp_change = (target_temp - self.temperature) * 0.02;
        let humidity_change = (target_humidity - self.humidity) * 0.02;
        
        self.temperature += temp_change;
        self.humidity = (self.humidity + humidity_change).clamp(0.0, 1.0);
    }
    
    /// Spawn rain based on environmental conditions
    pub fn process_rain_cycle(&mut self, rng: &mut impl Rng) {
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
    }
    
    /// Calculate environmental growth modifier based on season, temperature, etc.
    pub fn get_environmental_growth_modifier(&self) -> f32 {
        let temp_modifier = if self.temperature > 0.0 { 
            1.0 + self.temperature * 0.5 
        } else { 
            (1.0 + self.temperature).max(0.1) // Cold slows growth
        };
        
        let humidity_modifier = 0.5 + self.humidity * 0.5;
        
        temp_modifier * humidity_modifier
    }
}