mod argparser;
mod state;

use clap::Parser;
use image::{ImageBuffer, RgbaImage};

use wayland_client::{protocol::wl_shm::Format, Connection, EventQueue};

use argparser::Args;
use state::MockEvent;

use crate::state::ImageDims;

fn get_rgba(dims: &ImageDims, buf: &memmap2::Mmap) -> Vec<u8> {
    match dims.format {
        Format::Xrgb8888 | Format::Argb8888 => {
            let mut rgba_data = Vec::with_capacity(dims.total_size());

            for idx in (0..dims.height)
                .flat_map(move |h| (0..dims.width).map(move |w| (h * dims.stride + w * 4) as usize))
            {
                let r = buf[idx + 2];
                let g = buf[idx + 1];
                let b = buf[idx];

                let a = buf[idx + 3];
                rgba_data.extend_from_slice(&[r, g, b, a]);
            }
            rgba_data
        }
        _ => {
            panic!("Format is not currently supported! ({:?})", dims.format);
        }
    }
}

fn main() {
    let mut args = Args::parse();

    let connection = Connection::connect_to_env().unwrap();
    let display = connection.display();
    let mut e_queue: EventQueue<MockEvent> = connection.new_event_queue();
    let handle = e_queue.handle();
    let _ = display.get_registry(&handle, ());

    let mut state: MockEvent = MockEvent::new();

    e_queue.roundtrip(&mut state).unwrap();

    // TODO add parems for output no., cursor
    let frame = state
        .capture_screenshot(&handle)
        .expect("Could not capture screenshot!");

    e_queue.roundtrip(&mut state).unwrap();

    let shm_buf = state.wl_buffer.as_ref().expect("Did not unwrap wlBuffer!");
    frame.copy(shm_buf);

    e_queue.blocking_dispatch(&mut state).unwrap();

    let temp_file = state.file.expect("Could not unwrap temp file!");
    let mem_buf = unsafe { memmap2::Mmap::map(&temp_file).unwrap() };

    let img: RgbaImage = ImageBuffer::from_raw(
        state.image_dims.width,
        state.image_dims.height,
        get_rgba(&state.image_dims, &mem_buf),
    )
    .unwrap();

    img.save(args.get_output_dir()).unwrap();
}
