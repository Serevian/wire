mod engine;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::Engine;

#[derive(Default)]
struct App {
    window: Option<Box<dyn Window>>,
    engine: Option<Engine>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // The event loop has launched, and we can initialize our UI state.

        let attributes = WindowAttributes::default().with_resizable(false);
        self.window = Some(event_loop.create_window(attributes).unwrap());
        let display_handle = self
            .window
            .as_ref()
            .unwrap()
            .display_handle()
            .unwrap()
            .as_raw();

        let window_handle = self
            .window
            .as_ref()
            .unwrap()
            .window_handle()
            .unwrap()
            .as_raw();

        let required_extensions = ash_window::enumerate_required_extensions(display_handle)
            .expect("Couldn't get required extensions");

        self.engine = Some(Engine::new(
            required_extensions,
            display_handle,
            window_handle,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Called by `EventLoop::run_app` when a new event happens on the window.
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                self.window.as_ref().unwrap().pre_present_notify();

                // Swap buffers.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::KeyQ),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                is_synthetic: _,
            } => event_loop.exit(),
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(App::default())?;

    Ok(())
}
