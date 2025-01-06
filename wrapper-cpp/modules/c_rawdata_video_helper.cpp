#include "c_rawdata_video_helper.h"

#include <chrono>
#include <stdio.h>

extern "C" void on_raw_data_frame_received(void *ptr, struct exported_video_raw_data *data);

extern "C" void on_raw_data_status_changed(void *ptr, bool status);

extern "C" void on_renderer_be_destroyed(void *ptr);

class ZoomSDKRendererDelegate : public ZOOMSDK::IZoomSDKRendererDelegate {
public:
    // ZoomSDKRendererDelegate(void *ptr, uint32_t m_user_id) {
    ZoomSDKRendererDelegate(void *ptr) {
        ptr_to_rust = ptr;
        // user_id = m_user_id;
    }
    void onRawDataFrameReceived(YUVRawDataI420* data) override {
        using namespace std::chrono;
        int64_t timestamp = duration_cast<microseconds>(system_clock::now().time_since_epoch()).count();

        struct exported_video_raw_data exported_data = {
            data: data->GetBuffer(),
            time: timestamp,
            len: data->GetBufferLen(),
            user_id: data->GetSourceID(),
            width: data->GetStreamWidth(),
            height: data->GetStreamHeight(),
        };
        on_raw_data_frame_received(ptr_to_rust, &exported_data);
    }
    void onRawDataStatusChanged(RawDataStatus status) override {
        on_raw_data_status_changed(ptr_to_rust, status == RawData_On ? true : false);
    }
    void onRendererBeDestroyed() override {
        on_renderer_be_destroyed(ptr_to_rust);
    }
private:
    void *ptr_to_rust;
    // uint32_t user_id;
};

// SDK_API SDKError createRenderer(IZoomSDKRenderer** ppRenderer, IZoomSDKRendererDelegate* pDelegate);

extern "C" ZOOMSDK::IZoomSDKRendererDelegate* video_helper_create_delegate(void *arc_ptr) {
    auto* obj = new ZoomSDKRendererDelegate(arc_ptr); // TODO : Fix memory leak
    return obj;
}

extern "C" ZOOMSDK::SDKError video_helper_subscribe_delegate(
    ZOOMSDK::IZoomSDKRenderer* ctx,
    uint32_t user_id,
    ZOOMSDK::ZoomSDKRawDataType data_type)
{
        return ctx->subscribe(user_id, data_type);
}

extern "C" ZOOMSDK::SDKError video_helper_unsubscribe_delegate(ZOOMSDK::IZoomSDKRenderer* ctx) {
        return ctx->unSubscribe();
}

extern "C" ZOOMSDK::SDKError set_raw_data_resolution(
    ZOOMSDK::IZoomSDKRenderer* ctx,
    ZOOMSDK::ZoomSDKResolution resolution)
{
    return ctx->setRawDataResolution(resolution);
}