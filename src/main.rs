mod argparser;
mod state;

use clap::Parser;
use image::{ImageBuffer, RgbaImage};

use wayland_client::{protocol::wl_shm::Format, Connection, EventQueue, QueueHandle};

use argparser::Args;
use state::{ImageDims, RShotState};

fn get_rgba(dims: &ImageDims, buf: &memmap2::Mmap) -> Vec<u8> {
    match dims.format {
        Format::Xrgb8888 | Format::Argb8888 => {
            let mut rgba_data = Vec::with_capacity(dims.total_size());

            for idx in (0..(dims.height * dims.stride) as usize).step_by(4) {
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

fn initialize() -> (EventQueue<RShotState>, QueueHandle<RShotState>) {
    let connection = Connection::connect_to_env().unwrap();
    let display = connection.display();
    let e_queue: EventQueue<RShotState> = connection.new_event_queue();
    let handle = e_queue.handle();
    let _ = display.get_registry(&handle, ());

    (e_queue, handle)
}

fn capture_screenshot(
    state: &mut RShotState,
    queue: &mut EventQueue<RShotState>,
    handle: &QueueHandle<RShotState>,
) -> Result<RgbaImage, ()> {
    // TODO add parems for output no., cursor
    let frame = state
        .capture_screenshot(handle)
        .expect("Could not capture screenshot!");

    queue.roundtrip(state).unwrap();

    let shm_buf = state.wl_buffer.as_ref().expect("Did not unwrap WlBuffer!");
    frame.copy(shm_buf);

    queue.blocking_dispatch(state).unwrap();

    let fd = state
        .screenshot_fd
        .as_ref()
        .expect("Could not unwrap buffer fd!");
    let mem_buf = unsafe { memmap2::Mmap::map(fd).unwrap() };

    let img: RgbaImage = ImageBuffer::from_raw(
        state.image_dims.width,
        state.image_dims.height,
        get_rgba(&state.image_dims, &mem_buf),
    )
    .unwrap();

    Ok(img)
}

fn main() {
    let mut args = Args::parse();

    let (mut queue, handle) = initialize();

    let mut state: RShotState = RShotState::new();

    queue.roundtrip(&mut state).unwrap();

    let img = capture_screenshot(&mut state, &mut queue, &handle).unwrap();
    img.save(args.get_output_dir()).unwrap();

    if !args.no_prompt {
        state.create_layer_surface(&handle);
        while state.application_open {
            queue.blocking_dispatch(&mut state).unwrap();
        }
    }
}
