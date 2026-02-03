use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomSdkResult};

/// This trait handles events related to recording.
pub trait RecordingControllerEvent: std::fmt::Debug {
    /// Callback event that the status of request local recording privilege.
    /// - [RequestLocalRecordingStatus] of request local recording privilege status.
    fn on_recording_privilege_request_status(&mut self, _status: RequestLocalRecordingStatus) {}

    /// Callback event that the status of my local recording changes.
    /// - [RecordingStatus] of recording status. For more details, see \link RecordingStatus \endlink enum.
    fn on_recording_status(&mut self, _status: RecordingStatus, _time: i64) {}

    /// Callback event that the recording authority changes.
    /// - [bool] TRUE indicates to enable to record.
    fn on_recording_privilege_changed(&mut self, _can_rec: bool) {}
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_recording_privilege_request_status(
    ptr: *const u8,
    status: ZOOMSDK_RequestLocalRecordingStatus,
) {
    tracing::info!("Entering on_recording_privilege_request_status");
    (*convert(ptr).try_lock().unwrap()).on_recording_privilege_request_status(status.into());
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_recording_status(ptr: *const u8, status: ZOOMSDK_RecordingStatus, time: i64) {
    tracing::info!("Entering on_recording_status");
    (*convert(ptr).lock().unwrap()).on_recording_status(status.into(), time);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_recording_privilege_changed(ptr: *const u8, can_rec: bool) {
    tracing::info!("Entering on_recording_privilege_changed");
    (*convert(ptr).lock().unwrap()).on_recording_privilege_changed(can_rec);
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn RecordingControllerEvent>>> {
    let ptr: *const Mutex<Box<dyn RecordingControllerEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Main interface of [RecordingController].
#[derive(Debug)]
pub struct RecordingController<'a> {
    ref_recording_controller: &'a mut ZOOMSDK_IMeetingRecordingController,
    evt_mutex: Option<Arc<Mutex<Box<dyn RecordingControllerEvent>>>>,
}

impl<'a> RecordingController<'a> {
    /// Get the participants interface.
    /// - If the function succeeds, the return value is [RecordingController]. Otherwise returns None.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_recording_controller(meeting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_recording_controller: unsafe { ptr.as_mut() }.unwrap(),
                evt_mutex: None,
            })
        }
    }

    /// Set the recording controller callback event handler.  
    /// - [RecordingControllerEvent] A pointer to receive recording event.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn set_event(&mut self, ctx: Box<dyn RecordingControllerEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        tracing::info!("{:?}", ptr);
        ZoomSdkResult(
            unsafe { recording_set_event(self.ref_recording_controller, ptr) },
            (),
        )
        .into()
    }
    /// Send a request to enable the SDK to start local recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn request_local_recording_privilege(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_request_local_recording_privilege(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Send a request to ask the host to start cloud recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn request_start_cloud_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_request_start_cloud_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Start recording.
    /// - [time_t] startTimestamp The timestamps when start recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn start_recording(&mut self, time: &mut time_t) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_start_recording(self.ref_recording_controller, time) },
            (),
        )
        .into()
    }
    /// Stop recording.
    /// - [time_t] stopTimestamp The timestamps when stop recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn stop_recording(&mut self, time: &mut time_t) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_stop_recording(self.ref_recording_controller, time) },
            (),
        )
        .into()
    }
    /// Ask if raw recording is possible.
    /// - If the function succeeds (SDKErr_Success), raw recording can be started.
    /// - If the function fails, raw recording cannot be started, see [crate::SdkError] for details.
    pub fn can_start_raw_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_can_start_raw_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Start raw recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn start_raw_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_start_raw_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Stop raw recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn stop_raw_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_stop_raw_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Pause recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details
    pub fn pause_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_pause_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
    /// Resume recording.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details
    pub fn resume_recording(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { recording_resume_recording(self.ref_recording_controller) },
            (),
        )
        .into()
    }
}

/// Request local recording privilege status.
///
/// Here are more detailed structural descriptions.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestLocalRecordingStatus {
    /// Host granted the request.
    RequestLocalRecordingGranted =
        ZOOMSDK_RequestLocalRecordingStatus_RequestLocalRecording_Granted,
    /// Host denied the request.
    RequestLocalRecordingDenied = ZOOMSDK_RequestLocalRecordingStatus_RequestLocalRecording_Denied,
    /// Request timed out.
    RequestLocalRecordingTimeout =
        ZOOMSDK_RequestLocalRecordingStatus_RequestLocalRecording_Timeout,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_RequestLocalRecordingStatus_RequestLocalRecording_Timeout + 1,
}

impl From<u32> for RequestLocalRecordingStatus {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_RequestLocalRecordingStatus_RequestLocalRecording_Timeout => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// Recording status.
///
/// Here are more detailed structural descriptions.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStatus {
    /// Start recording on local computer or on cloud.
    RecordingStart = ZOOMSDK_RecordingStatus_Recording_Start,
    /// Stop recording on local computer or on cloud.
    RecordingStop = ZOOMSDK_RecordingStatus_Recording_Stop,
    /// There is no more space to store both local and cloud recording.
    RecordingDiskFull = ZOOMSDK_RecordingStatus_Recording_DiskFull,
    /// Pause recording on local or on cloud.
    RecordingPause = ZOOMSDK_RecordingStatus_Recording_Pause,
    /// Connecting, only for cloud recording.
    RecordingConnecting = ZOOMSDK_RecordingStatus_Recording_Connecting,
    /// Recording failed.
    RecordingFail = ZOOMSDK_RecordingStatus_Recording_Fail,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_RecordingStatus_Recording_Fail + 1,
}

impl From<u32> for RecordingStatus {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_RecordingStatus_Recording_Fail => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}
