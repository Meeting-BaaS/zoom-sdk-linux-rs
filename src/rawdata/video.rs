use std::fmt::Debug;
use std::ptr;
use std::sync::{Arc, Mutex};

use crate::{SdkResult, ZoomRsError, ZoomSdkResult};

use crate::bindings::*;

pub type ExportedVideoRawData = exported_video_raw_data;

/// RawData video events from delegate.
pub trait RawVideoEvent: Debug {
    /// Get Data frame.
    fn on_raw_data_frame_received(&mut self, _data: &ExportedVideoRawData);
    /// On status change.
    fn on_raw_data_status_changed(&mut self, _status: bool, time: i64);
    /// On renderer destroyed.
    fn on_renderer_be_destroyed(&mut self, time: i64);
    /// Use it when you want to do last operation after unsubscribing.
    fn flush(&mut self);
}

#[derive(Debug)]
pub struct Renderer<'a> {
    renderer: &'a mut ZOOMSDK_IZoomSDKRenderer,
    #[allow(dead_code)]
    delegate: &'a mut ZOOMSDK_IZoomSDKRendererDelegate,
    evt_mutex: Arc<Mutex<Box<dyn RawVideoEvent>>>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        evt_mutex: Arc<Mutex<Box<dyn RawVideoEvent>>>,
        resolution: VideoResolution,
    ) -> SdkResult<Self> {
        let mut ptr_renderer: *mut ZOOMSDK_IZoomSDKRenderer = ptr::null_mut();
        let ptr = Arc::as_ptr(&evt_mutex) as *mut _;
        let ptr_delegate = unsafe { video_helper_create_delegate(ptr) };
        let result: Result<(), ZoomRsError> = ZoomSdkResult(
            unsafe { ZOOMSDK_createRenderer(&mut ptr_renderer, ptr_delegate) },
            (),
        )
        .into();
        result.map(|_| {
            tracing::info!("Resolution : {:?}", unsafe {
                set_raw_data_resolution(ptr_renderer, resolution as u32)
            });
            Self {
                renderer: unsafe { ptr_renderer.as_mut() }.unwrap(),
                delegate: unsafe { ptr_delegate.as_mut() }.unwrap(),
                evt_mutex,
            }
        })
    }
    pub fn subscribe_delegate(&mut self, user_id: u32, data_type: RawDataType) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { video_helper_subscribe_delegate(self.renderer, user_id, data_type as u32) },
            (),
        )
        .into()
    }
    pub fn unsubscribe_delegate(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { video_helper_unsubscribe_delegate(self.renderer) },
            (),
        )
        .into()
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        tracing::info!("Droping renderer !");
        let r = self.unsubscribe_delegate();
        if let Err(e) = r {
            tracing::warn!("Error when unsubscribing delegate: {:?}", e);
        }
        tracing::info!("Flushing renderer...");
        self.evt_mutex.lock().unwrap().flush();
        tracing::info!("Destroying renderer...");
        let ret = unsafe { ZOOMSDK_destroyRenderer(self.renderer) };
        if ret != 0 {
            tracing::warn!("Error when destroying renderer: {:?}", ret);
        }
        tracing::info!("Renderer instance droped!");
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum RawDataType {
    Video = ZOOMSDK_ZoomSDKRawDataType_RAW_DATA_TYPE_VIDEO,
    Share = ZOOMSDK_ZoomSDKRawDataType_RAW_DATA_TYPE_SHARE,
}

#[tracing::instrument(level = "DEBUG", ret)]
#[no_mangle]
extern "C" fn on_raw_data_frame_received(ptr: *const u8, data: *const exported_video_raw_data) {
    if data.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        (*convert(ptr).lock().unwrap())
            .on_raw_data_frame_received(unsafe { data.as_ref() }.unwrap())
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_renderer_be_destroyed(ptr: *const u8, time: i64) {
    (*convert(ptr).lock().unwrap()).on_renderer_be_destroyed(time)
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_raw_data_status_changed(ptr: *const u8, status: bool, time: i64) {
    (*convert(ptr).lock().unwrap()).on_raw_data_status_changed(status, time)
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn RawVideoEvent>>> {
    let ptr: *const Mutex<Box<dyn RawVideoEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum VideoResolution {
    R90P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_90P,
    R180P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_180P,
    R360P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_360P,
    R720P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_720P,
    R1080P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_1080P,
}
