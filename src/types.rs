use rand::Rng;
use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum Season {
    Spring = 0, // Growth season - mild temperature, high humidity
    Summer = 1, // Hot season - high temperature, low humidity
    Fall = 2,   // Harvest season - moderate temperature, increasing humidity
    Winter = 3, // Cold season - low temperature, variable humidity
}

#[derive(Clone, Copy, PartialEq)]
pub enum Biome {
    Wetland,    // High moisture retention, frequent pools, lush plant growth
    Grassland,  // Balanced moisture, moderate plant density
    Drylands,   // Low moisture retention, sparse vegetation, sandy soil
    Woodland,   // Dense plant growth, high nutrient content, mixed terrain
}

#[derive(Debug, Clone)]
pub enum MovementStrategy {
    SeekFood((i32, i32)),    // Direction to food
    Social((i32, i32)),      // Direction to other pillbugs
    Avoid((i32, i32)),       // Direction away from danger
    Explore,                 // Random exploration
    Rest,                    // Stay put or minimal movement
}

impl MovementStrategy {
    pub fn get_movement_vector(&self, rng: &mut impl Rng) -> (i32, i32) {
        match self {
            MovementStrategy::SeekFood(direction) => *direction,
            MovementStrategy::Social(direction) => *direction,
            MovementStrategy::Avoid(direction) => *direction,
            MovementStrategy::Explore => {
                let moves = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                *moves.get(rng.gen_range(0..4)).unwrap()
            },
            MovementStrategy::Rest => (0, 0),
        }
    }
    
