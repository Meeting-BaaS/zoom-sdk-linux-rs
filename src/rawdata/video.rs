use std::fmt::Debug;
use std::ptr;
use std::sync::{Arc, Mutex};

use crate::{SdkResult, ZoomRsError, ZoomSdkResult};

use crate::bindings::*;

/// Raw data of an image.
pub type ExportedVideoRawData = exported_video_raw_data;

/// RawData video events from delegate.
pub trait RawVideoEvent: Debug {
    /// Get Data frame.
    fn on_raw_data_frame_received(&mut self, _data: &ExportedVideoRawData);
    /// On status change.
    fn on_raw_data_status_changed(&mut self, _status: bool, time: i64);
    /// Notify the current renderer object is going to be destroyed.
    /// After you handle this callback, you should never user this renderer object any more
    fn on_renderer_be_destroyed(&mut self, time: i64);
    /// Use it when you want to do last operation after unsubscribing.
    fn flush(&mut self);
}

/// A renderer takes a delegate and allows retrieving images.
#[derive(Debug)]
pub struct Renderer {
    renderer: Option<*mut ZOOMSDK_IZoomSDKRenderer>,
    #[allow(dead_code)]
    delegate: *mut ZOOMSDK_IZoomSDKRendererDelegate,
    evt_mutex: Arc<Mutex<Box<dyn RawVideoEvent>>>,
}

impl Renderer {
    /// Create a new renderer.
    pub fn new(
        evt_mutex: Arc<Mutex<Box<dyn RawVideoEvent>>>,
        resolution: VideoResolution,
    ) -> SdkResult<Self> {
        let mut renderer: *mut ZOOMSDK_IZoomSDKRenderer = ptr::null_mut();
        let ptr = Arc::as_ptr(&evt_mutex) as *mut _;
        let delegate = unsafe { video_helper_create_delegate(ptr) };
        let result: Result<(), ZoomRsError> = ZoomSdkResult(
            unsafe { ZOOMSDK_createRenderer(&mut renderer, delegate) },
            (),
        )
        .into();
        result.map(|_| {
            tracing::info!("Resolution : {:?}", unsafe {
                set_raw_data_resolution(renderer, resolution as u32)
            });
            Self {
                renderer: Some(renderer),
                delegate,
                evt_mutex,
            }
        })
    }
    /// Subscribe a delegate for given user_id and type.
    pub fn subscribe_delegate(&mut self, user_id: u32, data_type: RawDataType) -> SdkResult<()> {
        tracing::debug!("ptr renderer : {:?}", self.renderer);
        match self.renderer {
            Some(renderer) => ZoomSdkResult(
                unsafe { video_helper_subscribe_delegate(renderer, user_id, data_type as u32) },
                (),
            )
            .into(),
            None => {
                tracing::warn!("Cannot Subscribe : Renderer is in invalid state");
                Err(ZoomRsError::NullPtr)
            }
        }
    }
    /// Unsubscribe the renderer delegate.
    pub fn unsubscribe_delegate(&mut self) -> SdkResult<()> {
        tracing::debug!("ptr renderer : {:?}", self.renderer);
        match self.renderer {
            Some(renderer) => {
                ZoomSdkResult(unsafe { video_helper_unsubscribe_delegate(renderer) }, ()).into()
            }
            None => {
                tracing::warn!("Cannot Unsubscribe : Renderer is invalid state");
                Err(ZoomRsError::NullPtr)
            }
        }
    }

    /// The renderer is not valid anymore according to documentation.
    /// Notify the current renderer object is going to be destroyed.
    /// After you handle this callback, you should never user this renderer object any more.
    /// virtual void onRendererBeDestroyed() = 0;
    pub fn invalid(&mut self) {
        self.renderer = None;
        tracing::warn!("Invalid Renderer Pointer");
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        tracing::info!("Droping renderer !");
        let r = self.unsubscribe_delegate();
        if let Err(e) = r {
            tracing::warn!("Error when unsubscribing delegate: {:?}", e);
        }
        tracing::info!("Flushing renderer...");
        self.evt_mutex.lock().unwrap().flush();
        tracing::info!("Destroying renderer...");
        match self.renderer {
            Some(renderer) => {
                tracing::debug!("ZOOMSDK_destroyRenderer");
                let ret = unsafe { ZOOMSDK_destroyRenderer(renderer) };
                if ret != 0 {
                    tracing::warn!("Error when destroying renderer: {:?}", ret);
                }
            }
            None => tracing::error!("Renderer is invalid"),
        }
        tracing::info!("Renderer instance droped!");
    }
}

/// Type of data to subscribe.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum RawDataType {
    /// Video from camera.
    Video = ZOOMSDK_ZoomSDKRawDataType_RAW_DATA_TYPE_VIDEO,
    /// Sharing screen.
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

/// Resolution MAX of the input images.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum VideoResolution {
    R90P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_90P,
    R180P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_180P,
    R360P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_360P,
    R720P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_720P,
    R1080P = ZOOMSDK_ZoomSDKResolution_ZoomSDKResolution_1080P,
}
