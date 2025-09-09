mod state;

use image::{ImageBuffer, RgbaImage};

use wayland_client::{Connection, EventQueue};

use state::MockEvent;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;

fn main() {
    let connection = Connection::connect_to_env().unwrap();
    let display = connection.display();
    let mut e_queue: EventQueue<MockEvent> = connection.new_event_queue();
    let handle = e_queue.handle();
    let _ = display.get_registry(&handle, ());

    let mut state: MockEvent = MockEvent::new();
    e_queue.roundtrip(&mut state).unwrap();

    let mut output: Option<ZwlrScreencopyFrameV1> = None;

    if let Some(mgr) = state.ss_manager.as_ref() {
        output = Some(mgr.capture_output(1, &state.wl_outputs[0], &handle, ()));
    }

    e_queue.roundtrip(&mut state).unwrap();

    {
        let shm_buf = state.wl_buffer.as_ref().expect("Did not unwrap wlBuffer!");
        output
            .as_mut()
            .expect("Did not get a Some(Frame)")
            .copy(shm_buf);
    }
    e_queue.blocking_dispatch(&mut state).unwrap();

    let temp_file = state.file.expect("Could not unwrap Temp File");
    let mem_buf = unsafe { memmap2::Mmap::map(&temp_file).unwrap() };

    let height = state.image_dims.height;
    let width = state.image_dims.width;
    let stride = state.image_dims.stride;
    let data = &mem_buf;
    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        let row_start = (y * stride) as usize;
        for pixel_start in (0..width).map(|x| row_start + (x * 4) as usize) {
            let r = data[pixel_start + 2];
            let g = data[pixel_start + 1];
            let b = data[pixel_start];

            rgba_data.extend_from_slice(&[r, g, b, 255]);
        }
    }
    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba_data).unwrap();
    img.save("screenshot.png").unwrap();
}