    pub fn should_move(&self, rng: &mut impl Rng) -> bool {
        match self {
            MovementStrategy::SeekFood(_) => rng.gen_bool(0.8), // High urgency for food
            MovementStrategy::Social(_) => rng.gen_bool(0.4),   // Moderate social movement
            MovementStrategy::Avoid(_) => rng.gen_bool(0.9),    // Very high urgency to avoid
            MovementStrategy::Explore => rng.gen_bool(0.3),     // Casual exploration
            MovementStrategy::Rest => rng.gen_bool(0.1),        // Very low movement when resting
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Small = 0,   // Faster growth, shorter life, weaker
    Medium = 1,  // Normal values  
    Large = 2,   // Slower growth, longer life, stronger
}

impl Size {
    pub fn lifespan_multiplier(self) -> f32 {
        match self {
            Size::Small => 5.6,   // 30% shorter life (8x base multiplier)
            Size::Medium => 8.0,  // Normal lifespan (8x base multiplier)
            Size::Large => 11.2,  // 40% longer life (8x base multiplier)
        }
    }
    
    pub fn growth_rate_multiplier(self) -> f32 {
        match self {
            Size::Small => 1.3,   // 30% faster growth/reproduction
            Size::Medium => 1.0,  // Normal rate
            Size::Large => 0.8,   // 20% slower growth/reproduction
        }
    }
    
    pub fn to_char_modifier(self, base_char: char) -> char {
        match (self, base_char) {
            (Size::Small, '|') => 'i',    // Small stem
            (Size::Small, 'L') => 'l',    // Small leaf
            (Size::Small, 'o') => '°',    // Small bud
            (Size::Small, '/') => '\\',   // Small branch
            (Size::Small, '*') => '·',    // Small flower
            (Size::Small, '@') => 'ó',    // Small head
            (Size::Small, 'O') => 'o',    // Small body
            (Size::Small, 'w') => 'v',    // Small legs
            (Size::Small, 'r') => '·',    // Small root
            (Size::Small, '?') => '¿',    // Small diseased
            (Size::Large, '|') => '║',    // Large stem
            (Size::Large, 'L') => 'Ł',    // Large leaf
            (Size::Large, 'o') => 'O',    // Large bud
            (Size::Large, '/') => '╱',    // Large branch
            (Size::Large, '*') => '✱',    // Large flower
            (Size::Large, '@') => '●',    // Large head
            (Size::Large, 'O') => '●',    // Large body
            (Size::Large, 'w') => 'W',    // Large legs
            (Size::Large, 'r') => 'R',    // Large root
            (Size::Large, '?') => '‽',    // Large diseased
            _ => base_char, // Medium size keeps original char
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Empty,
    Dirt,
    Sand,
    Water(u8),        // Water with depth/pressure (0-255), affects flow behavior
    PlantStem(u8, Size),   // Main structural support, age 0-255 (dies at ~100*lifespan_8x), size
    PlantLeaf(u8, Size),   // Photosynthesis organs, age 0-255 (dies at ~50*lifespan_8x), size
    PlantBud(u8, Size),    // Growth points that become branches/flowers, age 0-255 (dies at 50), size
    PlantBranch(u8, Size), // Diagonal growth branches, age 0-255 (dies at ~100*lifespan_8x), size
    PlantFlower(u8, Size), // Reproductive organs, age 0-255 (dies at ~80*lifespan_8x), size
    PlantWithered(u8, Size), // Dying plant part, age 0-30 before becoming nutrient, size
    PlantDiseased(u8, Size), // Diseased plant part, spreads to nearby plants, age 0-60, size
    PlantRoot(u8, Size),     // Underground root system for nutrient absorption, age 0-255 (dies at ~200*lifespan_8x), size
    PillbugHead(u8, Size),    // Head segment of pillbug, age 0-255 (dies at ~150*lifespan_8x), size
    PillbugBody(u8, Size),    // Body segment of pillbug, age 0-255 (dies at ~150*lifespan_8x), size
    PillbugLegs(u8, Size),    // Leg segment of pillbug, age 0-255 (dies at ~150*lifespan_8x), size
    PillbugDecaying(u8, Size), // Dying pillbug part, age 0-20 before becoming nutrient, size
    Nutrient,
    Seed(u8, Size),           // Plant seed that can be dispersed by wind, age 0-255 (dies at 100), size
    Spore(u8),                // Fungal/bacterial spores, age 0-255 (dies at 50), carried by wind
}

impl TileType {
    pub fn to_char(self) -> char {
        match self {
            TileType::Empty => ' ',
            TileType::Dirt => '#',
            TileType::Sand => '.',
            TileType::Water(depth) => {
                match depth {
                    0..=50 => '·',     // Light water/droplets
                    51..=120 => '~',   // Normal water
                    121..=200 => '≈',  // Deep water
                    _ => '█',          // Very deep/pressurized water
                }
            },
            TileType::PlantStem(_, size) => size.to_char_modifier('|'),
            TileType::PlantLeaf(_, size) => size.to_char_modifier('L'),
            TileType::PlantBud(_, size) => size.to_char_modifier('o'),
            TileType::PlantBranch(_, size) => size.to_char_modifier('/'), // Diagonal branches
            TileType::PlantFlower(_, size) => size.to_char_modifier('*'),
            TileType::PlantWithered(_, size) => size.to_char_modifier('x'), // Withered plants
            TileType::PlantDiseased(_, size) => size.to_char_modifier('?'), // Diseased plants
            TileType::PlantRoot(_, size) => size.to_char_modifier('r'), // Underground roots
            TileType::PillbugHead(_, size) => size.to_char_modifier('@'),
            TileType::PillbugBody(_, size) => size.to_char_modifier('O'),
            TileType::PillbugLegs(_, size) => size.to_char_modifier('w'),
            TileType::PillbugDecaying(_, size) => size.to_char_modifier('░'), // Decaying pillbugs
            TileType::Nutrient => '+',
            TileType::Seed(_, size) => size.to_char_modifier('o'), // Seeds look like small buds
            TileType::Spore(_) => '∘', // Small spores
        }
    }
    
    pub fn to_color(self) -> Color {
        match self {
            TileType::Empty => Color::Black,
            TileType::Dirt => Color::Rgb(101, 67, 33),
            TileType::Sand => Color::Yellow,
            TileType::Water(depth) => {
                let _intensity = (depth as u16 * 255 / 255).min(255) as u8;
                match depth {
                    0..=50 => Color::Rgb(180, 220, 255),      // Light blue droplets
                    51..=120 => Color::Rgb(64, 164, 255),     // Normal blue water
                    121..=200 => Color::Rgb(0, 100, 200),     // Deep blue water
                    _ => Color::Rgb(0, 50, 150),              // Very deep dark blue
                }
            },
            TileType::PlantStem(age, size) => {
                let base_intensity = (255u16.saturating_sub(age as u16)).max(80) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,   // Slightly dimmer
                    Size::Medium => 1.0,   // Normal
                    Size::Large => 1.15,   // Slightly brighter
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity / 3, intensity, intensity / 4) // Brown-green stem
            },
            TileType::PlantLeaf(age, size) => {
                let base_intensity = (150u16.saturating_sub(age as u16)).max(60) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(0, intensity, 0) // Green leaves
            },
            TileType::PlantBud(age, size) => {
                let base_intensity = (50u16.saturating_sub(age as u16)).max(120) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 2, 0) // Orange-ish buds
            },
            TileType::PlantBranch(age, size) => {
                let base_intensity = (120u16.saturating_sub(age as u16)).max(70) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity / 4, intensity, intensity / 3) // Green-brown branches
            },
            TileType::PlantFlower(age, size) => {
                let fade = age as u16;
                let base_red = (255 - fade).max(100) as u8;
                let base_green = (200 - fade / 2).max(50) as u8;
                let base_blue = (255 - fade).max(100) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let red = (base_red as f32 * size_boost).min(255.0) as u8;
                let green = (base_green as f32 * size_boost).min(255.0) as u8;
                let blue = (base_blue as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(red, green, blue) // Pink-white flowers
            },
            TileType::PlantWithered(age, size) => {
                let decay_progress = age as f32 / 30.0; // 0.0 = fresh withered, 1.0 = almost nutrient
                let base_intensity = (100.0 * (1.0 - decay_progress * 0.6)) as u8; // Darken over time
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 2, 0) // Brown withered color
            },
            TileType::PlantDiseased(age, size) => {
                let disease_progress = age as f32 / 60.0; // 0.0 = fresh infection, 1.0 = full disease
                let base_red = (100.0 + disease_progress * 155.0) as u8; // Red intensifies with disease
                let base_green = (80.0 * (1.0 - disease_progress * 0.8)) as u8; // Green fades
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let red = (base_red as f32 * size_boost).min(255.0) as u8;
                let green = (base_green as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(red, green, 0) // Red-brown disease color
            },
            TileType::PlantRoot(age, size) => {
                let base_intensity = (200u16.saturating_sub(age as u16)).max(80) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity / 2, intensity / 3, intensity / 4) // Brown-ish root color
            },
            TileType::PillbugHead(age, size) => {
                let base_intensity = (180u16.saturating_sub(age as u16)).max(60) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity.saturating_add(20), intensity, intensity.saturating_sub(10)) // Slightly reddish head
            },
            TileType::PillbugBody(age, size) => {
                let base_intensity = (180u16.saturating_sub(age as u16)).max(50) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity, intensity) // Gray body
            },
            TileType::PillbugLegs(age, size) => {
                let base_intensity = (180u16.saturating_sub(age as u16)).max(40) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity.saturating_sub(20), intensity.saturating_sub(10), intensity) // Slightly bluish legs
            },
            TileType::PillbugDecaying(age, size) => {
                let decay_progress = age as f32 / 20.0; // 0.0 = fresh decay, 1.0 = almost nutrient
                let base_intensity = (80.0 * (1.0 - decay_progress * 0.7)) as u8; // Darken significantly over time
                let size_boost = match size {
                    Size::Small => 0.7,
                    Size::Medium => 1.0,
                    Size::Large => 1.3,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 3, intensity / 2) // Dark brownish-red decay color
            },
            TileType::Nutrient => Color::Magenta,
            TileType::Seed(age, size) => {
                let vitality = (100u16.saturating_sub(age as u16)).max(50) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let red = (vitality as f32 * 0.6 * size_boost) as u8;
                let green = (vitality as f32 * 0.4 * size_boost) as u8;
                let blue = (vitality as f32 * 0.2 * size_boost) as u8;
                Color::Rgb(red, green, blue) // Brown-ish seeds
            },
            TileType::Spore(age) => {
                let vitality = (50u16.saturating_sub(age as u16)).max(20) as u8;
                Color::Rgb(vitality, vitality / 2, vitality / 3) // Fading brownish spores
            },
        }
    }
    
    pub fn is_plant(self) -> bool {
        matches!(self, TileType::PlantStem(_, _) | TileType::PlantLeaf(_, _) | TileType::PlantBud(_, _) | TileType::PlantBranch(_, _) | TileType::PlantFlower(_, _) | TileType::PlantWithered(_, _) | TileType::PlantDiseased(_, _) | TileType::PlantRoot(_, _))
    }
    
    pub fn is_pillbug(self) -> bool {
        matches!(self, TileType::PillbugHead(_, _) | TileType::PillbugBody(_, _) | TileType::PillbugLegs(_, _) | TileType::PillbugDecaying(_, _))
    }
    
    pub fn get_size(self) -> Option<Size> {
        match self {
            TileType::PlantStem(_, size) | TileType::PlantLeaf(_, size) | 
            TileType::PlantBud(_, size) | TileType::PlantBranch(_, size) | TileType::PlantFlower(_, size) | TileType::PlantWithered(_, size) | TileType::PlantDiseased(_, size) | TileType::PlantRoot(_, size) |
            TileType::PillbugHead(_, size) | TileType::PillbugBody(_, size) | TileType::PillbugLegs(_, size) | TileType::PillbugDecaying(_, size) => Some(size),
            _ => None,
        }
    }
    
    pub fn is_water(self) -> bool {
        matches!(self, TileType::Water(_))
    }
    
    pub fn get_water_depth(self) -> Option<u8> {
        match self {
            TileType::Water(depth) => Some(depth),
            _ => None,
        }
    }
    
    pub fn can_water_flow_into(self) -> bool {
        matches!(self, TileType::Empty)
    }
    
    pub fn blocks_water(self) -> bool {
        !matches!(self, TileType::Empty | TileType::Water(_))
    }
    
    pub fn is_wind_dispersible(self) -> bool {
        matches!(self, TileType::Seed(_, _) | TileType::Spore(_) | TileType::Nutrient)
    }
    
    pub fn is_light_particle(self) -> bool {
        matches!(self, TileType::Seed(_, Size::Small) | TileType::Spore(_) | TileType::Nutrient | TileType::Water(0..=30))
    }
}

