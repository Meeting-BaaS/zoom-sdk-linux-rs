use std::ffi::{c_int, CStr};
use std::fmt::Debug;
use std::ptr;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use crate::SdkError;
use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// Sub-service for controlling audio
pub mod audio_controller;
/// Allows sending messages.
pub mod chat_interface;
/// Allows obtaining information about the participants.
pub mod participants_interface;
/// Allows managing, starting, and stopping the recording.
pub mod recording_controller;
/// Allows obtaining the events necessary for screen sharing.
pub mod sharing_controller;
/// Allows injecting an image into the bot webcam.
pub mod webcam_interface;

pub use audio_controller::AudioController;
pub use chat_interface::ChatInterface;
pub use participants_interface::{ParticipantsEvent, ParticipantsInterface};
pub use recording_controller::RecordingController;
pub use sharing_controller::SharingController;
pub use webcam_interface::{new_webcam_injection_boitlerplate, VideoToWebcam};

/// Main instance of the meeting.
#[derive(Debug)]
pub struct MeetingService<'a> {
    // Inner stuff
    evt_mutex: Option<Arc<Mutex<Box<dyn MeetingServiceEvent>>>>,
    ref_meeting_service: &'a mut ZOOMSDK_IMeetingService,

    // Natural borrow
    recording_controller: Option<RecordingController<'a>>,
    participants_interface: Option<ParticipantsInterface<'a>>,
    chat_interface: Option<ChatInterface<'a>>,
    sharing_controller: Option<SharingController<'a>>,
    audio_controller: Option<AudioController<'a>>,

    // Exception Class II
    camera_mutex: Option<Arc<Mutex<Box<dyn VideoToWebcam>>>>,
}

/// This trait handles all events related to the meeting
pub trait MeetingServiceEvent: Debug {
    /// Meeting status changed callback.
    /// - [MeetingStatus] The value of meeting. For more details.  
    /// - [i32] Detailed reasons for special meeting status.  
    /// - If the status is [MeetingStatus::MeetingStatusFailed], the value of result is one of those listed in MeetingFailCode enum.  
    /// - If the status is [MeetingStatus::MeetingStatusEnded], the value of result is one of those listed in MeetingEndReason.
    fn on_meeting_status_changed(&mut self, _status: MeetingStatus, _result: MeetingFailCode) {}

    /// Meeting statistics warning notification callback.  
    /// - [StatisticsWarningType] The warning type of the meeting statistics.
    fn on_meeting_statistics_warning_notification(&mut self, _warn_type: StatisticsWarningType) {}

    /// Meeting parameter notification callback.  
    /// - [MeetingParameter] Meeting parameter.  
    /// - NOTE : The callback will be triggered right before the meeting starts. The meeting_param will be destroyed once the function calls end.
    fn on_meeting_parameter_notification(&mut self, _meeting_param: &MeetingParameter) {}

    /// Callback event when a meeting is suspended.
    fn on_suspend_participants_activities(&mut self) {}

    /// Callback event for the AI Companion active status changed.  
    /// - [bool] active Specify whether the AI Companion active or not.
    fn on_ai_companion_active_change_notice(&mut self, _active: bool) {}

    /// Callback event for the meeting topic changed.  
    /// - [str] topic The new meeting topic.
    fn on_meeting_topic_changed(&mut self, _topic: &str) {}

    /// Calback event that the meeting users have reached the meeting capacity.  
    /// The new join user can not join meeting, but they can watch the meeting live stream.  
    /// - [str] The live stream URL to watch the meeting live stream.
    fn on_meeting_full_to_watch_live_stream(&mut self, _s_live_stream_url: &str) {}
}

