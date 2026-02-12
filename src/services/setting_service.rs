use std::ptr;

use crate::{bindings::*, SdkResult, ZoomRsError};

/// Sub-service related to audio, microphone, auto-join, etc...
pub mod audio_context;

use audio_context::AudioContext;

/// Main instance of the parameters.
#[derive(Debug)]
pub struct SettingService<'a> {
    ref_setting_service: &'a mut ZOOMSDK_ISettingService,
    audio_context: Option<AudioContext<'a>>,
}

impl<'a> SettingService<'a> {
    /// Create setting service interface
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [crate::SdkError] for details.
    pub fn new() -> SdkResult<Self> {
        let mut ptr = ptr::null_mut();
        let ret = unsafe { ZOOMSDK_CreateSettingService(&mut ptr) };
        if ret == ZOOMSDK_SDKError_SDKERR_SUCCESS {
            Ok(SettingService {
                ref_setting_service: unsafe { ptr.as_mut() }.unwrap(),
                audio_context: None,
            })
        } else {
            Err(ZoomRsError::Sdk(ret.into()))
        }
    }
    /// Get Audio Context.
    pub fn audio_context(&mut self) -> &mut AudioContext<'a> {
        if self.audio_context.is_none() {
            self.audio_context = Some(AudioContext::new(self.ref_setting_service).unwrap());
            self.audio_context
                .as_ref()
                .expect("Cannot create AudioContext");
        }
        self.audio_context.as_mut().unwrap()
    }
}

impl<'a> Drop for SettingService<'a> {
    fn drop(&mut self) {
        // Always destroy the setting service — service objects are our responsibility.
        let ret = unsafe { ZOOMSDK_DestroySettingService(self.ref_setting_service) };
        if ret != ZOOMSDK_SDKError_SDKERR_SUCCESS {
            tracing::warn!("Error when droping SettingService : {:?}", ret);
        } else {
            tracing::info!("Setting instance droped!");
        }
    }
}
