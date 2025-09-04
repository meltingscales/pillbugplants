mod types;
mod world;
mod life;
mod physics;
mod environment;
mod app;

use std::env;
use std::fs::File;
use std::io::{self, Write};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use crate::world::World;
use crate::app::{App, run_app};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut sim_ticks: Option<u64> = None;
    let mut output_file: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            arg if arg.starts_with("--sim-ticks=") => {
                let ticks_str = arg.strip_prefix("--sim-ticks=").unwrap();
                sim_ticks = Some(ticks_str.parse().map_err(|_| "Invalid --sim-ticks value")?);
            }
            arg if arg.starts_with("--output-file=") => {
                let file_str = arg.strip_prefix("--output-file=").unwrap();
                output_file = Some(file_str.to_string());
            }
            "--help" | "-h" => {
                println!("Pillbug Plants Simulation");
                println!("Usage: {} [options]", args[0]);
                println!("Options:");
                println!("  --sim-ticks=N    Run simulation for N ticks and exit");
                println!("  --output-file=F  Save simulation output to file F");
                println!("  --help, -h       Show this help message");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    // Run in simulation mode if --sim-ticks is specified
    if let Some(ticks) = sim_ticks {
        return run_simulation(ticks, output_file);
    }
    
    // Set up panic hook to restore terminal state
    std::panic::set_hook(Box::new(|panic_info| {
        // Try to restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        
        eprintln!("{}", panic_info);
    }));
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    let world_width = size.width.saturating_sub(4) as usize;
    let world_height = size.height.saturating_sub(6) as usize;
    
    let mut app = App::new(world_width, world_height);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_simulation(ticks: u64, output_file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a world with fixed dimensions for consistency
    let world_width = 80;
    let world_height = 40;
    let mut world = World::new(world_width, world_height);
    
    println!("Running simulation for {} ticks...", ticks);
    
    // Run simulation
    for tick in 0..ticks {
        world.update();
        
        // Print progress every 100 ticks
        if tick % 100 == 0 || tick == ticks - 1 {
            println!("Progress: {}/{} ticks", tick + 1, ticks);
        }
    }
    
    let final_state = world.to_string();
    
    // Output results
    if let Some(file_path) = output_file {
        let mut file = File::create(&file_path)?;
        write!(file, "{}", final_state)?;
        println!("Simulation results saved to: {}", file_path);
    } else {
        println!("Final simulation state:");
        print!("{}", final_state);
    }
    
    Ok(())
}