#ifndef _C_RAWDATA_VIDEO_SOURCE_H_
#define _C_RAWDATA_VIDEO_SOURCE_H_

#include "meeting_service_interface.h"
#include "../../zoom-meeting-sdk-linux/h/rawdata/zoom_rawdata_api.h"
#include "../../zoom-meeting-sdk-linux/h/rawdata/rawdata_video_source_helper_interface.h"
#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_video_interface.h"

// Init video injection through webcam
extern "C" ZOOMSDK::SDKError init_video_to_virtual_webcam(ZOOMSDK::IMeetingService* meeting_service, void *ptr_to_rust);

// Send frames to webcam
extern "C" ZOOMSDK::SDKError play_video_to_virtual_webcam(
    ZOOMSDK::IZoomSDKVideoSender* video_sender,
    const char* video_source_ptr);

#endif