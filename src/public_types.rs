use std::ffi::CString;

use super::*;

/// Main Error type
#[derive(Debug, Copy, Clone)]
pub enum ZoomRsError {
    Sdk(SdkError),
    NullPtr,
}

/// Result from Zoom SDK
pub type SdkResult<T> = Result<T, ZoomRsError>;

pub struct ZoomSdkResult<T>(pub u32, pub T);

impl<T> From<ZoomSdkResult<T>> for SdkResult<T> {
    #[inline]
    fn from(ZoomSdkResult(res, value): ZoomSdkResult<T>) -> Self {
        match res {
            0 => Ok(value),
            n => Err(ZoomRsError::Sdk(n.into())),
        }
    }
}

/// Represents the possible SDK errors in the Zoom SDK.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SdkError {
    /// Success.
    Success = ZOOMSDK_SDKError_SDKERR_SUCCESS,
    /// This feature is currently invalid.
    NoImpl = ZOOMSDK_SDKError_SDKERR_NO_IMPL,
    /// Incorrect usage of the feature.
    WrongUsage = ZOOMSDK_SDKError_SDKERR_WRONG_USAGE,
    /// Wrong parameter.
    InvalidParameter = ZOOMSDK_SDKError_SDKERR_INVALID_PARAMETER,
    /// Loading module failed.
    ModuleLoadFailed = ZOOMSDK_SDKError_SDKERR_MODULE_LOAD_FAILED,
    /// No memory is allocated.
    MemoryFailed = ZOOMSDK_SDKError_SDKERR_MEMORY_FAILED,
    /// Internal service error.
    ServiceFailed = ZOOMSDK_SDKError_SDKERR_SERVICE_FAILED,
    /// Not initialized before the usage.
    Uninitialize = ZOOMSDK_SDKError_SDKERR_UNINITIALIZE,
    /// Not authorized before the usage.
    Unauthentication = ZOOMSDK_SDKError_SDKERR_UNAUTHENTICATION,
    /// No recording in process.
    NoRecordingInProcess = ZOOMSDK_SDKError_SDKERR_NORECORDINGINPROCESS,
    /// Transcoder module is not found.
    TranscoderNotFound = ZOOMSDK_SDKError_SDKERR_TRANSCODER_NOFOUND,
    /// The video service is not ready.
    VideoNotReady = ZOOMSDK_SDKError_SDKERR_VIDEO_NOTREADY,
    /// No permission.
    NoPermission = ZOOMSDK_SDKError_SDKERR_NO_PERMISSION,
    /// Unknown error.
    Unknown = ZOOMSDK_SDKError_SDKERR_UNKNOWN,
    /// The other instance of the SDK is in process.
    OtherSdkInstanceRunning = ZOOMSDK_SDKError_SDKERR_OTHER_SDK_INSTANCE_RUNNING,
    /// SDK internal error.
    InternalError = ZOOMSDK_SDKError_SDKERR_INTERNAL_ERROR,
    /// No audio device found.
    NoAudioDeviceFound = ZOOMSDK_SDKError_SDKERR_NO_AUDIODEVICE_ISFOUND,
    /// No video device found.
    NoVideoDeviceFound = ZOOMSDK_SDKError_SDKERR_NO_VIDEODEVICE_ISFOUND,
    /// API calls too frequently.
    TooFrequentCall = ZOOMSDK_SDKError_SDKERR_TOO_FREQUENT_CALL,
    /// User can't be assigned with new privilege.
    FailAssignUserPrivilege = ZOOMSDK_SDKError_SDKERR_FAIL_ASSIGN_USER_PRIVILEGE,
    /// The current meeting doesn't support the feature.
    MeetingDontSupportFeature = ZOOMSDK_SDKError_SDKERR_MEETING_DONT_SUPPORT_FEATURE,
    /// The current user is not the presenter.
    MeetingNotShareSender = ZOOMSDK_SDKError_SDKERR_MEETING_NOT_SHARE_SENDER,
    /// There is no sharing.
    MeetingNoShare = ZOOMSDK_SDKError_SDKERR_MEETING_YOU_HAVE_NO_SHARE,
    /// Incorrect ViewType parameters.
    MeetingViewTypeParameterIsWrong = ZOOMSDK_SDKError_SDKERR_MEETING_VIEWTYPE_PARAMETER_IS_WRONG,
    /// Annotation is disabled.
    MeetingAnnotationIsOff = ZOOMSDK_SDKError_SDKERR_MEETING_ANNOTATION_IS_OFF,
    /// Current OS doesn't support the setting.
    SettingOsDontSupport = ZOOMSDK_SDKError_SDKERR_SETTING_OS_DONT_SUPPORT,
    /// Email login is disabled.
    EmailLoginIsDisabled = ZOOMSDK_SDKError_SDKERR_EMAIL_LOGIN_IS_DISABLED,
    /// Computer doesn't meet the minimum requirements to use virtual background feature.
    HardwareNotMeetForVb = ZOOMSDK_SDKError_SDKERR_HARDWARE_NOT_MEET_FOR_VB,
    /// Need process disclaimer.
    NeedUserConfirmRecordDisclaimer = ZOOMSDK_SDKError_SDKERR_NEED_USER_CONFIRM_RECORD_DISCLAIMER,
    /// There is no raw data of sharing.
    NoShareData = ZOOMSDK_SDKError_SDKERR_NO_SHARE_DATA,
    /// Cannot subscribe to your own share.
    ShareCannotSubscribeMyself = ZOOMSDK_SDKError_SDKERR_SHARE_CANNOT_SUBSCRIBE_MYSELF,
    /// Not in a meeting.
    NotInMeeting = ZOOMSDK_SDKError_SDKERR_NOT_IN_MEETING,
    /// Audio not joined.
    NotJoinAudio = ZOOMSDK_SDKError_SDKERR_NOT_JOIN_AUDIO,
    /// The current device doesn't support the feature.
    HardwareDontSupport = ZOOMSDK_SDKError_SDKERR_HARDWARE_DONT_SUPPORT,
    /// Domain doesn't support the feature.
    DomainDontSupport = ZOOMSDK_SDKError_SDKERR_DOMAIN_DONT_SUPPORT,
    /// Remote control is disabled.
    MeetingRemoteControlIsOff = ZOOMSDK_SDKError_SDKERR_MEETING_REMOTE_CONTROL_IS_OFF,
    /// File transfer error.
    FileTransferError = ZOOMSDK_SDKError_SDKERR_FILETRANSFER_ERROR,
    /// Unexpected error from SDK - Normally it doesnot happen
    UnexpectedError = ZOOMSDK_SDKError_SDKERR_FILETRANSFER_ERROR + 1,
}

