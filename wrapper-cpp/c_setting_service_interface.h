#ifndef _C_SETTING_SERVICE_INTERFACE_H_
#define _C_SETTING_SERVICE_INTERFACE_H_

#include "../zoom-meeting-sdk-linux/h/setting_service_interface.h"

/// \brief Get audio setting interface.
/// \return If the function succeeds, the return value an object pointer to IAudioSettingContext.
/// Otherwise failed, returns NULL.
/// For more details, see \link IAudioSettingContext \endlink.
extern "C" ZOOMSDK::IAudioSettingContext* get_audio_settings(ZOOMSDK::ISettingService *setting_service);

/// \brief Enable or disable the audio automatically when join meeting.
/// \param bEnable TRUE indicates to enable the audio automatically when join meeting.
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError enable_auto_join_audio(ZOOMSDK::IAudioSettingContext* audio_setting, bool value);

#endif