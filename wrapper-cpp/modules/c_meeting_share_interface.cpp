#include "c_meeting_share_interface.h"

extern "C" void on_sharing_status(void *ptr, ZOOMSDK::SharingStatus status, unsigned int userId);

extern "C" void on_lock_share_status(void *ptr, bool bLocked);

extern "C" void on_share_content_notification(void *ptr, ZOOMSDK::ShareInfo *shareInfo);

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

	    /// \brief Callback event of the changed sharing status.
	    /// \param status The values of sharing status. For more details, see \link SharingStatus \endlink enum.
	    /// \param userId Sharer ID.
	    /// \remarks The userId changes according to the status value. When the status value is the Sharing_Self_Send_Begin or Sharing_Self_Send_End, the userId is the user own ID. Otherwise, the value of userId is the sharer ID.
	    void onSharingStatus(ZOOMSDK::SharingStatus status, unsigned int userId) override {
            return on_sharing_status(ptr_to_rust, status, userId);
        }

	    /// \brief Callback event of locked share status.
	    /// \param bLocked TRUE indicates that it is locked. FALSE unlocked.
	    void onLockShareStatus(bool bLocked) override {
            return on_lock_share_status(ptr_to_rust, bLocked);
        }

	    /// \brief Callback event of changed sharing information.
	    /// \param shareInfo Sharing information. For more details, see \link ShareInfo \endlink structure.
	    void onShareContentNotification(ZOOMSDK::ShareInfo& shareInfo) override {
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

    private:
        void *ptr_to_rust;
};

ZOOMSDK::SDKError sharing_set_event(ZOOMSDK::IMeetingShareController* controller, void *arc_ptr) {
    auto* obj = new C_MeetingShareCtrlEvent(arc_ptr); // TODO : Fix memory leak
    return controller->SetEvent(obj);
}