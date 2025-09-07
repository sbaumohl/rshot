use std::{env, path::PathBuf};
use wayland_client::{
    protocol::{
        wl_registry::{self, WlRegistry},
        wl_shm_pool,
    },
    Connection, Dispatch, EventQueue, Proxy,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

#[derive(Debug, Default)]
struct MockEvent {
    ss_manager: Option<ZwlrScreencopyManagerV1>,
}

impl Dispatch<WlRegistry, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface.as_str() == "zwlr_screencopy_manager_v1" {
                let manager =
                    proxy.bind::<ZwlrScreencopyManagerV1, _, _>(name, version, qhandle, ());
                state.ss_manager = Some(manager);
            }
        }
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &ZwlrScreencopyManagerV1,
        event: <ZwlrScreencopyManagerV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        println!("CAPTURE!");
    }
}

fn create_buffer(sz: usize) -> wl_shm_pool::WlShmPool {
    let temp_dir = env::temp_dir();
    let file = temp_dir.join("ss.tmp");

    unsafe {
        memmap2::Mmap::map(file);
    }

    todo!();
}

fn main() {
    println!("Hello, world!");
    let connection = Connection::connect_to_env().unwrap();

    let display = connection.display();
    println!("{:?}", display);

    let mut e_queue: EventQueue<MockEvent> = connection.new_event_queue();
    let handle = e_queue.handle();

    // let registry = display.get_registry(&handle, ());

    let mut state: MockEvent = MockEvent::default();
    e_queue.roundtrip(&mut state).unwrap();

    println!("State: {:?}", state);
    if let Some(mgr) = state.ss_manager {
        // let mut buffer = mgr.capture_output(0, output, &handle, ());
    }
}
