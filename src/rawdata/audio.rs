use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// Rust type definition.
pub type ExportedAudioRawData = exported_audio_raw_data;

#[derive(Debug, Clone)]
/// This structure represents the ZOOM SDK audio sender.
pub struct AudioRawDataSenderInterface(*mut ZOOMSDK_IZoomSDKAudioRawDataSender);

unsafe impl Send for AudioRawDataSenderInterface {}

impl AudioRawDataSenderInterface {
    /// Send audio raw data. Audio sample must be 16-bit audio.
    /// - &[u8] the audio datas address.
    /// - [usize] sample_rate the audio datas sampling rate.
    /// Supported sample rates: 8000/11025/16000/32000/44100/48000/50000/50400/96000/192000/2822400
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn send(&mut self, data: &[u8], sample_rate: usize) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe {
                send_audio_raw_data(
                    self.0,
                    data.as_ptr() as *mut i8,
                    data.len() as _,
                    sample_rate as i32,
                )
            },
            (),
        )
        .into()
    }
}

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

/// VirtualAudioMicEvent
pub trait VirtualAudioMicEvent: Debug {
    /// Callback for virtual audio mic to do some initialization.
    fn on_mic_initialize(&mut self, sender: AudioRawDataSenderInterface);
    /// Callback for virtual audio mic can send raw data with 'pSender'.
    fn on_mic_start_send(&mut self);
    /// Callback for virtual audio mic should stop send raw data.
    fn on_mic_stop_send(&mut self);
    /// Callback for virtual audio mic is uninitialized.
    fn on_mic_uninitialized(&mut self);
}

/// Audio RawData Helper.
#[derive(Debug)]
pub struct AudioRawDataHelper<'a> {
    ref_rawdata_helper: &'a mut ZOOMSDK_IZoomSDKAudioRawDataHelper,
    delegate: Option<RawAudioDelegate<'a>>,
    evt_mic_event_mutex: Option<Arc<Mutex<Box<dyn VirtualAudioMicEvent>>>>,
}

impl<'a> AudioRawDataHelper<'a> {
    /// Create a new RawData helper.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn new() -> SdkResult<Self> {
        let ptr = unsafe { ZOOMSDK_GetAudioRawdataHelper() };
        if ptr.is_null() {
            return Err(ZoomRsError::NullPtr);
        }
        Ok(Self {
            ref_rawdata_helper: unsafe { ptr.as_mut() }.unwrap(),
            delegate: None,
            evt_mic_event_mutex: None,
        })
    }
    /// Subscribe raw audio data.
    /// - [RawAudioEvent], the callback handler of raw audio data.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
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
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
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
    /// \Subscribe audio mic raw data with a callback.
    /// - [VirtualAudioMicEvent], the callback handler of raw audio data.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn set_external_audio_source(
        &mut self,
        arc_event: Arc<Mutex<Box<dyn VirtualAudioMicEvent>>>,
    ) -> SdkResult<()> {
        let ptr = Arc::as_ptr(&arc_event) as *mut _;
        self.evt_mic_event_mutex = Some(arc_event);
        ZoomSdkResult(
            unsafe { audio_helper_set_external_audio_source(self.ref_rawdata_helper, ptr) },
            (),
        )
        .into()
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

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_mic_initialize(ptr: *const u8, sender: *mut ZOOMSDK_IZoomSDKAudioRawDataSender) {
    if sender.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        (*convert_n(ptr).lock().unwrap()).on_mic_initialize(AudioRawDataSenderInterface(sender))
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_mic_start_send(ptr: *const u8) {
    (*convert_n(ptr).lock().unwrap()).on_mic_start_send()
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_mic_stop_send(ptr: *const u8) {
    (*convert_n(ptr).lock().unwrap()).on_mic_stop_send()
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_mic_uninitialized(ptr: *const u8) {
    (*convert_n(ptr).lock().unwrap()).on_mic_uninitialized()
}

#[inline]
fn convert_n(ptr: *const u8) -> Arc<Mutex<Box<dyn VirtualAudioMicEvent>>> {
    let ptr: *const Mutex<Box<dyn VirtualAudioMicEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}
