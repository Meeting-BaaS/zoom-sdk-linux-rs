use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_int;
use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomSdkResult};

const REMINDER_ACTION_NONE: c_int = 0;
const REMINDER_ACTION_ACCEPT: c_int = 1;
const REMINDER_ACTION_DECLINE: c_int = 2;
const REMINDER_ACTION_IGNORE: c_int = 3;
const REMINDER_ACTION_START: c_int = 4;

const TYPE_LOGIN_REQUIRED: u32 = 0;
const TYPE_START_OR_JOIN_MEETING: u32 = 1;
const TYPE_RECORD_REMINDER: u32 = 2;
const TYPE_RECORD_DISCLAIMER: u32 = 3;
const TYPE_LIVE_STREAM_DISCLAIMER: u32 = 4;
const TYPE_ARCHIVE_DISCLAIMER: u32 = 5;
const TYPE_WEBINAR_AS_PANELIST_JOIN: u32 = 6;
const TYPE_TERMS_OF_SERVICE: u32 = 7;
const TYPE_SMART_SUMMARY_DISCLAIMER: u32 = 8;
const TYPE_SMART_SUMMARY_ENABLE_REQUEST_REMINDER: u32 = 9;
const TYPE_QUERY_DISCLAIMER: u32 = 10;
const TYPE_QUERY_ENABLE_REQUEST_REMINDER: u32 = 11;
const TYPE_ENABLE_SMART_SUMMARY_REMINDER: u32 = 12;
const TYPE_WEBINAR_ATTENDEE_PROMOTE_REMINDER: u32 = 13;
const TYPE_JOIN_PRIVATE_MODE_MEETING_REMINDER: u32 = 14;
const TYPE_SMART_RECORDING_ENABLE_REQUEST_REMINDER: u32 = 15;
const TYPE_ENABLE_SMART_RECORDING_REMINDER: u32 = 16;
const TYPE_AI_COMPANION_PLUS_DISCLAIMER: u32 = 17;
const TYPE_CLOSED_CAPTION_DISCLAIMER: u32 = 18;
const TYPE_MULTI_DISCLAIMER: u32 = 19;
const TYPE_JOIN_MEETING_CONNECTOR_AS_GUEST_REMINDER: u32 = 20;
const TYPE_COMMON_DISCLAIMER: u32 = 21;
const TYPE_CUSTOM_AI_COMPANION_DISCLAIMER: u32 = 22;
const TYPE_AIC_RESTRICT_NOTIFY_DISCLAIMER: u32 = 23;

/// Action returned to the Zoom SDK reminder handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderAction {
    /// Leave the reminder dialog unchanged.
    None,
    /// Accept or agree to the reminder dialog.
    Accept,
    /// Decline the reminder dialog.
    Decline,
    /// Ignore/dismiss the reminder dialog when the SDK supports it.
    Ignore,
    /// Start the feature covered by an enable-reminder dialog.
    Start,
}

impl From<ReminderAction> for c_int {
    fn from(action: ReminderAction) -> Self {
        match action {
            ReminderAction::None => REMINDER_ACTION_NONE,
            ReminderAction::Accept => REMINDER_ACTION_ACCEPT,
            ReminderAction::Decline => REMINDER_ACTION_DECLINE,
            ReminderAction::Ignore => REMINDER_ACTION_IGNORE,
            ReminderAction::Start => REMINDER_ACTION_START,
        }
    }
}

/// Data from Zoom's reminder/disclaimer dialog callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderContent {
    /// Raw `MeetingReminderType` value from the Zoom SDK.
    pub reminder_type: u32,
    /// Best-effort symbolic name for `reminder_type`.
    pub reminder_type_name: &'static str,
    /// Dialog title supplied by Zoom.
    pub title: String,
    /// Dialog body supplied by Zoom.
    pub content: String,
    /// Whether the dialog blocks meeting progress until handled.
    pub is_blocking: bool,
    /// Raw reminder action type value from the Zoom SDK.
    pub action_type: u32,
}

/// This trait handles Zoom reminder/disclaimer dialogs.
pub trait ReminderEvent: fmt::Debug + Send {
    /// Callback for reminder dialogs with Accept/Decline/Ignore semantics.
    fn on_reminder_notify(&mut self, _content: ReminderContent) -> ReminderAction {
        ReminderAction::None
    }

    /// Callback for feature-enable reminder dialogs.
    fn on_enable_reminder_notify(&mut self, _content: ReminderContent) -> ReminderAction {
        ReminderAction::None
    }
}

/// Main reminder controller interface.
pub struct ReminderController<'a> {
    ref_reminder_controller: &'a mut ZOOMSDK_IMeetingReminderController,
    evt_mutex: Option<Arc<Mutex<Box<dyn ReminderEvent>>>>,
}

impl<'a> fmt::Debug for ReminderController<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReminderController")
            .field("ref_reminder_controller", self.ref_reminder_controller)
            .finish()
    }
}

