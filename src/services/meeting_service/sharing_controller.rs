use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomSdkResult};

/// Information about the sharing.
pub type ShareInfo = ZOOMSDK_ShareInfo;
/// Obscure pointer of the SDK.
pub type ShareSwitchMultitoSingleConfirmHandler = ZOOMSDK_IShareSwitchMultiToSingleConfirmHandler;

/// This trait handles events related to sharing.
pub trait SharingControllerEvent: std::fmt::Debug {
    /// Callback event of the changed sharing status.
    /// - status The values of sharing status. For more details, see \link SharingStatus \endlink enum.
    /// - [u32] Sharer ID.
    /// The userId changes according to the status value. When the status value is
    /// the Sharing_Self_Send_Begin or Sharing_Self_Send_End, the userId is the user own ID.
    /// Otherwise, the value of userId is the sharer ID.
    fn on_sharing_status(&mut self, _status: SharingStatus, _user_id: u32) {}

    /// Callback event of locked share status.
    /// - [bool] TRUE indicates that it is locked. FALSE unlocked.
    fn on_lock_share_status(&mut self, _locked: bool) {}

    /// Callback event of changed sharing information.
    /// - [ShareInfo] Sharing information.
    fn on_share_content_notification(&mut self, _share_info: &ShareInfo) {}

    /// Callback event of switching multi-participants share to one participant share.
    /// - [ShareSwitchMultitoSingleConfirmHandler] An object pointer used by user to complete
    /// all the related operations.
    fn on_multi_share_switch_to_single_share_need_confirm(
        &mut self,
        _handler: &ShareSwitchMultitoSingleConfirmHandler,
    ) {
    }

    /// Callback event of sharing setting type changed.
    /// [SharingSettingType] type Sharing setting type. For more details.
    fn on_share_setting_type_changed_notification(&mut self, _kind: SharingSettingType) {}

    /// Callback event of the shared video's playback has completed.
    fn on_shared_video_ended(&mut self) {}

    /// Callback event of the video file playback error.
    /// - [SharingPlayError] The error type. For more details,
    fn on_video_file_share_play_error(&mut self, _error: SharingPlayError) {}
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_sharing_status(ptr: *const u8, status: ZOOMSDK_SharingStatus, user_id: u32) {
    tracing::debug!("Entering on_sharing_status");
    (*convert(ptr).try_lock().unwrap()).on_sharing_status(status.into(), user_id);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_lock_share_status(ptr: *const u8, locked: bool) {
    (*convert(ptr).lock().unwrap()).on_lock_share_status(locked);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_share_content_notification(ptr: *const u8, share_info: *const ZOOMSDK_ShareInfo) {
    if share_info.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        (*convert(ptr).lock().unwrap())
            .on_share_content_notification(unsafe { share_info.as_ref() }.unwrap());
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_multi_share_switch_to_single_share_need_confirm(
    ptr: *const u8,
    handler: *mut ZOOMSDK_IShareSwitchMultiToSingleConfirmHandler,
) {
    if handler.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        (*convert(ptr).lock().unwrap()).on_multi_share_switch_to_single_share_need_confirm(
            unsafe { handler.as_ref() }.unwrap(),
        );
    }
}
#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_share_setting_type_changed_notification(
    ptr: *const u8,
    kind: ZOOMSDK_ShareSettingType,
) {
    (*convert(ptr).try_lock().unwrap()).on_share_setting_type_changed_notification(kind.into());
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_shared_video_ended(ptr: *const u8) {
    (*convert(ptr).try_lock().unwrap()).on_shared_video_ended();
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_video_file_share_play_error(
    ptr: *const u8,
    error: ZOOMSDK_ZoomSDKVideoFileSharePlayError,
) {
    tracing::debug!("Entering on_sharing_status");
    (*convert(ptr).try_lock().unwrap()).on_video_file_share_play_error(error.into());
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn SharingControllerEvent>>> {
    let ptr: *const Mutex<Box<dyn SharingControllerEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Main interface of [SharingController].
#[derive(Debug)]
pub struct SharingController<'a> {
    ref_sharing_controller: &'a mut ZOOMSDK_IMeetingShareController,
    evt_mutex: Option<Arc<Mutex<Box<dyn SharingControllerEvent>>>>,
}

impl<'a> SharingController<'a> {
    /// Get the sharing interface.
    /// - If the function succeeds, the return value is [SharingController]. Otherwise returns None.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_share_controller(meeting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_sharing_controller: unsafe { ptr.as_mut() }.unwrap(),
                evt_mutex: None,
            })
        }
    }

    /// Set the sharing controller callback event handler.
    /// - [SharingControllerEvent] A pointer to receive sharing event.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn set_event(&mut self, ctx: Box<dyn SharingControllerEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        tracing::info!("{:?}", ptr);
        ZoomSdkResult(
            unsafe { sharing_set_event(self.ref_sharing_controller, ptr) },
            (),
        )
        .into()
    }
}

