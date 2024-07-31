use crate::gfx;
use crate::Game;
use crate::GameEntry;
use lazy_static::lazy_static;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use winit::application::ApplicationHandler;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event_loop;
use winit::event_loop::ControlFlow;
use winit::keyboard;
use winit::keyboard::KeyCode;
use winit::window::WindowAttributes;
lazy_static! {
    pub static ref FRAME_DURATION: std::time::Duration =
        std::time::Duration::from_secs_f32(1.0 / 60.0);
}
impl ApplicationHandler for GameEntry {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        println!("Resumed");
        let next_frame_time = std::time::Instant::now() + *FRAME_DURATION;
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
        match self {
            GameEntry::Ready(game) => {}
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

    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        if let GameEntry::Ready(game) = self {
            let now = std::time::Instant::now();
            let delta_time = now - game.last_update;

            if delta_time >= *FRAME_DURATION {
                // 更新游戏逻辑
                game.update_game(delta_time.as_secs_f32());

                // 渲染
                game.window.request_redraw();

                game.last_update = now;
            }

            // 计算下一帧的时间
            let next_frame_time = game.last_update + *FRAME_DURATION;
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let GameEntry::Ready(game) = self {
            if game.gui.is_some() {
                game.gui
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .handle_input(&game.window, &event);
            }

            match event {
                winit::event::WindowEvent::Resized(size) => {
                    println!("Resized");
                    game.bridge_with_gfx(size);
                    // now arrivate the normal full size in window
                    game.set_gui();
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
