#include "c_setting_service_interface.h"

extern "C" ZOOMSDK::IAudioSettingContext* get_audio_settings(ZOOMSDK::ISettingService* setting_service) {
    return setting_service->GetAudioSettings();
}

extern "C" ZOOMSDK::SDKError enable_auto_join_audio(ZOOMSDK::IAudioSettingContext* audio_setting, bool value) {
    return audio_setting->EnableAutoJoinAudio(value);
}