impl<'a> ReminderController<'a> {
    /// Get the reminder controller interface.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_reminder_controller(meeting_service) };

        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_reminder_controller: unsafe { ptr.as_mut() }.unwrap(),
                evt_mutex: None,
            })
        }
    }

    /// Set the reminder controller callback event handler.
    pub fn set_event(&mut self, ctx: Box<dyn ReminderEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        tracing::info!("Setting reminder event handler: {:?}", ptr);
        ZoomSdkResult(
            unsafe { reminder_set_event(self.ref_reminder_controller, ptr) },
            (),
        )
        .into()
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_reminder_notify(
    ptr: *const u8,
    reminder_type: c_int,
    title: *const zchar_t,
    content: *const zchar_t,
    is_blocking: c_int,
    action_type: c_int,
) -> c_int {
    let content = ReminderContent::new(reminder_type, title, content, is_blocking, action_type);
    (*convert_reminder_event(ptr).lock().unwrap())
        .on_reminder_notify(content)
        .into()
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_enable_reminder_notify(
    ptr: *const u8,
    reminder_type: c_int,
    title: *const zchar_t,
    content: *const zchar_t,
    is_blocking: c_int,
    action_type: c_int,
) -> c_int {
    let content = ReminderContent::new(reminder_type, title, content, is_blocking, action_type);
    (*convert_reminder_event(ptr).lock().unwrap())
        .on_enable_reminder_notify(content)
        .into()
}

impl ReminderContent {
    fn new(
        reminder_type: c_int,
        title: *const zchar_t,
        content: *const zchar_t,
        is_blocking: c_int,
        action_type: c_int,
    ) -> Self {
        let reminder_type = reminder_type.max(0) as u32;
        Self {
            reminder_type,
            reminder_type_name: reminder_type_name(reminder_type),
            title: unsafe_zchar_to_string(title),
            content: unsafe_zchar_to_string(content),
            is_blocking: is_blocking != 0,
            action_type: action_type.max(0) as u32,
        }
    }
}

#[inline]
fn unsafe_zchar_to_string(ptr: *const zchar_t) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .unwrap_or("")
        .to_owned()
}

#[inline]
fn convert_reminder_event(ptr: *const u8) -> Arc<Mutex<Box<dyn ReminderEvent>>> {
    let ptr: *const Mutex<Box<dyn ReminderEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Return a symbolic name for a raw Zoom `MeetingReminderType` value.
pub fn reminder_type_name(reminder_type: u32) -> &'static str {
    match reminder_type {
        TYPE_LOGIN_REQUIRED => "TYPE_LOGIN_REQUIRED",
        TYPE_START_OR_JOIN_MEETING => "TYPE_START_OR_JOIN_MEETING",
        TYPE_RECORD_REMINDER => "TYPE_RECORD_REMINDER",
        TYPE_RECORD_DISCLAIMER => "TYPE_RECORD_DISCLAIMER",
        TYPE_LIVE_STREAM_DISCLAIMER => "TYPE_LIVE_STREAM_DISCLAIMER",
        TYPE_ARCHIVE_DISCLAIMER => "TYPE_ARCHIVE_DISCLAIMER",
        TYPE_WEBINAR_AS_PANELIST_JOIN => "TYPE_WEBINAR_AS_PANELIST_JOIN",
        TYPE_TERMS_OF_SERVICE => "TYPE_TERMS_OF_SERVICE",
        TYPE_SMART_SUMMARY_DISCLAIMER => "TYPE_SMART_SUMMARY_DISCLAIMER",
        TYPE_SMART_SUMMARY_ENABLE_REQUEST_REMINDER => "TYPE_SMART_SUMMARY_ENABLE_REQUEST_REMINDER",
        TYPE_QUERY_DISCLAIMER => "TYPE_QUERY_DISCLAIMER",
        TYPE_QUERY_ENABLE_REQUEST_REMINDER => "TYPE_QUERY_ENABLE_REQUEST_REMINDER",
        TYPE_ENABLE_SMART_SUMMARY_REMINDER => "TYPE_ENABLE_SMART_SUMMARY_REMINDER",
        TYPE_WEBINAR_ATTENDEE_PROMOTE_REMINDER => "TYPE_WEBINAR_ATTENDEE_PROMOTE_REMINDER",
        TYPE_JOIN_PRIVATE_MODE_MEETING_REMINDER => "TYPE_JOIN_PRIVATE_MODE_MEETING_REMINDER",
        TYPE_SMART_RECORDING_ENABLE_REQUEST_REMINDER => {
            "TYPE_SMART_RECORDING_ENABLE_REQUEST_REMINDER"
        }
        TYPE_ENABLE_SMART_RECORDING_REMINDER => "TYPE_ENABLE_SMART_RECORDING_REMINDER",
        TYPE_AI_COMPANION_PLUS_DISCLAIMER => "TYPE_AI_COMPANION_PLUS_DISCLAIMER",
        TYPE_CLOSED_CAPTION_DISCLAIMER => "TYPE_CLOSED_CAPTION_DISCLAIMER",
        TYPE_MULTI_DISCLAIMER => "TYPE_MULTI_DISCLAIMER",
        TYPE_JOIN_MEETING_CONNECTOR_AS_GUEST_REMINDER => {
            "TYPE_JOIN_MEETING_CONNECTOR_AS_GUEST_REMINDER"
        }
        TYPE_COMMON_DISCLAIMER => "TYPE_COMMON_DISCLAIMER",
        TYPE_CUSTOM_AI_COMPANION_DISCLAIMER => "TYPE_CUSTOM_AI_COMPANION_DISCLAIMER",
        TYPE_AIC_RESTRICT_NOTIFY_DISCLAIMER => "TYPE_AIC_RESTRICT_NOTIFY_DISCLAIMER",
        _ => "TYPE_UNKNOWN",
    }
}

/// Return whether the bot may automatically accept this reminder type.
pub fn is_auto_acceptable_reminder(reminder_type: u32) -> bool {
    matches!(
        reminder_type,
        TYPE_START_OR_JOIN_MEETING
            | TYPE_RECORD_REMINDER
            | TYPE_RECORD_DISCLAIMER
            | TYPE_TERMS_OF_SERVICE
            | TYPE_JOIN_PRIVATE_MODE_MEETING_REMINDER
            | TYPE_MULTI_DISCLAIMER
            | TYPE_JOIN_MEETING_CONNECTOR_AS_GUEST_REMINDER
            | TYPE_COMMON_DISCLAIMER
    )
}
