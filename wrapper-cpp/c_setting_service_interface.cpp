#include "c_setting_service_interface.h"

#include <stdio.h>
#include <cstdlib>

extern "C" ZOOMSDK::IAudioSettingContext* get_audio_settings(ZOOMSDK::ISettingService* setting_service) {
    return setting_service->GetAudioSettings();
}

extern "C" ZOOMSDK::SDKError enable_auto_join_audio(ZOOMSDK::IAudioSettingContext* audio_setting, bool value) {
    return audio_setting->EnableAutoJoinAudio(value);
}

extern "C" struct mic_list* get_mic_list(ZOOMSDK::IAudioSettingContext* audio_setting, unsigned int *len) {
    auto mic_list = audio_setting->GetMicList();
    if (!mic_list) {
        printf("NullPtr GetMicList\n");
        return NULL;
    }
    unsigned int count = mic_list->GetCount();
    struct mic_list *m = (struct mic_list*)malloc(sizeof(struct mic_list) * (size_t)count);
    for (unsigned int i = 0; i < count; i += 1) {
        ZOOMSDK::IMicInfo *mic = mic_list->GetItem(i);
        m[i].device_id = mic->GetDeviceId();
        m[i].device_name = mic->GetDeviceName();
        m[i].selected = mic->IsSelectedDevice();

        printf("id : %s\n", m[i].device_id);
        printf("name : %s\n", m[i].device_name);
        printf("is_selected: %i\n", m[i].selected);
    }
    *len = count;
    return m;
}

extern "C" ZOOMSDK::SDKError select_mic(
    ZOOMSDK::IAudioSettingContext* audio_setting,
    const zchar_t* deviceId,
    const zchar_t* deviceName
) {
    ZOOMSDK::SDKError ret_a = audio_setting->SelectMic(deviceId, deviceName);
    ZOOMSDK::SDKError ret_b = audio_setting->SetSuppressBackgroundNoiseLevel(ZOOMSDK::Suppress_Background_Noise_Level::Suppress_BGNoise_Level_None);
    printf("Noise None = %i\n", ret_b);
    if (ret_b != 0) {
        ZOOMSDK::SDKError ret_c = audio_setting->SetSuppressBackgroundNoiseLevel(ZOOMSDK::Suppress_Background_Noise_Level::Suppress_BGNoise_Level_Low);
        printf("Noise Low = %i\n", ret_c);
    }
    printf("Final = %i\n", ret_a);
    return ret_a;
}