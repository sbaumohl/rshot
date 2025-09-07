mod state;
use std::{
    env::{self},
    fs::OpenOptions,
    io::Write,
    os::fd::AsFd,
};

use wayland_client::{Connection, EventQueue};

use state::MockEvent;

fn main() {
    let connection = Connection::connect_to_env().unwrap();

    let display = connection.display();

    let mut e_queue: EventQueue<MockEvent> = connection.new_event_queue();
    let handle = e_queue.handle();

    let _ = display.get_registry(&handle, ());

    let mut state: MockEvent = MockEvent::default();
    e_queue.roundtrip(&mut state).unwrap();

    if let Some(b) = state.empty_shm {
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("ss.tmp");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .unwrap();
        file.write_all(b" ").unwrap();

        let mem_buf = unsafe { memmap2::Mmap::map(&file).unwrap() };
        mem_buf.make_mut().unwrap();

        let pool = b.create_pool(file.as_fd(), 1, &handle, ());
        let shm_buf = pool.create_buffer(
            0,
            1,
            1,
            1,
            wayland_client::protocol::wl_shm::Format::Argb16161616,
            &handle,
            (),
        );

        if let Some(mgr) = state.ss_manager {
            let output = mgr.capture_output(0, &state.output.unwrap(), &handle, ());
            println!("{:?}", output);
        }
    }
}
