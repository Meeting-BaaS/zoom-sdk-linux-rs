use std::ffi::CString;

use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// Main chat interface.
#[derive(Debug)]
pub struct ChatInterface<'a> {
    ptr_chat_controler: &'a mut ZOOMSDK_IMeetingChatController,
}

impl<'a> ChatInterface<'a> {
    /// Get the chat controller interface.
    /// - If the function succeeds, the return value is [ChatInterface]. Otherwise returns None.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_chat_controller(meeting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr_chat_controler: unsafe { ptr.as_mut() }.unwrap(),
            })
        }
    }
    /// Send chat message [str]
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn send_message(&mut self, message: String) -> SdkResult<()> {
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
        ZoomSdkResult(
            unsafe { meeting_send_chat_message_to(self.ptr_chat_controler, build) },
            (),
        )
        .into()
    }
}
