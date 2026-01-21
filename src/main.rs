mod state;
mod parser;
mod tcp;
mod web;

use std::sync::Arc;
use tokio::runtime::Runtime;
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use tao::event_loop::{ControlFlow, EventLoop};
use log::info;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--debug".to_string()) {
        unsafe { std::env::set_var("RUST_LOG", "debug"); }
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let event_loop = EventLoop::new();
    
    // Create Tray Menu
    let tray_menu = Menu::new();
    let open_web = MenuItem::new("Open Webpage", true, None);
    let quit = MenuItem::new("Quit", true, None);
    
    tray_menu.append(&open_web).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&quit).unwrap();
    
    // Create Tray Icon
    // Ideally we load an icon. For now, we might fail or use a default if possible.
    // tray-icon requires an Icon struct.
    // Let's try to generate a simple icon or load one. 
    // For MVP transparency, maybe just a colored box.
    
    let icon = load_icon();
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Lynx vMix Bridge")
        .with_icon(icon)
        .build()
        .unwrap();

    // Initialize State
    let state = state::initialize_state();
    
    // Create Tokio Runtime
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    
    // Spawn Tasks
    let state_tcp = state.clone();
    rt.spawn(async move {
        tcp::start_listener(state_tcp, 12345).await;
    });
    
    let state_web = state.clone();
    rt.spawn(async move {
        web::start_server(state_web, 3000).await;
    });

    info!("Application started. TCP: 12345, Web: 3000");

    // Run Event Loop
    let menu_channel = tray_icon::menu::MenuEvent::receiver();
    let tray_channel = tray_icon::TrayIconEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == open_web.id() {
                let _ = open::that("http://localhost:3000");
            } else if event.id == quit.id() {
                let _ = *control_flow = ControlFlow::Exit;
            }
        }
        
        if let Ok(_event) = tray_channel.try_recv() {
            // Handle tray click events if needed
        }
    });
}

fn load_icon() -> tray_icon::Icon {
    // specific red square as generic icon
    let width = 64;
    let height = 64;
    let mut rgba = Vec::new();
    for _ in 0..height {
        for _ in 0..width {
            rgba.push(255); // R
            rgba.push(0);   // G
            rgba.push(0);   // B
            rgba.push(255); // A
        }
    }
    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
