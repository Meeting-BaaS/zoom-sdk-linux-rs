#include "c_meeting_share_interface.h"

extern "C" void on_sharing_status(void *ptr, ZOOMSDK::ZoomSDKSharingSourceInfo *shareInfo);

extern "C" void on_failed_to_start_share(void *ptr);

extern "C" void on_lock_share_status(void *ptr, bool bLocked);

extern "C" void on_share_content_notification(void *ptr, ZOOMSDK::ZoomSDKSharingSourceInfo *shareInfo);

extern "C" void on_multi_share_switch_to_single_share_need_confirm(void *ptr, ZOOMSDK::IShareSwitchMultiToSingleConfirmHandler* handler_);

extern "C" void on_share_setting_type_changed_notification(void *ptr, ZOOMSDK::ShareSettingType type);

extern "C" void on_shared_video_ended(void *ptr);

extern "C" void on_video_file_share_play_error(void *ptr, ZOOMSDK::ZoomSDKVideoFileSharePlayError error);

extern "C" void on_optimizing_share_for_video_clip_status_changed(void *ptr, ZOOMSDK::ZoomSDKSharingSourceInfo *shareInfo);

class C_MeetingShareCtrlEvent: public ZOOMSDK::IMeetingShareCtrlEvent {
    public:
        ~C_MeetingShareCtrlEvent() override {}

        C_MeetingShareCtrlEvent(void *ptr) {
            ptr_to_rust = ptr;
        }

	    /// \brief Callback event of the changed sharing status.
	    /// \param shareInfo Sharing information.
	    void onSharingStatus(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            return on_sharing_status(ptr_to_rust, &shareInfo);
        }

	    /// \brief Callback event of failure to start sharing.
	    void onFailedToStartShare() override {
            return on_failed_to_start_share(ptr_to_rust);
        }

	    /// \brief Callback event of locked share status.
	    /// \param bLocked TRUE indicates that it is locked. FALSE unlocked.
	    void onLockShareStatus(bool bLocked) override {
            return on_lock_share_status(ptr_to_rust, bLocked);
        }

	    /// \brief Callback event of changed sharing information.
	    /// \param shareInfo Sharing information.
	    void onShareContentNotification(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            return on_share_content_notification(ptr_to_rust, &shareInfo);
        }

	    /// \brief Callback event of switching multi-participants share to one participant share.
	    /// \param handler_ An object pointer used by user to complete all the related operations. For more details, see \link IShareSwitchMultiToSingleConfirmHandler \endlink.
        void onMultiShareSwitchToSingleShareNeedConfirm(ZOOMSDK::IShareSwitchMultiToSingleConfirmHandler* handler_) override {
            return on_multi_share_switch_to_single_share_need_confirm(ptr_to_rust, handler_);
        }

	    /// \brief Callback event of sharing setting type changed.
	    /// \param type Sharing setting type. For more details, see \link ShareSettingType \endlink structure.
	    void onShareSettingTypeChangedNotification(ZOOMSDK::ShareSettingType type) override {
            return on_share_setting_type_changed_notification(ptr_to_rust, type);
        }

	    /// \brief Callback event of the shared video's playback has completed.
	    void onSharedVideoEnded() override {
            return on_shared_video_ended(ptr_to_rust);
        }

	    /// \brief Callback event of the video file playback error.
	    /// \param error The error type. For more details, see \link ZoomSDKVideoFileSharePlayError \endlink structure.
	    void onVideoFileSharePlayError(ZOOMSDK::ZoomSDKVideoFileSharePlayError error) override {
            return on_video_file_share_play_error(ptr_to_rust, error);
        }

	    /// \brief Callback event of the changed optimizing video status.
	    /// \param shareInfo Sharing information.
	    void onOptimizingShareForVideoClipStatusChanged(ZOOMSDK::ZoomSDKSharingSourceInfo shareInfo) override {
            return on_optimizing_share_for_video_clip_status_changed(ptr_to_rust, &shareInfo);
        }

    private:
        void *ptr_to_rust;
};

ZOOMSDK::SDKError sharing_set_event(ZOOMSDK::IMeetingShareController* controller, void *arc_ptr) {
    auto* obj = new C_MeetingShareCtrlEvent(arc_ptr); // TODO : Fix memory leak
    return controller->SetEvent(obj);
}
