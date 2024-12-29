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

/// \brief Subscribe audio mic raw data with a callback.
/// \param pSource, Callback sink object.
/// \return If the function succeeds, the return value is SDKERR_SUCCESS.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError audio_helper_set_external_audio_source(
    ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx,
    void *arc_ptr);

/// \brief Send audio raw data. Audio sample must be 16-bit audio.
/// \param data the audio datas address.
/// \param data_length the audio datas length. Must be an even number.
/// \param sample_rate the audio datas sampling rate.
/// When the channel is mono, supported sample rates: 8000/11025/16000/32000/44100/48000/50000/50400/96000/192000/2822400
/// \return If the function succeeds, the return value is SDKERR_SUCCESS.
// /Otherwise the function fails and returns an error code. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError send_audio_raw_data(
    ZOOMSDK::IZoomSDKAudioRawDataSender* p_sender,
    char* data,
    unsigned int data_length,
    int sample_rate);

#endif