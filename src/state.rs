use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
        wl_registry::{self, WlRegistry},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
    },
    Connection, Dispatch, Proxy,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

#[derive(Debug, Default)]
pub struct MockEvent {
    pub ss_manager: Option<ZwlrScreencopyManagerV1>,
    pub empty_shm: Option<WlShm>,
    pub full_shm: Option<WlShm>,
    pub output: Option<WlOutput>,
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
                    state.empty_shm = Some(shm)
                }
                "wl_output" => {
                    let output = proxy.bind::<WlOutput, _, _>(name, version, qhandle, ());
                    state.output = Some(output);
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
