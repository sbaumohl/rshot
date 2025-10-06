use std::os::fd::{AsFd, OwnedFd};

use rshot_macros::default_dispatch;
use rustix::fs::{memfd_create, MemfdFlags};
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_data_device::WlDataDevice,
        wl_keyboard::{self, WlKeyboard},
        wl_output::WlOutput,
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
        wl_shm::{Format, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{
        Anchor, Event as LayerSurfaceEvent, KeyboardInteractivity, ZwlrLayerSurfaceV1,
    },
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
#[default_dispatch(
    WlCompositor,
    WlSurface,
    WlShm,
    WlShmPool,
    WlBuffer,
    WlOutput,
    ZwlrScreencopyManagerV1,
    ZwlrLayerShellV1,
    WlDataDevice
)]
pub struct RShotState {
    pub ss_manager: Option<ZwlrScreencopyManagerV1>,
    pub shell: Option<ZwlrLayerShellV1>,
    pub wl_compositor: Option<WlCompositor>,
    pub wl_shm: Option<WlShm>,
    pub wl_outputs: Vec<WlOutput>,
    pub wl_buffer: Option<WlBuffer>,
    pub wl_shm_pool: Option<WlShmPool>,
    pub wl_seat: Option<WlSeat>,

    pub application_open: bool,
    pub application_surface: Option<(WlSurface, ZwlrLayerSurfaceV1)>,

    pub screenshot_fd: Option<OwnedFd>,
    pub image_dims: ImageDims,
}

impl RShotState {
    pub fn create_buffer(
        &mut self,
        offset: i32,
        height: i32,
        width: i32,
        stride: i32,
        format: Format,
        handle: &QueueHandle<RShotState>,
    ) {
        let total_size = stride * height;
        let fd = RShotState::create_temp_file(total_size as u64);
        let pool = self
            .wl_shm
            .as_ref()
            .unwrap()
            .create_pool(fd.as_fd(), total_size, handle, ());
        let shm_buf = pool.create_buffer(offset, width, height, stride, format, handle, ());
        self.wl_shm_pool = Some(pool);
        self.wl_buffer = Some(shm_buf);
        self.screenshot_fd = Some(fd);
    }

    pub fn create_temp_file(no_bytes: u64) -> OwnedFd {
        let fd = memfd_create("screenshot_buffer", MemfdFlags::CLOEXEC).unwrap();
        rustix::io::retry_on_intr(|| rustix::fs::ftruncate(&fd, no_bytes)).unwrap();

        fd
    }

    pub fn new() -> Self {
        RShotState {
            application_open: false,
            ..Default::default()
        }
    }

    pub fn capture_screenshot(
        &self,
        qhandle: &QueueHandle<RShotState>,
    ) -> Result<ZwlrScreencopyFrameV1, ()> {
        match self.ss_manager.as_ref() {
            None => Err(()),
            Some(mgr) => Ok(mgr.capture_output(0, &self.wl_outputs[0], qhandle, ())),
        }
    }

    pub fn create_layer_surface(&mut self, qhandle: &QueueHandle<RShotState>) {
        if self.shell.is_none() || self.wl_compositor.is_none() {
            return;
        }

        let shell = self.shell.as_ref().unwrap();
        let compositor = self.wl_compositor.as_ref().unwrap();

        let surface = compositor.create_surface(qhandle, ());

        let layer_surface = shell.get_layer_surface(
            &surface,
            Some(&self.wl_outputs[0]),
            Layer::Top,
            "Make Selection".to_string(),
            qhandle,
            (),
        );

        layer_surface.set_anchor(Anchor::all());
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        surface.commit();

        self.application_open = true;
        self.application_surface = Some((surface, layer_surface));
    }

    fn render_layer_surface(&mut self, qhandle: &QueueHandle<RShotState>) {
        let (surface, _) = self.application_surface.as_ref().unwrap();

        let width = 500;
        let height = 500;

        let stride = width * 4;
        let size = stride * height;

        // Create in-memory file descriptor
        let fd = memfd_create("wayland-buffer", MemfdFlags::CLOEXEC).unwrap();
        rustix::io::retry_on_intr(|| rustix::fs::ftruncate(&fd, size as u64)).unwrap();

        let pool = self
            .wl_shm
            .as_ref()
            .unwrap()
            .create_pool(fd.as_fd(), size, qhandle, ());
        let buffer = pool.create_buffer(0, width, height, stride, Format::Argb8888, qhandle, ());

        let mut mmap = unsafe { memmap2::MmapMut::map_mut(&fd).unwrap() };

        for pixel in mmap.chunks_exact_mut(4) {
            pixel[0] = 0; // Blue
            pixel[1] = 0; // Green
            pixel[2] = 50; // Red
            pixel[3] = 128; // Alpha
        }

        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width, height);
        surface.commit();
    }
}

impl Dispatch<WlRegistry, ()> for RShotState {
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
                "zwlr_layer_shell_v1" => {
                    let shell = proxy.bind::<ZwlrLayerShellV1, _, _>(name, version, qhandle, ());
                    state.shell = Some(shell);
                }
                "wl_compositor" => {
                    state.wl_compositor =
                        Some(proxy.bind::<WlCompositor, _, _>(name, version, qhandle, ()));
                }
                "wl_seat" => {
                    state.wl_seat = Some(proxy.bind::<WlSeat, _, _>(name, version, qhandle, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for RShotState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
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

impl Dispatch<ZwlrLayerSurfaceV1, ()> for RShotState {
    fn event(
        state: &mut Self,
        surface: &ZwlrLayerSurfaceV1,
        event: <ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            LayerSurfaceEvent::Configure {
                serial,
                width,
                height,
            } => {
                surface.ack_configure(serial);
                if let (Some(_), Some(_)) = (&state.wl_shm, &state.wl_compositor) {
                    state.render_layer_surface(qhandle);
                }
            }
            LayerSurfaceEvent::Closed => {
                state.application_open = false;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlSeat, ()> for RShotState {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(val),
        } = event
        {
            if val.contains(wl_seat::Capability::Keyboard) {
                let keyboard = seat.get_keyboard(qhandle, ());
            }
        }
    }
}

impl Dispatch<WlKeyboard, ()> for RShotState {
    fn event(
        app_state: &mut Self,
        proxy: &WlKeyboard,
        event: <WlKeyboard as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                serial,
                time,
                key,
                state,
            } => {
                // check for `esc` key
                // TODO don't use 1, find constant
                if WEnum::Value(wl_keyboard::KeyState::Pressed) == state && key == 1 {
                    println!("Exiting...");
                    app_state.application_open = false;
                }
            }
            // wl_keyboard::Event::Leave { serial, surface } => {}
            // wl_keyboard::Event::Modifiers {
            //     serial,
            //     mods_depressed,
            //     mods_latched,
            //     mods_locked,
            //     group,
            // } => {}
            // wl_keyboard::Event::Key {
            //     serial,
            //     time,
            //     key,
            //     state,
            // } => {}
            _ => {}
        }
    }
}