/// This enumeration describes all the events related to sharing.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SharingStatus {
    /// Begin to share by the user himself.
    SharingSelfSendBegin = ZOOMSDK_SharingStatus_Sharing_Self_Send_Begin,
    /// top sharing by the user.
    SharingSelfSendEnd = ZOOMSDK_SharingStatus_Sharing_Self_Send_End,
    /// Begin to share pure audio by the user himself.
    SharingSelfSendPureAudioBegin = ZOOMSDK_SharingStatus_Sharing_Self_Send_Pure_Audio_Begin,
    /// Stop sharing pure audio by the user.
    SharingSelfSendPureAudioEnd = ZOOMSDK_SharingStatus_Sharing_Self_Send_Pure_Audio_End,
    /// Others begin to share.
    SharingOtherShareBegin = ZOOMSDK_SharingStatus_Sharing_Other_Share_Begin,
    /// Others stop sharing.
    SharingOtherShareEnd = ZOOMSDK_SharingStatus_Sharing_Other_Share_End,
    /// Others begin to share pure audio.
    SharingOtherSendPureAudioBegin = ZOOMSDK_SharingStatus_Sharing_Other_Share_Pure_Audio_Begin,
    /// Others stop sharing pure audio.
    SharingOtherSendPureAudioEnd = ZOOMSDK_SharingStatus_Sharing_Other_Share_Pure_Audio_End,
    /// iew the sharing of others.
    SharingViewOtherSharing = ZOOMSDK_SharingStatus_Sharing_View_Other_Sharing,
    /// Pause sharing.
    SharingPause = ZOOMSDK_SharingStatus_Sharing_Pause,
    /// Resume sharing.
    SharingResume = ZOOMSDK_SharingStatus_Sharing_Resume,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_SharingStatus_Sharing_Resume + 1,
}

impl From<u32> for SharingStatus {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_SharingStatus_Sharing_Resume => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// This enumeration describes all configuration related to sharing.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SharingSettingType {
    /// Only host can share, the same as "lock share".
    LockShare = ZOOMSDK_ShareSettingType_ShareSettingType_LOCK_SHARE,
    /// Anyone can share, but one sharing only at one moment, and only host can grab other's sharing.
    HostGrab = ZOOMSDK_ShareSettingType_ShareSettingType_HOST_GRAB,
    /// Anyone can share, but one sharing only at one moment, and anyone can grab other's sharing.
    AnyoneGrab = ZOOMSDK_ShareSettingType_ShareSettingType_ANYONE_GRAB,
    /// nyone can share, Multi-share can exist at the same time.
    MultiShare = ZOOMSDK_ShareSettingType_ShareSettingType_MULTI_SHARE,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_ShareSettingType_ShareSettingType_MULTI_SHARE + 1,
}

impl From<u32> for SharingSettingType {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_ShareSettingType_ShareSettingType_MULTI_SHARE => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// This enumeration describes all errors related to sharing.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SharingPlayError {
    /// Only host can share, the same as "lock share".
    None = ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_None,
    /// Not supported..
    NotSupported =
        ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Not_Supported,
    /// The resolution is too high to play.
    ResolutionTooHigh =
        ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Resolution_Too_High,
    /// Failed to open.
    OpenFail = ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Open_Fail,
    /// Failed to play.
    PlayFail = ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Play_Fail,
    /// Failed to seek.
    SeekFail = ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Seek_Fail,
    /// Unexpected result from SDK
    Unexpected =
        ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Seek_Fail + 1,
}

impl From<u32> for SharingPlayError {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_ZoomSDKVideoFileSharePlayError_ZoomSDKVideoFileSharePlayError_Seek_Fail => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}
