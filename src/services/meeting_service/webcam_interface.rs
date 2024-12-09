use crate::{bindings::*, SdkResult, ZoomSdkResult};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
/// This structure represents the ZOOM SDK video sender.
pub struct CamInterface(*mut ZOOMSDK_IZoomSDKVideoSender);

/// Unsafe Send boilerplate for CamInterface.
unsafe impl Send for CamInterface {}

/// Implements this Trait to send video data through virtual webcam.
pub trait VideoToWebcam: Debug {
    /// Event triggered when virtual camera is ready to take data.
    fn on_video_source_started(&mut self, interface: CamInterface);

    /// Event triggered when the virtual camera has stopped.
    fn on_video_source_stopped(&mut self);
}

/// Get WebCam injection boilerplates.
/// - If the function succeeds, the return value is Ok(, otherwise failed, see [SdkError] for details
pub fn new_webcam_injection_boitlerplate(
    meeting_service: &mut ZOOMSDK_IMeetingService,
    ctx: Box<dyn VideoToWebcam>,
) -> Option<Arc<Mutex<Box<dyn VideoToWebcam>>>> {
    let camera_mutex = Some(Arc::new(Mutex::new(ctx)));
    let ptr = Arc::as_ptr(camera_mutex.as_ref().unwrap());

    let result: SdkResult<()> = ZoomSdkResult(
        unsafe { init_video_to_virtual_webcam(meeting_service, ptr as _) },
        (),
    )
    .into();
    match result {
        Ok(_) => camera_mutex,
        Err(e) => {
            tracing::warn!("Unexpected result : {:?}", e);
            None
        }
    }
}

impl CamInterface {
    /// Use this method to send 640*480 YUV420 data to camera.
    /// - Unsafe as fuck -> Ensure you that [CamInterface] is always valid.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub unsafe fn send_video_buffer(&mut self, framebuffer: *const i8) -> SdkResult<()> {
        ZoomSdkResult(play_video_to_virtual_webcam(self.0, framebuffer), ()).into()
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn video_source_started(ptr: *const u8, sender: *mut ZOOMSDK_IZoomSDKVideoSender) {
    if sender.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        (*convert(ptr).lock().unwrap()).on_video_source_started(CamInterface(sender));
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn video_source_stopped(ptr: *const u8) {
    (*convert(ptr).lock().unwrap()).on_video_source_stopped();
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn VideoToWebcam>>> {
    let ptr: *const Mutex<Box<dyn VideoToWebcam>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}