impl<'a> MeetingService<'a> {
    /// Create meeting service interface
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [SdkError] for details.
    pub fn new() -> SdkResult<Self> {
        let mut ptr = ptr::null_mut();
        let ret = unsafe { ZOOMSDK_CreateMeetingService(&mut ptr) };
        if ret == ZOOMSDK_SDKError_SDKERR_SUCCESS {
            Ok(MeetingService {
                evt_mutex: None,
                ref_meeting_service: unsafe { ptr.as_mut() }.unwrap(),
                recording_controller: None,
                sharing_controller: None,
                participants_interface: None,
                chat_interface: None,
                camera_mutex: None,
                audio_controller: None,
            })
        } else {
            Err(ZoomRsError::Sdk(ret.into()))
        }
    }
    /// Set meeting service callback event handler.  
    /// - [MeetingServiceEvent] A pointer to the IMeetingServiceEvent that receives the meeting service callback event.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.   
    pub fn set_event(&mut self, ctx: Box<dyn MeetingServiceEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        ZoomSdkResult(
            unsafe { meeting_set_event(self.ref_meeting_service, ptr) },
            (),
        )
        .into()
    }
    /// Join the meeting.  
    /// - [JoinParam] The parameter is used to join meeting.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.   
    pub fn join(&mut self, join_params: JoinParam) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe {
                meeting_join(
                    self.ref_meeting_service,
                    join_params.meeting_id.map(|id| id as u64).unwrap_or(0),
                    join_params.vanity_id.map(|v| v.as_ptr()).unwrap_or(std::ptr::null()) as *mut _,
                    join_params.username.as_ptr() as *mut _,
                    match join_params.password {
                        Some(ptr) => ptr.as_ptr(),
                        None => CStr::from_bytes_with_nul_unchecked(b"\0").as_ptr(),
                    } as *mut _,
                    match join_params.zoom_access_token {
                        Some(ptr) => ptr.as_ptr(),
                        None => std::ptr::null(),
                    } as *mut _,
                    match join_params.on_behalf_token {
                        Some(ptr) => ptr.as_ptr(),
                        None => std::ptr::null(),
                    } as *mut _,
                )
            },
            (),
        )
        .into()
    }
    /// Leave meeting.  
    /// - [LeaveMeetingCmd] Leave meeting command.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.   
    pub fn leave(&mut self, leave_meeting_cmd: LeaveMeetingCmd) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { meeting_leave(self.ref_meeting_service, leave_meeting_cmd as u32) },
            (),
        )
        .into()
    }
    /// Get Chat Interface.
    pub fn chat(&mut self) -> &mut ChatInterface<'a> {
        if self.chat_interface.is_none() {
            self.chat_interface = Some(ChatInterface::new(self.ref_meeting_service).unwrap());
            self.chat_interface
                .as_ref()
                .expect("Cannot create ChatInterface");
        }
        self.chat_interface.as_mut().unwrap()
    }
    /// Get Participants Interface.
    pub fn participants(&mut self) -> &mut ParticipantsInterface<'a> {
        if self.participants_interface.is_none() {
            self.participants_interface =
                Some(ParticipantsInterface::new(self.ref_meeting_service).unwrap());
            self.participants_interface
                .as_ref()
                .expect("Cannot create ParticipantsInterface");
        }
        self.participants_interface.as_mut().unwrap()
    }
    /// Get Recording Controller.
    pub fn recording_ctrl(&mut self) -> &mut RecordingController<'a> {
        if self.recording_controller.is_none() {
            self.recording_controller =
                Some(RecordingController::new(self.ref_meeting_service).unwrap());
            self.recording_controller
                .as_ref()
                .expect("Cannot create RecordingController");
        }
        self.recording_controller.as_mut().unwrap()
    }
    /// Get Sharing Controller.
    pub fn sharing_ctrl(&mut self) -> &mut SharingController<'a> {
        if self.sharing_controller.is_none() {
            self.sharing_controller =
                Some(SharingController::new(self.ref_meeting_service).unwrap());
            self.sharing_controller
                .as_ref()
                .expect("Cannot create SharingController");
        }
        self.sharing_controller.as_mut().unwrap()
    }
    /// Get Audio Controller.
    pub fn audio_ctrl(&mut self) -> &mut AudioController<'a> {
        if self.audio_controller.is_none() {
            self.audio_controller = Some(AudioController::new(self.ref_meeting_service).unwrap());
            self.audio_controller
                .as_ref()
                .expect("Cannot create AudioController");
        }
        self.audio_controller.as_mut().unwrap()
    }
    /// Initialize WebCam Injection.
    pub fn set_webcam_injection(&mut self, ctx: Option<Box<dyn VideoToWebcam>>) -> SdkResult<()> {
        match ctx {
            None => {
                // TODO : Check is CAM is always OFF.
                Ok(())
            }
            Some(ctx) => {
                if let Some(camera_mutex) =
                    new_webcam_injection_boitlerplate(self.ref_meeting_service, ctx)
                {
                    self.camera_mutex = Some(camera_mutex);
                    Ok(())
                } else {
                    Err(ZoomRsError::NullPtr)
                }
            }
        }
    }
}

