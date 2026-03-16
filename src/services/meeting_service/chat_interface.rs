use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// Data received from a chat message notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessageReceived {
    /// Unique identifier for the chat message.
    pub message_id: String,
    /// User ID of the message sender.
    pub sender_user_id: u32,
    /// Display name of the message sender.
    pub sender_display_name: String,
    /// Text content of the chat message.
    pub content: String,
    /// Unix timestamp when the message was sent.
    pub timestamp: u64,
}

/// This trait handles events related to chat messages.
pub trait ChatEvent: std::fmt::Debug + Send {
    /// Callback event when a chat message is received.
    fn on_chat_msg_notification(&mut self, _msg: ChatMessageReceived) {}
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_chat_msg_notification(
    ptr: *const u8,
    message_id: *const i8,
    sender_user_id: u32,
    sender_display_name: *const i8,
    content: *const i8,
    timestamp: i64,
) {
    let msg = ChatMessageReceived {
        message_id: unsafe_cstr_to_string(message_id),
        sender_user_id,
        sender_display_name: unsafe_cstr_to_string(sender_display_name),
        content: unsafe_cstr_to_string(content),
        timestamp: timestamp as u64,
    };
    tracing::info!("Chat message received: {:?}", msg);
    (*convert_chat_event(ptr).lock().unwrap()).on_chat_msg_notification(msg);
}

#[inline]
fn unsafe_cstr_to_string(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .unwrap_or("")
        .to_owned()
}

#[inline]
fn convert_chat_event(ptr: *const u8) -> Arc<Mutex<Box<dyn ChatEvent>>> {
    let ptr: *const Mutex<Box<dyn ChatEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Main chat interface.
#[derive(Debug)]
pub struct ChatInterface<'a> {
    ptr_chat_controler: &'a mut ZOOMSDK_IMeetingChatController,
    evt_mutex: Option<Arc<Mutex<Box<dyn ChatEvent>>>>,
}

impl<'a> ChatInterface<'a> {
    /// Get the chat controller interface.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_chat_controller(meeting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr_chat_controler: unsafe { ptr.as_mut() }.unwrap(),
                evt_mutex: None,
            })
        }
    }

    /// Set the chat controller callback event handler.
    pub fn set_event(&mut self, ctx: Box<dyn ChatEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        tracing::info!("Setting chat event handler: {:?}", ptr);
        ZoomSdkResult(
            unsafe { chat_set_event(self.ptr_chat_controler, ptr) },
            (),
        )
        .into()
    }

    /// Send chat message [str]
    pub fn send_message(&mut self, message: String) -> SdkResult<()> {
        if message.len() == 0 {
            return Ok(());
        }
        let builder = unsafe { meeting_get_chat_message_builder(self.ptr_chat_controler) };
        if builder.is_null() {
            tracing::warn!("NullPtr received!");
            return Err(ZoomRsError::NullPtr);
        }
        let message = CString::new(message).unwrap();
        let build = unsafe {
            meeting_chat_build(
                builder,
                message.as_ptr() as _,
                ZOOMSDK_SDKChatMessageType_SDKChatMessageType_To_All,
            )
        };
        if build.is_null() {
            tracing::warn!("NullPtr received!");
            return Err(ZoomRsError::NullPtr);
        }
        let sdk_error = unsafe { meeting_send_chat_message_to(self.ptr_chat_controler, build) };
        tracing::info!("meeting_send_chat_message_to() returned SDK error code: {}", sdk_error);
        ZoomSdkResult(sdk_error, ()).into()
    }
}
