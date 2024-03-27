use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create window");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    // event_loop.set_control_flow(ControlFlow::Wait);

    // if #[cfg(target_arch = "wasm32")] {
    //     use winit::platform::web::EventLoopExtWebSys;
    //     let event_loop_function = EventLoop::spawn;
    // } else {
    let event_loop_function = EventLoop::run;
    // }

    event_loop_function(
        event_loop,
        |evt, elwt: &EventLoopWindowTarget<()>| match evt {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => elwt.exit(),
                _ => {}
            },
            _ => {}
        },
    )
    .expect("Failed to run event loop")
}
