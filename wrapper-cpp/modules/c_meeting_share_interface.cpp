#include "c_meeting_share_interface.h"

extern "C" void on_sharing_status(void *ptr, ZOOMSDK::SharingStatus status, unsigned int userId, unsigned int shareSourceId);

extern "C" void on_lock_share_status(void *ptr, bool bLocked);

extern "C" void on_share_content_notification(void *ptr, ZOOMSDK::SharingStatus status, unsigned int userId, unsigned int shareSourceId);

extern "C" void on_multi_share_switch_to_single_share_need_confirm(void *ptr, ZOOMSDK::IShareSwitchMultiToSingleConfirmHandler* handler_);

extern "C" void on_share_setting_type_changed_notification(void *ptr, ZOOMSDK::ShareSettingType type);

extern "C" void on_shared_video_ended(void *ptr);

extern "C" void on_video_file_share_play_error(void *ptr, ZOOMSDK::ZoomSDKVideoFileSharePlayError error);

class C_MeetingShareCtrlEvent: public ZOOMSDK::IMeetingShareCtrlEvent {
    public:
        ~C_MeetingShareCtrlEvent() override {}

        C_MeetingShareCtrlEvent(void *ptr) {
            ptr_to_rust = ptr;
        }

	    void onSharingStatus(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            return on_sharing_status(ptr_to_rust, shareInfo.status, shareInfo.userid, shareInfo.shareSourceID);
        }

	    void onFailedToStartShare() override {
            // Not forwarded to Rust
        }

	    void onLockShareStatus(bool bLocked) override {
            return on_lock_share_status(ptr_to_rust, bLocked);
        }

	    void onShareContentNotification(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            return on_share_content_notification(ptr_to_rust, shareInfo.status, shareInfo.userid, shareInfo.shareSourceID);
        }

        void onMultiShareSwitchToSingleShareNeedConfirm(ZOOMSDK::IShareSwitchMultiToSingleConfirmHandler* handler_) override {
            return on_multi_share_switch_to_single_share_need_confirm(ptr_to_rust, handler_);
        }

	    void onShareSettingTypeChangedNotification(ZOOMSDK::ShareSettingType type) override {
            return on_share_setting_type_changed_notification(ptr_to_rust, type);
        }

	    void onSharedVideoEnded() override {
            return on_shared_video_ended(ptr_to_rust);
        }

	    void onVideoFileSharePlayError(ZOOMSDK::ZoomSDKVideoFileSharePlayError error) override {
            return on_video_file_share_play_error(ptr_to_rust, error);
        }

	    void onOptimizingShareForVideoClipStatusChanged(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            (void)shareInfo;
            // Not forwarded to Rust
        }

    private:
        void *ptr_to_rust;
};

ZOOMSDK::SDKError sharing_set_event(ZOOMSDK::IMeetingShareController* controller, void *arc_ptr) {
    auto* obj = new C_MeetingShareCtrlEvent(arc_ptr); // TODO : Fix memory leak
    return controller->SetEvent(obj);
}