/// From boilerplate for SdkError
impl From<u32> for SdkError {
    #[inline]
    fn from(err: u32) -> Self {
        match err {
            n if n <= ZOOMSDK_SDKError_SDKERR_FILETRANSFER_ERROR => unsafe {
                std::mem::transmute::<u32, SdkError>(err)
            },
            _ => SdkError::UnexpectedError,
        }
    }
}

/// Zoom initialization parameters
#[derive(Default, Clone)]
pub struct SdkInitParam {
    /// Web domain.
    pub str_web_domain: CString,
    /// Branding name.
    pub str_branding_name: CString,
    /// Support URL.
    pub str_support_url: CString,
    /// The ID of the SDK language.
    pub em_language_id: SdkLanguageId,
    /// Enable generate dump file if the app crashes.
    pub enable_generate_dump: bool,
    /// Enable log feature.
    pub enable_log_by_default: bool,
    /// Size of a log file in megabytes (M). The default size is 5M.
    /// There are 5 log files in total, and the file size varies from 1M to 50M.
    pub ui_log_file_size: u32,
    /// Raw data options.
    pub rawdata_opts: SdkRawDataOptions,
    /// Wrapper type.
    pub wrapper_type: u32,
}

/// Raw data options
#[derive(Default, Debug, Copy, Clone)]
pub struct SdkRawDataOptions {
    /// false -- YUV420data, true -- intermediate data
    pub enable_rawdata_intermediate_mode: bool,
    pub video_rawdata_memory_mode: SdkRawDataMemoryMode,
    pub share_rawdata_memory_mode: SdkRawDataMemoryMode,
    pub audio_rawdata_memory_mode: SdkRawDataMemoryMode,
}

/// A From boilerplate
impl From<SdkRawDataOptions> for ZOOMSDK_RawDataOptions {
    fn from(this: SdkRawDataOptions) -> Self {
        Self {
            enableRawdataIntermediateMode: this.enable_rawdata_intermediate_mode,
            videoRawdataMemoryMode: this.video_rawdata_memory_mode as u32,
            shareRawdataMemoryMode: this.share_rawdata_memory_mode as u32,
            audioRawdataMemoryMode: this.audio_rawdata_memory_mode as u32,
        }
    }
}

/// Memory mode for RawData
#[derive(Default, Debug, Copy, Clone)]
#[repr(u32)]
pub enum SdkRawDataMemoryMode {
    #[default]
    Stack = ZOOMSDK_ZoomSDKRawDataMemoryMode_ZoomSDKRawDataMemoryModeStack,
    Heap = ZOOMSDK_ZoomSDKRawDataMemoryMode_ZoomSDKRawDataMemoryModeHeap,
}

/// Represents the possible language IDs in the Zoom SDK.
#[derive(Default, Debug, Clone)]
#[repr(u32)]
pub enum SdkLanguageId {
    /// For initialization.
    Unknown = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Unknown,
    /// In English.
    #[default]
    English = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_English,
    /// In simplified Chinese.
    ChineseSimplified = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Chinese_Simplified,
    /// In traditional Chinese.
    ChineseTraditional = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Chinese_Traditional,
    /// In Japanese.
    Japanese = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Japanese,
    /// In Spanish.
    Spanish = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Spanish,
    /// In German.
    German = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_German,
    /// In French.
    French = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_French,
    /// In Portuguese.
    Portuguese = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Portuguese,
    /// In Russian.
    Russian = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Russian,
    /// In Korean.
    Korean = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Korean,
    /// In Vietnamese.
    Vietnamese = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Vietnamese,
    /// In Italian.
    Italian = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Italian,
    /// In Polish.
    Polish = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Polish,
    /// In Turkish.
    Turkish = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Turkish,
    /// In Indonesian.
    Indonesian = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Indonesian,
    /// In Dutch.
    Dutch = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Dutch,
    /// In Swedish.
    Swedish = ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Swedish,
}
