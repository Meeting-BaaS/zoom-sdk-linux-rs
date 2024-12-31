use std::ffi::CStr;

use crate::{bindings::*, SdkError, SdkResult, ZoomSdkResult};

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
    /// elect mic device.
    /// [MicDriver] Specify the device name assigned by deviceId.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn select_microphone(&mut self, driver: &MicDriver) -> SdkResult<()> {
        if let MicDriver::Default = driver {
            return Ok(());
        }
        let mut len: u32 = 0;
        let mic_list_ptr = unsafe { get_mic_list(self.ref_audio_context, &mut len) };
        let mut v = Vec::new();
        for i in 0..len {
            unsafe {
                let p = mic_list_ptr.offset(i as isize);
                v.push(MicList {
                    device_id: CStr::from_ptr((*p).device_id),
                    device_name: CStr::from_ptr((*p).device_name),
                    selected: (*p).selected,
                })
            }
        }
        tracing::info!("Detected microphones : {:#?}", &v);
        let mic = v.iter().find(|v| {
            use MicDriver::*;
            match driver {
                SndAloop => v.device_id.to_str().unwrap().contains("snd_aloop"),
                Pulse => unimplemented!(),
                Default => unreachable!(),
            }
        });
        if let Some(selected_mic) = mic {
            tracing::info!("Selecting microphone : {:?}", selected_mic);
            ZoomSdkResult(
                unsafe {
                    select_mic(
                        self.ref_audio_context,
                        selected_mic.device_id.as_ptr(),
                        selected_mic.device_name.as_ptr(),
                    )
                },
                (),
            )
            .into()
        } else {
            tracing::error!("Cannot found microphone for {:?}", driver);
            ZoomSdkResult(SdkError::UnexpectedError as u32, ()).into()
        }
    }
    /// Set the suppress background noise level.
    /// [SupressBackgroundNoiseLevel] level The new suppress background noise level to be set.
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.
    pub fn set_suppress_background_noise_level(
        &mut self,
        level: SupressBackgroundNoiseLevel,
    ) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { set_suppress_background_noise_level(self.ref_audio_context, level as u32) },
            (),
        )
        .into()
    }
}

#[derive(Debug)]
pub enum MicDriver {
    SndAloop,
    Pulse,
    Default,
}

#[derive(Debug)]
pub struct MicList<'a> {
    pub device_id: &'a CStr,
    pub device_name: &'a CStr,
    pub selected: bool,
}

#[repr(u32)]
pub enum SupressBackgroundNoiseLevel {
    None = ZOOMSDK_Suppress_Background_Noise_Level_Suppress_BGNoise_Level_None,
    Auto = ZOOMSDK_Suppress_Background_Noise_Level_Suppress_BGNoise_Level_Auto,
    Low = ZOOMSDK_Suppress_Background_Noise_Level_Suppress_BGNoise_Level_Low,
    Medium = ZOOMSDK_Suppress_Background_Noise_Level_Suppress_BGNoise_Level_Medium,
    Heigh = ZOOMSDK_Suppress_Background_Noise_Level_Suppress_BGNoise_Level_High,
}
