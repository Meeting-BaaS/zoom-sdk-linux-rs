use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// Rust type definition.
pub type ExportedAudioRawData = exported_audio_raw_data;

/// RawData audio events from delegate.
pub trait RawAudioEvent: Debug {
    /// Mixed audio represents all audio channels mixed.
    fn on_mixed_audio_raw_data(&mut self, _data: &ExportedAudioRawData) -> i32;
    /// Separate channels by users.
    fn on_one_way_audio_raw_data(&mut self, _data: &ExportedAudioRawData, _user_id: u32) -> i32;
    /// Sharing audio song from Zoom.
    fn on_share_audio_raw_data(&mut self, _data: &ExportedAudioRawData) -> i32;
    /// Use it when you want to do last operation after unsubscribing.
    fn flush(&mut self);
}

/// Audio RawData Helper.
#[derive(Debug)]
pub struct AudioRawDataHelper<'a> {
    ref_rawdata_helper: &'a mut ZOOMSDK_IZoomSDKAudioRawDataHelper,
    delegate: Option<RawAudioDelegate<'a>>,
}

impl<'a> AudioRawDataHelper<'a> {
    /// Create a new RawData helper.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn new() -> SdkResult<Self> {
        let ptr = unsafe { ZOOMSDK_GetAudioRawdataHelper() };
        if ptr.is_null() {
            return Err(ZoomRsError::NullPtr);
        }
        Ok(Self {
            ref_rawdata_helper: unsafe { ptr.as_mut() }.unwrap(),
            delegate: None,
        })
    }
    /// Subscribe raw audio data.
    /// - [RawAudioEvent], the callback handler of raw audio data.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn subscribe_delegate(
        &mut self,
        event: Box<dyn RawAudioEvent>,
        use_separate_channels: bool,
    ) -> SdkResult<()> {
        self.delegate = Some(RawAudioDelegate::new(event, use_separate_channels)?);
        ZoomSdkResult(
            unsafe {
                audio_helper_subscribe_delegate(
                    self.ref_rawdata_helper,
                    self.delegate.as_mut().unwrap().ref_delegate,
                )
            },
            (),
        )
        .into()
    }
    /// Unsubscribe raw audio data.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn unsubscribe_delegate(&mut self) -> SdkResult<()> {
        let result = ZoomSdkResult(
            unsafe { audio_helper_unsubscribe_delegate(self.ref_rawdata_helper) },
            (),
        )
        .into();
        if let Some(mut _trashes) = self.delegate.take() {
            _trashes.flush();
        }
        result
    }
}

/// Drop boilerplate for RawDataHelper.
impl<'a> Drop for AudioRawDataHelper<'a> {
    fn drop(&mut self) {
        let r = self.unsubscribe_delegate();
        if let Err(e) = r {
            tracing::warn!("Error when unsubscribing delegate: {:?}", e);
        }
        tracing::info!("RawDataHelper instance droped!");
    }
}

#[derive(Debug)]
struct RawAudioDelegate<'a> {
    evt_mutex: Option<Arc<Mutex<Box<dyn RawAudioEvent>>>>,
    ref_delegate: &'a mut ZOOMSDK_IZoomSDKAudioRawDataDelegate,
}

impl<'a> RawAudioDelegate<'a> {
    fn new(ctx: Box<dyn RawAudioEvent>, use_separate_channels: bool) -> SdkResult<Self> {
        let evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(evt_mutex.as_ref().unwrap()) as *mut _;
        let delegate = unsafe { audio_helper_create_delegate(ptr, use_separate_channels) };
        if delegate.is_null() {
            return Err(ZoomRsError::NullPtr);
        }
        Ok(Self {
            evt_mutex,
            ref_delegate: unsafe { delegate.as_mut() }.unwrap(),
        })
    }
    fn flush(&mut self) {
        self.evt_mutex.as_ref().unwrap().lock().unwrap().flush();
    }
}

// #[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_mixed_audio_raw_data(ptr: *const u8, data: *const exported_audio_raw_data) -> i32 {
    if data.is_null() {
        tracing::warn!("Null pointer detected!");
        0
    } else {
        (*convert(ptr).lock().unwrap()).on_mixed_audio_raw_data(unsafe { data.as_ref() }.unwrap())
    }
}

// #[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_one_way_audio_raw_data(
    ptr: *const u8,
    data: *const exported_audio_raw_data,
    user_id: __uint32_t,
) -> i32 {
    if data.is_null() {
        tracing::warn!("Null pointer detected!");
        0
    } else {
        (*convert(ptr).lock().unwrap())
            .on_one_way_audio_raw_data(unsafe { data.as_ref() }.unwrap(), user_id)
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_share_audio_raw_data(ptr: *const u8, data: *const exported_audio_raw_data) -> i32 {
    if data.is_null() {
        tracing::warn!("Null pointer detected!");
        0
    } else {
        (*convert(ptr).lock().unwrap()).on_share_audio_raw_data(unsafe { data.as_ref() }.unwrap())
    }
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn RawAudioEvent>>> {
    let ptr: *const Mutex<Box<dyn RawAudioEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}