impl Biome {
    /// Moisture retention factor - affects water pooling and evaporation
    pub fn moisture_retention(self) -> f32 {
        match self {
            Biome::Wetland => 1.4,   // Retains water well
            Biome::Grassland => 1.0, // Normal retention
            Biome::Drylands => 0.6,  // Loses water quickly
            Biome::Woodland => 1.2,  // Good retention under tree cover
        }
    }
    
    /// Plant growth modifier - affects how well plants grow in this biome
    pub fn plant_growth_modifier(self) -> f32 {
        match self {
            Biome::Wetland => 1.3,   // Lush growth
            Biome::Grassland => 1.0, // Normal growth
            Biome::Drylands => 0.7,  // Sparse growth
            Biome::Woodland => 1.5,  // Dense growth
        }
    }
    
    /// Nutrient concentration - affects nutrient spawning and availability
    pub fn nutrient_modifier(self) -> f32 {
        match self {
            Biome::Wetland => 1.1,   // Rich nutrients from decomposition
            Biome::Grassland => 1.0, // Balanced nutrients
            Biome::Drylands => 0.8,  // Fewer nutrients
            Biome::Woodland => 1.4,  // Very rich soil
        }
    }
    
    /// Terrain composition - affects what terrain types are common
    pub fn get_terrain_preferences(self) -> (f32, f32) { // (dirt_ratio, sand_ratio)
        match self {
            Biome::Wetland => (0.8, 0.2),   // More dirt for water retention
            Biome::Grassland => (0.7, 0.3), // Balanced
            Biome::Drylands => (0.4, 0.6),  // More sand
            Biome::Woodland => (0.9, 0.1),  // Rich soil, little sand
        }
    }
    
    /// Rain accumulation bonus - how much more/less rain stays in this biome
    pub fn rain_accumulation_bonus(self) -> f32 {
        match self {
            Biome::Wetland => 1.5,   // Forms pools easily
            Biome::Grassland => 1.0, // Normal accumulation
            Biome::Drylands => 0.7,  // Rain flows away quickly
            Biome::Woodland => 1.2,  // Tree cover helps retention
        }
    }
}

pub fn random_size(rng: &mut impl Rng) -> Size {
    match rng.gen_range(0..10) {
        0..=2 => Size::Small,   // 30% small
        3..=6 => Size::Medium,  // 40% medium  
        7..=9 => Size::Large,   // 30% large
        _ => Size::Medium,
    }
}

pub fn random_biome(rng: &mut impl Rng) -> Biome {
    match rng.gen_range(0..4) {
        0 => Biome::Wetland,
        1 => Biome::Grassland,
        2 => Biome::Drylands,
        _ => Biome::Woodland,
    }
}