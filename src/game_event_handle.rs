use super::Game;
use super::GameEntry;
use crate::gfx;
use std::sync::Arc;
use std::sync::Mutex;
use winit::application::ApplicationHandler;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event_loop;
use winit::keyboard;
use winit::keyboard::KeyCode;
use winit::window::WindowAttributes;

impl ApplicationHandler for GameEntry {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        println!("Resumed");
        match self {
            GameEntry::Ready(game) => {
                // game.resumed(event_loop);
            }
            GameEntry::Loading => {
                let window = Arc::new(
                    event_loop
                        .create_window(WindowAttributes::default())
                        .unwrap(),
                );
                pollster::block_on(async move {
                    println!("in async : Loading");
                    let context = gfx::GfxContext::new(window.clone()).await;
                    let context = Arc::new(Mutex::new(context));
                    let game = Game::new(window, context.clone());
                    *self = GameEntry::Ready(game);
                    println!("in async : Ready");
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let GameEntry::Ready(game) = self {
            match event {
                winit::event::WindowEvent::Resized(size) => {
                    println!("Resized");
                    game.bridge_with_gfx(size);
                    // now arrivate the normal full size in window
                    game.list_painter();
                    game.window.request_redraw();
                }
                winit::event::WindowEvent::Moved(_) => {
                    println!("Moved")
                }
                winit::event::WindowEvent::CloseRequested => {
                    println!("CloseRequested")
                }
                winit::event::WindowEvent::Destroyed => {
                    println!("Destroyed")
                }
                winit::event::WindowEvent::Focused(_) => {
                    println!("Focused")
                }
                winit::event::WindowEvent::KeyboardInput { event, .. } => match event {
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    } => {
                        if state == ElementState::Pressed {
                            println!("KeyboardInput: {:?}", physical_key);
                            if physical_key == keyboard::PhysicalKey::Code(KeyCode::Space) {
                                game.mount_next_scene();
                                game.window.request_redraw();
                            }
                        }
                    }
                },
                winit::event::WindowEvent::RedrawRequested => {
                    println!("RedrawRequested");
                    game.studio.as_ref().unwrap().render_current_scene();
                }
                _ => {}
            }
        }
    }
}
