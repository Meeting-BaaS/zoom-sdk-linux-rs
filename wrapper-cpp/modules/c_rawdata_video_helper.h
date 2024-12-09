#ifndef _C_RAWDATA_VIDEO_HELPER_H_
#define _C_RAWDATA_VIDEO_HELPER_H_

#include "../../zoom-meeting-sdk-linux/h/rawdata/zoom_rawdata_api.h"
#include "../../zoom-meeting-sdk-linux/h/rawdata/rawdata_renderer_interface.h"
#include "../../zoom-meeting-sdk-linux/h/zoom_sdk_raw_data_def.h"

struct exported_video_raw_data {
    char *data;
    int64_t time;
    unsigned int len;
    uint32_t user_id;
};

// SDK_API SDKError createRenderer(IZoomSDKRenderer** ppRenderer, IZoomSDKRendererDelegate* pDelegate);

extern "C" ZOOMSDK::IZoomSDKRendererDelegate* video_helper_create_delegate(void *arc_ptr);

extern "C" ZOOMSDK::SDKError video_helper_subscribe_delegate(
    ZOOMSDK::IZoomSDKRenderer* ctx,
    uint32_t user_id,
    ZOOMSDK::ZoomSDKRawDataType data_type);

extern "C" ZOOMSDK::SDKError video_helper_unsubscribe_delegate(ZOOMSDK::IZoomSDKRenderer* ctx);

extern "C" ZOOMSDK::SDKError set_raw_data_resolution(
    ZOOMSDK::IZoomSDKRenderer* ctx,
    ZOOMSDK::ZoomSDKResolution resolution);

#endif