#include "c_rawdata_audio_helper.h"

#include <chrono>
#include <stdio.h>

extern "C" int32_t on_one_way_audio_raw_data(void *ptr, struct exported_audio_raw_data *data, uint32_t user_id);

extern "C" int32_t on_mixed_audio_raw_data(void *ptr, struct exported_audio_raw_data *data);

extern "C" int32_t on_share_audio_raw_data(void *ptr, struct exported_audio_raw_data *data);

// #include <fstream>
// #include <iostream>

class ZoomSDKAudioRawDataDelegate : public ZOOMSDK::IZoomSDKAudioRawDataDelegate {
public:
    ZoomSDKAudioRawDataDelegate(void *ptr, bool separate_channels) {
        ptr_to_rust = ptr;
        use_separate_channels = separate_channels;
    }
    void onMixedAudioRawDataReceived(AudioRawData* rawdata) override {
        // std::ofstream file("output_audio.raw", std::ios::binary | std::ios::app);
        // if (!file.is_open()) {
        //     std::cerr << "Failed to open file!" << std::endl;
        //     return;
        // }
        // int buffer_len = rawdata->GetBufferLen();
        // int channel_num = rawdata->GetChannelNum();
        // int sample_rate = rawdata->GetSampleRate();

        // file.write(reinterpret_cast<const char*>(rawdata->GetBuffer()), buffer_len);

        // file.close();

        struct exported_audio_raw_data data = provide(rawdata);
        on_mixed_audio_raw_data(ptr_to_rust, &data);
    }
    void onOneWayAudioRawDataReceived(AudioRawData* rawdata, uint32_t user_id) override {
        if (!use_separate_channels) {
            return;
        }
        struct exported_audio_raw_data data = provide(rawdata);
        on_one_way_audio_raw_data(ptr_to_rust, &data, user_id);
    }
    void onShareAudioRawDataReceived(AudioRawData* rawdata, uint32_t user_id) override {
        (void) user_id; // Unused parameter
        // TODO : May crash with segfault
        struct exported_audio_raw_data data = provide(rawdata);
        int32_t res = on_share_audio_raw_data(ptr_to_rust, &data);
        if (res == 1) {
            printf("increment\n");
            rawdata->AddRef();
        } else if (res < 0) {
            for (; res < 0; res += 1) {
                printf("Executing purge\n");
                rawdata->Release();
            }
        }
    }
    void onOneWayInterpreterAudioRawDataReceived(AudioRawData* data_, const zchar_t* pLanguageName) override {
        (void) data_;
        (void) pLanguageName;
    }
private:
    inline struct exported_audio_raw_data provide(AudioRawData* rawdata) {
        using namespace std::chrono;
        int64_t timestamp = duration_cast<microseconds>(system_clock::now().time_since_epoch()).count();
        
        struct exported_audio_raw_data data = {
            data: rawdata->GetBuffer(),
            time: timestamp,
            len: rawdata->GetBufferLen(),
            // can_add_ref: rawdata->CanAddRef(),
        };
        return data;
    }
    void *ptr_to_rust;
    bool use_separate_channels;
};

extern "C" ZOOMSDK::IZoomSDKAudioRawDataDelegate* audio_helper_create_delegate(
    void *arc_ptr,
    bool separate_channels) {
    auto* obj = new ZoomSDKAudioRawDataDelegate(arc_ptr, separate_channels); // TODO : Fix memory leak
    return obj;
}

extern "C" ZOOMSDK::SDKError audio_helper_subscribe_delegate(
    ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx,
    ZOOMSDK::IZoomSDKAudioRawDataDelegate* pDelegate) {
    return ctx->subscribe(pDelegate);
}

extern "C" ZOOMSDK::SDKError audio_helper_unsubscribe_delegate(ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx) {
    return ctx->unSubscribe();
}

extern "C" void on_mic_initialize(void *ptr, ZOOMSDK::IZoomSDKAudioRawDataSender* pSender);

extern "C" void on_mic_start_send(void *ptr);

extern "C" void on_mic_stop_send(void *ptr);

extern "C" void on_mic_uninitialized(void *ptr);

class ZoomSDKVirtualAudioMicEvent : public ZOOMSDK::IZoomSDKVirtualAudioMicEvent {
public:
	ZoomSDKVirtualAudioMicEvent(void *ptr) {
         ptr_to_rust = ptr;
    }

	/// \brief Callback for virtual audio mic to do some initialization.
	/// \param pSender, You can send audio data based on this object, see \link IZoomSDKAudioRawDataSender \endlink.
	virtual void onMicInitialize(ZOOMSDK::IZoomSDKAudioRawDataSender* pSender) override {
        on_mic_initialize(ptr_to_rust, pSender);
    }

	/// \brief Callback for virtual audio mic can send raw data with 'pSender'.
	virtual void onMicStartSend() override {
        on_mic_start_send(ptr_to_rust);
    }

	/// \brief Callback for virtual audio mic should stop send raw data.
	virtual void onMicStopSend() override {
        on_mic_stop_send(ptr_to_rust);
    }

	/// \brief Callback for virtual audio mic is uninitialized.
	virtual void onMicUninitialized() override {
        on_mic_uninitialized(ptr_to_rust);
    }
private:
    void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError audio_helper_set_external_audio_source(
    ZOOMSDK::IZoomSDKAudioRawDataHelper* ctx,
    void *arc_ptr) {
        auto* obj = new ZoomSDKVirtualAudioMicEvent(arc_ptr); // TODO : Fix memory leak
        return ctx->setExternalAudioSource(obj);
}

extern "C" ZOOMSDK::SDKError send_audio_raw_data(
    ZOOMSDK::IZoomSDKAudioRawDataSender* p_sender,
    char* data,
    unsigned int data_length,
    int sample_rate) {
        return p_sender->send(data, data_length, sample_rate);
}

// TODO : Check it is not bullshit documentation
// boolean canAddRef()
// Determine if the reference count for the interface pointer can be increased.
//

// If you call addRef(), the SDK will try to hold the raw data buffer until the reference becomes 0.
// When you finish using the raw data buffer, you must call releaseRef(); to release it.

// class AudioRawData
// {
// public:
// 	/// \brief Determine if the reference count can be increased.
// 	/// \return TRUE indicates to the reference count can be increased.
// 	virtual bool CanAddRef() = 0;

// 	/// \brief Add one to the reference count.
// 	/// \return If the function succeeds, the return value is TRUE.
// 	virtual bool AddRef() = 0;

// 	/// \brief Subtract one from the reference count.
// 	/// \return The current reference count. If the currrent reference count is 0, the SDK will delete this object instance.
// 	virtual int Release() = 0;

// 	/// \brief Get the audio raw data.
// 	/// \return A pointer to the audio raw data.
// 	virtual char* GetBuffer() = 0;

// 	/// \brief Get the buffer length of the audio raw data.
// 	/// \return The length of the audio raw data.
// 	virtual unsigned int GetBufferLen() = 0;

// 	/// \brief Get the sample rate of the audio raw data.
// 	/// \return The sample rate of the audio raw data.
// 	virtual unsigned int GetSampleRate() = 0;

// 	/// \brief Get the channel number of the audio raw data.
// 	/// \return The channel number of the audio raw data.
// 	virtual unsigned int GetChannelNum() = 0;
// 	virtual ~AudioRawData(){}
// };
