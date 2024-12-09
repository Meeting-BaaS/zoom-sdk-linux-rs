use crate::{bindings::*, SdkResult, ZoomSdkResult};

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub ref_audio_context: &'a mut ZOOMSDK_IAudioSettingContext,
}

impl<'a> AudioContext<'a> {
    /// Create audio context interface
    /// - If the function succeeds, the return value is [AudioContext]. Otherwise returns None.
    pub fn new(setting_service: &mut ZOOMSDK_ISettingService) -> Option<Self> {
        let ptr = unsafe { get_audio_settings(setting_service) };
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_audio_context: unsafe { ptr.as_mut() }.unwrap(),
            })
        }
    }
    /// Enable the audio automatically when join meeting.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn enable_auto_join_audio(&mut self) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { enable_auto_join_audio(self.ref_audio_context, true) },
            (),
        )
        .into()
    }
}