impl<'a> Drop for MeetingService<'a> {
    fn drop(&mut self) {
        if crate::is_sdk_tearing_down() {
            tracing::info!("MeetingService drop: skipping DestroyMeetingService (SDK is tearing down)");
            return;
        }
        let ret = unsafe { ZOOMSDK_DestroyMeetingService(self.ref_meeting_service) };
        if ret != ZOOMSDK_SDKError_SDKERR_SUCCESS {
            tracing::warn!("Error when droping MeetingService : {:?}", ret);
        } else {
            tracing::info!("Meeting instance droped!");
        }
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_meeting_status_changed(
    ptr: *const u8,
    status: ZOOMSDK_MeetingStatus,
    result: c_int,
) {
    // When the SDK fires MeetingStatusDisconnecting, it will immediately begin
    // internal teardown (freeing renderer, audio, etc.) after this callback returns.
    // Set the global teardown flag NOW — on the SDK thread, before returning —
    // so that the glib main-loop tick handler and Drop impls skip SDK object access.
    if status == ZOOMSDK_MeetingStatus_MEETING_STATUS_DISCONNECTING {
        crate::mark_sdk_teardown();
    }

    let result: MeetingFailCode = match (result as u32).try_into() {
        Ok(fail_code) => fail_code,
        Err(e) => {
            tracing::error!("{}", e);
            MeetingFailCode::Unknown
        }
    };
    (*convert(ptr).lock().unwrap()).on_meeting_status_changed(status.into(), result);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_meeting_statistics_warning_notification(
    ptr: *const u8,
    warn_type: ZOOMSDK_StatisticsWarningType,
) {
    (*convert(ptr).lock().unwrap()).on_meeting_statistics_warning_notification(warn_type.into());
}

#[tracing::instrument]
#[no_mangle]
extern "C" fn on_meeting_parameter_notification(
    ptr: *const u8,
    meeting_param: *const ZOOMSDK_MeetingParameter,
) {
    if meeting_param.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        #[repr(C)]
        struct RawMeetingParameter {
            pub meeting_type: u32,             // Offset 0
            pub is_view_only: bool,            // Offset 4
            pub is_auto_recording_local: bool, // Offset 5
            pub is_auto_recording_cloud: bool, // Offset 6
            pub meeting_number: u64,           // Offset 8
            pub meeting_topic: *const u8,      // Offset 16 (C string pointer)
            pub meeting_host: *const u8,       // Offset 24 (C string pointer)
        }
        let raw: *const RawMeetingParameter = meeting_param as *const _;
        let meeting_param = unsafe {
            MeetingParameter {
                meeting_type: (*raw).meeting_type,
                is_view_only: (*raw).is_view_only,
                is_auto_recording_local: (*raw).is_auto_recording_local,
                is_auto_recording_cloud: (*raw).is_auto_recording_cloud,
                meeting_number: (*raw).meeting_number,
                meeting_topic: if (*raw).meeting_topic.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr((*raw).meeting_topic as _).to_str().unwrap())
                },
                meeting_host: if (*raw).meeting_host.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr((*raw).meeting_host as _).to_str().unwrap())
                },
            }
        };
        (*convert(ptr).lock().unwrap()).on_meeting_parameter_notification(&meeting_param);
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_suspend_participants_activities(ptr: *const u8) {
    (*convert(ptr).lock().unwrap()).on_suspend_participants_activities();
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_ai_companion_active_change_notice(ptr: *const u8, b_active: c_int) {
    (*convert(ptr).lock().unwrap()).on_ai_companion_active_change_notice(b_active != 0);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_meeting_topic_changed(ptr: *const u8, topic: *const zchar_t) {
    if topic.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        let s = unsafe { CStr::from_ptr(topic) }.to_str().unwrap();
        (*convert(ptr).lock().unwrap()).on_meeting_topic_changed(&s);
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_meeting_full_to_watch_live_stream(
    ptr: *const u8,
    s_live_stream_url: *const zchar_t,
) {
    if s_live_stream_url.is_null() {
        tracing::warn!("Null pointer detected!");
    } else {
        let s = unsafe { CStr::from_ptr(s_live_stream_url) }
            .to_str()
            .unwrap();
        (*convert(ptr).lock().unwrap()).on_meeting_full_to_watch_live_stream(&s);
    }
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn MeetingServiceEvent>>> {
    let ptr: *const Mutex<Box<dyn MeetingServiceEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Meeting status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MeetingStatus {
    /// No meeting is running.
    MeetingStatusIdle = ZOOMSDK_MeetingStatus_MEETING_STATUS_IDLE,
    /// Connect to the meeting server status.
    MeetingStatusConnecting = ZOOMSDK_MeetingStatus_MEETING_STATUS_CONNECTING,
    /// Waiting for the host to start the meeting.
    MeetingStatusWaitingForHost = ZOOMSDK_MeetingStatus_MEETING_STATUS_WAITINGFORHOST,
    /// Meeting is ready, in meeting status.
    MeetingStatusInMeeting = ZOOMSDK_MeetingStatus_MEETING_STATUS_INMEETING,
    /// Disconnect the meeting server, leave meeting status.
    MeetingStatusDisconnecting = ZOOMSDK_MeetingStatus_MEETING_STATUS_DISCONNECTING,
    /// Reconnecting meeting server status.
    MeetingStatusReconnecting = ZOOMSDK_MeetingStatus_MEETING_STATUS_RECONNECTING,
    /// Failed to connect the meeting server.
    MeetingStatusFailed = ZOOMSDK_MeetingStatus_MEETING_STATUS_FAILED,
    /// Meeting ends.
    MeetingStatusEnded = ZOOMSDK_MeetingStatus_MEETING_STATUS_ENDED,
    /// Unknown status.
    MeetingStatusUnknown = ZOOMSDK_MeetingStatus_MEETING_STATUS_UNKNOWN,
    /// Meeting is locked to prevent the further participants to join the meeting.
    MeetingStatusLocked = ZOOMSDK_MeetingStatus_MEETING_STATUS_LOCKED,
    /// Meeting is open and participants can join the meeting.
    MeetingStatusUnlocked = ZOOMSDK_MeetingStatus_MEETING_STATUS_UNLOCKED,
    /// Participants who join the meeting before the start are in the waiting room.
    MeetingStatusInWaitingRoom = ZOOMSDK_MeetingStatus_MEETING_STATUS_IN_WAITING_ROOM,
    /// Upgrade the attendees to panelist in webinar.
    MeetingStatusWebinarPromote = ZOOMSDK_MeetingStatus_MEETING_STATUS_WEBINAR_PROMOTE,
    /// Downgrade the attendees from the panelist.
    MeetingStatusWebinarDepromote = ZOOMSDK_MeetingStatus_MEETING_STATUS_WEBINAR_DEPROMOTE,
    /// Join the breakout room.
    MeetingStatusJoinBreakoutRoom = ZOOMSDK_MeetingStatus_MEETING_STATUS_JOIN_BREAKOUT_ROOM,
    /// Leave the breakout room.
    MeetingStatusLeaveBreakoutRoom = ZOOMSDK_MeetingStatus_MEETING_STATUS_LEAVE_BREAKOUT_ROOM,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_MeetingStatus_MEETING_STATUS_LEAVE_BREAKOUT_ROOM + 1,
}

impl From<u32> for MeetingStatus {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_MeetingStatus_MEETING_STATUS_LEAVE_BREAKOUT_ROOM => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// Meeting statistics warning type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StatisticsWarningType {
    /// No warning.
    WarningNone = ZOOMSDK_StatisticsWarningType_Statistics_Warning_None,
    /// The network connection quality is bad.
    WarningNetworkQualityBad = ZOOMSDK_StatisticsWarningType_Statistics_Warning_Network_Quality_Bad,
    /// The system is busy.
    WarningBusySystem = ZOOMSDK_StatisticsWarningType_Statistics_Warning_Busy_System,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_StatisticsWarningType_Statistics_Warning_Busy_System + 1,
}

impl From<u32> for StatisticsWarningType {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_StatisticsWarningType_Statistics_Warning_Busy_System => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// Leave meeting command.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveMeetingCmd {
    /// Leave meeting.
    LeaveMeeting = ZOOMSDK_LeaveMeetingCmd_LEAVE_MEETING,
    /// End meeting.
    EndMeeting = ZOOMSDK_LeaveMeetingCmd_END_MEETING,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_LeaveMeetingCmd_END_MEETING + 1,
}

impl From<u32> for LeaveMeetingCmd {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_LeaveMeetingCmd_END_MEETING => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// Meeting parameter.
#[derive(Debug, Clone)]
pub struct MeetingParameter<'a> {
    /// Meeting type.
    pub meeting_type: u32,
    /// View only or not. True indicates to view only.
    pub is_view_only: bool,
    /// Auto local recording or not. True indicates to auto local recording.
    pub is_auto_recording_local: bool,
    /// Auto cloud recording or not. True indicates to auto cloud recording.
    pub is_auto_recording_cloud: bool,
    /// Meeting number.
    pub meeting_number: u64,
    /// Meeting topic.
    pub meeting_topic: Option<&'a str>,
    /// Meeting host.
    pub meeting_host: Option<&'a str>,
}

/// The parameters of non-login user when joins the meeting.
#[derive(Debug, Clone)]
pub struct JoinParam<'a> {
    /// Meeting number (for numeric meeting IDs)
    pub meeting_id: Option<usize>,
    /// Vanity ID (for Personal Meeting Room URLs, e.g., "audiencelab")
    pub vanity_id: Option<&'a CStr>,
    /// Username when logged in the meeting.
    pub username: &'a CStr,
    /// Meeting password.
    pub password: Option<&'a CStr>,
    /// Zoom access token.
    pub zoom_access_token: Option<&'a CStr>,
    /// On Behalf Of token for OBF authorization.
    pub on_behalf_token: Option<&'a CStr>,
}

/// Meeting failure code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MeetingFailCode {
    /// Start meeting successfully
    Success = ZOOMSDK_MeetingFailCode_MEETING_SUCCESS,
    /// Connection error
    ConnectionError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_CONNECTION_ERR,
    /// Reconnect error
    ReconnectError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_RECONNECT_ERR,
    /// Multi-media Router error
    MMRError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MMR_ERR,
    /// Password is wrong
    PasswordError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_PASSWORD_ERR,
    /// Session error
    SessionError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_SESSION_ERR,
    /// Meeting is over
    MeetingOver = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_OVER,
    /// Meeting has not begun
    MeetingNotStarted = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_START,
    /// Meeting does not exist
    MeetingNotExist = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_EXIST,
    /// The capacity of meeting is full
    MeetingUserFull = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_USER_FULL,
    /// The client is incompatible
    ClientIncompatible = ZOOMSDK_MeetingFailCode_MEETING_FAIL_CLIENT_INCOMPATIBLE,
    /// The Multi-media router is not found
    NoMMR = ZOOMSDK_MeetingFailCode_MEETING_FAIL_NO_MMR,
    /// The meeting is locked
    MeetingLocked = ZOOMSDK_MeetingFailCode_MEETING_FAIL_CONFLOCKED,
    /// Meeting restricted by same account
    MeetingRestricted = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED,
    /// Meeting restricted (JBH - Join Before Host)
    MeetingRestrictedJBH = ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED_JBH,
    /// Unable to send web request
    CannotEmitWebRequest = ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_EMIT_WEBREQUEST,
    /// Token expired
    TokenExpired = ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_START_TOKENEXPIRE,
    /// Video hardware/software error
    VideoError = ZOOMSDK_MeetingFailCode_SESSION_VIDEO_ERR,
    /// Audio autostart error
    AudioAutoStartError = ZOOMSDK_MeetingFailCode_SESSION_AUDIO_AUTOSTARTERR,
    /// Webinar registration full
    WebinarRegisterFull = ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_FULL,
    /// Webinar host registration
    WebinarHostRegister = ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_HOSTREGISTER,
    /// Webinar panelist registration
    WebinarPanelistRegister = ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_PANELISTREGISTER,
    /// Webinar denied email
    WebinarDeniedEmail = ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_DENIED_EMAIL,
    /// Login required
    EnforceLogin = ZOOMSDK_MeetingFailCode_MEETING_FAIL_ENFORCE_LOGIN,
    /// Certificate changed
    CertificateChanged = ZOOMSDK_MeetingFailCode_CONF_FAIL_ZC_CERTIFICATE_CHANGED,
    /// Vanity URL not exist
    VanityNotExist = ZOOMSDK_MeetingFailCode_CONF_FAIL_VANITY_NOT_EXIST,
    /// Join webinar with same email
    JoinWebinarSameEmail = ZOOMSDK_MeetingFailCode_CONF_FAIL_JOIN_WEBINAR_WITHSAMEEMAIL,
    /// Host not allowed to start meeting
    DisallowHostMeeting = ZOOMSDK_MeetingFailCode_CONF_FAIL_DISALLOW_HOST_MEETING,
    /// Write config file error
    WriteConfigFile = ZOOMSDK_MeetingFailCode_MEETING_FAIL_WRITE_CONFIG_FILE,
    /// Forbidden to join internal meeting
    ForbidJoinInternalMeeting =
        ZOOMSDK_MeetingFailCode_MEETING_FAIL_FORBID_TO_JOIN_INTERNAL_MEETING,
    /// Removed by host
    RemovedByHost = ZOOMSDK_MeetingFailCode_CONF_FAIL_REMOVED_BY_HOST,
    /// Host disallow outside user join
    HostDisallowOutsideUserJoin =
        ZOOMSDK_MeetingFailCode_MEETING_FAIL_HOST_DISALLOW_OUTSIDE_USER_JOIN,
    /// Unable to join external meeting
    UnableJoinExternalMeeting =
        ZOOMSDK_MeetingFailCode_MEETING_FAIL_UNABLE_TO_JOIN_EXTERNAL_MEETING,
    /// Blocked by account admin
    BlockedByAccountAdmin = ZOOMSDK_MeetingFailCode_MEETING_FAIL_BLOCKED_BY_ACCOUNT_ADMIN,
    /// Need sign in for private meeting
    NeedSignInPrivateMeeting =
        ZOOMSDK_MeetingFailCode_MEETING_FAIL_NEED_SIGN_IN_FOR_PRIVATE_MEETING,
    /// App privilege token error
    AppPrivilegeTokenError = ZOOMSDK_MeetingFailCode_MEETING_FAIL_APP_PRIVILEGE_TOKEN_ERROR,
    /// Authorized user not in meeting (OBF token required user not present)
    AuthorizedUserNotInMeeting = ZOOMSDK_MeetingFailCode_MEETING_FAIL_AUTHORIZED_USER_NOT_INMEETING,
    /// JMAK user email not match
    JMAKUserEmailNotMatch = ZOOMSDK_MeetingFailCode_MEETING_FAIL_JMAK_USER_EMAIL_NOT_MATCH,
    /// Unknown error
    Unknown = ZOOMSDK_MeetingFailCode_MEETING_FAIL_UNKNOWN,
}

#[allow(non_upper_case_globals)]
impl TryFrom<u32> for MeetingFailCode {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            ZOOMSDK_MeetingFailCode_MEETING_SUCCESS => Ok(Self::Success),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_CONNECTION_ERR => Ok(Self::ConnectionError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_RECONNECT_ERR => Ok(Self::ReconnectError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MMR_ERR => Ok(Self::MMRError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_PASSWORD_ERR => Ok(Self::PasswordError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_SESSION_ERR => Ok(Self::SessionError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_OVER => Ok(Self::MeetingOver),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_START => Ok(Self::MeetingNotStarted),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_EXIST => Ok(Self::MeetingNotExist),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_USER_FULL => Ok(Self::MeetingUserFull),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_CLIENT_INCOMPATIBLE => {
                Ok(Self::ClientIncompatible)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_NO_MMR => Ok(Self::NoMMR),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_CONFLOCKED => Ok(Self::MeetingLocked),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED => Ok(Self::MeetingRestricted),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED_JBH => {
                Ok(Self::MeetingRestrictedJBH)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_EMIT_WEBREQUEST => {
                Ok(Self::CannotEmitWebRequest)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_START_TOKENEXPIRE => Ok(Self::TokenExpired),
            ZOOMSDK_MeetingFailCode_SESSION_VIDEO_ERR => Ok(Self::VideoError),
            ZOOMSDK_MeetingFailCode_SESSION_AUDIO_AUTOSTARTERR => Ok(Self::AudioAutoStartError),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_FULL => {
                Ok(Self::WebinarRegisterFull)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_HOSTREGISTER => {
                Ok(Self::WebinarHostRegister)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_PANELISTREGISTER => {
                Ok(Self::WebinarPanelistRegister)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_DENIED_EMAIL => {
                Ok(Self::WebinarDeniedEmail)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_ENFORCE_LOGIN => Ok(Self::EnforceLogin),
            ZOOMSDK_MeetingFailCode_CONF_FAIL_ZC_CERTIFICATE_CHANGED => {
                Ok(Self::CertificateChanged)
            }
            ZOOMSDK_MeetingFailCode_CONF_FAIL_VANITY_NOT_EXIST => Ok(Self::VanityNotExist),
            ZOOMSDK_MeetingFailCode_CONF_FAIL_JOIN_WEBINAR_WITHSAMEEMAIL => {
                Ok(Self::JoinWebinarSameEmail)
            }
            ZOOMSDK_MeetingFailCode_CONF_FAIL_DISALLOW_HOST_MEETING => {
                Ok(Self::DisallowHostMeeting)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_WRITE_CONFIG_FILE => Ok(Self::WriteConfigFile),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_FORBID_TO_JOIN_INTERNAL_MEETING => {
                Ok(Self::ForbidJoinInternalMeeting)
            }
            ZOOMSDK_MeetingFailCode_CONF_FAIL_REMOVED_BY_HOST => Ok(Self::RemovedByHost),
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_HOST_DISALLOW_OUTSIDE_USER_JOIN => {
                Ok(Self::HostDisallowOutsideUserJoin)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_UNABLE_TO_JOIN_EXTERNAL_MEETING => {
                Ok(Self::UnableJoinExternalMeeting)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_BLOCKED_BY_ACCOUNT_ADMIN => {
                Ok(Self::BlockedByAccountAdmin)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_NEED_SIGN_IN_FOR_PRIVATE_MEETING => {
                Ok(Self::NeedSignInPrivateMeeting)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_APP_PRIVILEGE_TOKEN_ERROR => {
                Ok(Self::AppPrivilegeTokenError)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_AUTHORIZED_USER_NOT_INMEETING => {
                Ok(Self::AuthorizedUserNotInMeeting)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_JMAK_USER_EMAIL_NOT_MATCH => {
                Ok(Self::JMAKUserEmailNotMatch)
            }
            ZOOMSDK_MeetingFailCode_MEETING_FAIL_UNKNOWN => Ok(Self::Unknown),
            _ => Err("Invalid meeting fail code"),
        }
    }
}
