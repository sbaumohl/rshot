use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsFd,
};

use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
        wl_registry::{self, WlRegistry},
        wl_shm::{Format, WlShm},
        wl_shm_pool::WlShmPool,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{Event, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

#[derive(Debug)]
pub struct ImageDims {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: Format,
}

impl Default for ImageDims {
    fn default() -> Self {
        ImageDims {
            width: 0,
            height: 0,
            stride: 0,
            format: Format::Xbgr8888,
        }
    }
}

impl ImageDims {
    pub fn total_size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}

#[derive(Debug, Default)]
pub struct MockEvent {
    pub ss_manager: Option<ZwlrScreencopyManagerV1>,
    pub wl_shm: Option<WlShm>,
    pub wl_outputs: Vec<WlOutput>,
    pub wl_buffer: Option<WlBuffer>,
    pub wl_shm_pool: Option<WlShmPool>,
    pub file: Option<File>,
    pub image_dims: ImageDims,
}

impl MockEvent {
    pub fn create_buffer(
        &mut self,
        offset: i32,
        height: i32,
        width: i32,
        stride: i32,
        format: Format,
        handle: &QueueHandle<MockEvent>,
    ) {
        let total_size = stride * height;
        let file = MockEvent::create_temp_file("ss.tmp".to_string(), total_size as u64);
        let pool = self
            .wl_shm
            .as_ref()
            .unwrap()
            .create_pool(file.as_fd(), total_size, handle, ());
        let shm_buf = pool.create_buffer(offset, width, height, stride, format, handle, ());
        self.wl_shm_pool = Some(pool);
        self.wl_buffer = Some(shm_buf);
        self.file = Some(file);
    }

    pub fn create_temp_file(name: String, no_bytes: u64) -> File {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(name);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .unwrap();
        file.write_all((0..no_bytes).map(|_| ' ').collect::<String>().as_bytes())
            .unwrap();
        file
    }

    pub fn new() -> Self {
        MockEvent {
            ..Default::default()
        }
    }

    pub fn capture_screenshot(
        &self,
        qhandle: &QueueHandle<MockEvent>,
    ) -> Result<ZwlrScreencopyFrameV1, ()> {
        match self.ss_manager.as_ref() {
            None => Err(()),
            Some(mgr) => Ok(mgr.capture_output(0, &self.wl_outputs[0], qhandle, ())),
        }
    }
}

impl Dispatch<WlRegistry, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "zwlr_screencopy_manager_v1" => {
                    let manager =
                        proxy.bind::<ZwlrScreencopyManagerV1, _, _>(name, version, qhandle, ());
                    state.ss_manager = Some(manager);
                }
                "wl_shm" => {
                    let shm = proxy.bind::<WlShm, _, _>(name, version, qhandle, ());
                    state.wl_shm = Some(shm)
                }

                "wl_output" => {
                    let output = proxy.bind::<WlOutput, _, _>(name, version, qhandle, ());
                    state.wl_outputs.push(output);
                }
                _ => {}
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
        if let Event::Buffer {
            format,
            width,
            height,
            stride,
        } = event
        {
            println!("FORMAT: {:?}", format);
            // TODO handle other formats
            let f = if let WEnum::Value(f) = format {
                f
            } else {
                Format::Xbgr8888
            };
            state.image_dims = ImageDims {
                width,
                height,
                stride,
                format: f,
            };

            state.create_buffer(0, height as i32, width as i32, stride as i32, f, qhandle);
        }
    }
}

impl Dispatch<WlShm, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &WlShm,
        event: <WlShm as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &WlShmPool,
        event: <WlShmPool as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &WlBuffer,
        event: <WlBuffer as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlOutput, ()> for MockEvent {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: <WlOutput as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
