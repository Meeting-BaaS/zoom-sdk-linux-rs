#include "c_rawdata_video_source.h"

#include <iostream>

constexpr auto WIDTH = 640;
constexpr auto HEIGHT = 480;
constexpr auto YUV420_480P_FRAMELENGTH = 460800;

extern "C" ZOOMSDK::SDKError play_video_to_virtual_webcam(
    ZOOMSDK::IZoomSDKVideoSender* video_sender,
    const char* video_source_ptr)
{
    return video_sender->sendVideoFrame(
        (char *)video_source_ptr,
        WIDTH,
        HEIGHT,
        YUV420_480P_FRAMELENGTH,
        0);
}

// Indicate that the webcam diffusion can begin.
extern "C" void video_source_started(void *ptr_to_rust, ZOOMSDK::IZoomSDKVideoSender* sender);

// Indicate to stop the webcam diffusion.
extern "C" void video_source_stopped(void *ptr_to_rust);

class ZoomSDKVideoSource: public ZOOMSDK::IZoomSDKVideoSource {
    public:
	    ~ZoomSDKVideoSource(){}
        ZoomSDKVideoSource(void *ptr_to_rust) {
            ptr_to_rust_ = ptr_to_rust;
        }
    protected:
	    void onInitialize(
            ZOOMSDK::IZoomSDKVideoSender* sender,
            ZOOMSDK::IList<ZOOMSDK::VideoSourceCapability >* _support_cap_list,
            ZOOMSDK::VideoSourceCapability& _suggest_cap)
        {
            printf("ZoomSDKVideoSource::onInitialize()\n");
            (void) _support_cap_list;
            (void) _suggest_cap;
            video_sender_ = sender;
        }
	    void onPropertyChange
            (ZOOMSDK::IList<ZOOMSDK::VideoSourceCapability >* _support_cap_list,
            ZOOMSDK::VideoSourceCapability _suggest_cap)
        {
            printf("ZoomSDKVideoSource::onPropertyChange()\n");
            (void) _support_cap_list;
            (void) _suggest_cap;
        }
	    void onStartSend() override{
            video_source_started(ptr_to_rust_, video_sender_);
        }
	    void onStopSend() override {
            video_source_stopped(ptr_to_rust_);
        }
	    void onUninitialized() override {
            printf("ZoomSDKVideoSource::onUninitialized()\n");
            video_sender_ = nullptr;
        }
    private:
        ZOOMSDK::IZoomSDKVideoSender* video_sender_;
        void *ptr_to_rust_;
};

extern "C" ZOOMSDK::SDKError init_video_to_virtual_webcam(ZOOMSDK::IMeetingService* meeting_service, void *ptr_to_rust)
{
	ZoomSDKVideoSource* virtual_camera_video_source = new ZoomSDKVideoSource(ptr_to_rust);
	ZOOMSDK::IZoomSDKVideoSourceHelper* p_videoSourceHelper = ZOOMSDK::GetRawdataVideoSourceHelper();

	if (p_videoSourceHelper) {
		ZOOMSDK::SDKError err = p_videoSourceHelper->setExternalVideoSource(virtual_camera_video_source);

		if (err != ZOOMSDK::SDKERR_SUCCESS) {
			printf("attemptToStartRawVideoSending(): Failed to set external video source, error code: %d\n", err);
            return err;
		} else {
			ZOOMSDK::IMeetingVideoController* meeting_video_controller = meeting_service->GetMeetingVideoController();
            if (!meeting_video_controller) {
                printf("NullPtr meetingController");
                return ZOOMSDK::SDKERR_INTERNAL_ERROR;
            }
			return meeting_video_controller->UnmuteVideo();
		}
	}
	else {
		printf("attemptToStartRawVideoSending(): Failed to get video source helper\n");
        return ZOOMSDK::SDKERR_INTERNAL_ERROR;
	}
}