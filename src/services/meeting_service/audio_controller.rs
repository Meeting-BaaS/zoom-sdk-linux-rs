use crate::{bindings::*, SdkResult, ZoomSdkResult};

/// Main audio controller instance.
#[derive(Debug)]
pub struct AudioController<'a> {
    /// Pointer to rhe underlaying cpp audio controller.
    pub ref_audio_controller: &'a mut ZOOMSDK_IMeetingAudioController,
}

impl<'a> AudioController<'a> {
    /// Create audio controller
    /// - If the function succeeds, the return value is [AudioController]. Otherwise returns None.
    pub fn new(meting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_audio_controller(meting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_audio_controller: unsafe { ptr.as_mut() }.unwrap(),
            })
        }
    }

    /// Unmute a given user_id
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn unmute_audio(&mut self, user_id: u32) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { meeting_unmute_microphone(self.ref_audio_controller, user_id) },
            (),
        )
        .into()
    }
}
