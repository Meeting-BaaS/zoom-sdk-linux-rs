#ifndef _C_RAWDATA_AUDIO_HELPER_H_
#define _C_RAWDATA_AUDIO_HELPER_H_

#include "../../zoom-meeting-sdk-linux/h/rawdata/zoom_rawdata_api.h"
#include "../../zoom-meeting-sdk-linux/h/rawdata/rawdata_audio_helper_interface.h"
#include "../../zoom-meeting-sdk-linux/h/zoom_sdk_raw_data_def.h"

extern "C" struct exported_audio_raw_data {
    char *data;
    int64_t time;
    unsigned int len;
    // bool can_add_ref;
};

extern "C" ZOOMSDK::IZoomSDKAudioRawDataDelegate* audio_helper_create_delegate(
    void *arc_ptr,
    bool use_separate_channels);

/// \brief Subscribe raw audio data.
/// \param pDelegate, the callback handler of raw audio data.
/// \param bWithInterpreters, if bWithInterpreters is true, it means that you want to get the raw audio data of interpreters, otherwise not. 
///        NOTE: if bWithInterpreters is true, it will cause your local interpreter related functions to be unavailable.
/// \return If the function succeeds, the return value is SDKERR_SUCCESS.
///Otherwise fails. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError audio_helper_subscribe_delegate(
    ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx,
    ZOOMSDK::IZoomSDKAudioRawDataDelegate* pDelegate);

extern "C" ZOOMSDK::SDKError audio_helper_unsubscribe_delegate(ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx);

#endif