mod argparser;
mod render;
mod state;

use std::process::exit;

use clap::Parser;
use image::{ImageBuffer, RgbaImage};

use wayland_client::{Connection, EventQueue, QueueHandle};

use argparser::Args;
use state::RShotState;

use crate::render::{crop_to_region, get_rgba};

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
    // TODO add params for output no., cursor
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

    if args.dry_run {
        println!("{:#?}", args);
        exit(0);
    }

    let (mut queue, handle) = initialize();

    let mut state: RShotState = RShotState::new();

    queue.roundtrip(&mut state).unwrap();

    let mut img = capture_screenshot(&mut state, &mut queue, &handle).unwrap();

    if let Some(region) = args.region.as_ref() {
        img = crop_to_region(&img, region).unwrap_or_else(|err| {
            println!("{}", err);
            exit(1);
        });
    }

    img.save(args.get_output_dir()).unwrap();

    if !args.no_prompt {
        state.create_layer_surface(&handle);
        while state.application_open {
            queue.blocking_dispatch(&mut state).unwrap();
        }
    }
}
