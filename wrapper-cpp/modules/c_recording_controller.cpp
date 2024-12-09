#include "c_recording_controller.h"
#include <stdio.h>
#include <chrono>

extern "C" ZOOMSDK::SDKError recording_request_local_recording_privilege(ZOOMSDK::IMeetingRecordingController *ctrl) {
    return (ctrl->RequestLocalRecordingPrivilege());
}

extern "C" ZOOMSDK::SDKError recording_request_start_cloud_recording(ZOOMSDK::IMeetingRecordingController *ctrl) {
    return (ctrl->RequestStartCloudRecording());
}

extern "C" ZOOMSDK::SDKError recording_start_recording(ZOOMSDK::IMeetingRecordingController *ctrl, time_t *startTimestamp) {
    return (ctrl->StartRecording(*startTimestamp));
}

extern "C" ZOOMSDK::SDKError recording_stop_recording(ZOOMSDK::IMeetingRecordingController *ctrl, time_t *stopTimestamp) {
    return (ctrl->StopRecording(*stopTimestamp));
}

extern "C" bool recording_can_start_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl) {
    return ctrl->CanStartRawRecording();
}

extern "C" ZOOMSDK::SDKError recording_start_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl) {
    return ctrl->StartRawRecording();
}

extern "C" ZOOMSDK::SDKError recording_stop_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl) {
    return ctrl->StopRawRecording();
}

extern "C" void on_recording_privilege_request_status(void *ptr_to_rust, ZOOMSDK::RequestLocalRecordingStatus status);

extern "C" void on_recording_status(void *ptr_to_rust, ZOOMSDK::RecordingStatus status, int64_t timestamp);

class C_MeetingRecordingCtrlEvent: public ZOOMSDK::IMeetingRecordingCtrlEvent {
    public:
        C_MeetingRecordingCtrlEvent(void *ptr) {
            ptr_to_rust = ptr;
        }
    protected:
	    /// \brief Callback event that the status of my local recording changes.
	    /// \param status Value of recording status. For more details, see \link RecordingStatus \endlink enum.
	    void onRecordingStatus(ZOOMSDK::RecordingStatus status) {
            using namespace std::chrono;
            int64_t timestamp = duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();

            printf("onRecordingStatus %i\n", status);
            on_recording_status(ptr_to_rust, status, timestamp);
        }
	    /// \brief Callback event that the status of cloud recording changes.
	    /// \param status Value of recording status. For more details, see \link RecordingStatus \endlink enum.
	    void onCloudRecordingStatus(ZOOMSDK::RecordingStatus status) {
            (void) status;
        }
	    /// \brief Callback event that the recording authority changes.
	    /// \param bCanRec TRUE indicates to enable to record.
	    void onRecordPrivilegeChanged(bool bCanRec) {
            (void) bCanRec;
        }
	    /// \brief Callback event that the status of request local recording privilege.
	    /// \param status Value of request local recording privilege status. For more details, see \link RequestLocalRecordingStatus \endlink enum.
	    void onLocalRecordingPrivilegeRequestStatus(ZOOMSDK::RequestLocalRecordingStatus status) {
            printf("onLocalRecordingPrivilegeRequestStatus %i\n", status);
            on_recording_privilege_request_status(ptr_to_rust, status);
        }
	    /// \brief Callback event for when the host responds to a cloud recording permission request
	    /// \param status Value of request host to start cloud recording response status. For more details, see \link RequestStartCloudRecordingStatus \endlink enum.
	    void onRequestCloudRecordingResponse(ZOOMSDK::RequestStartCloudRecordingStatus status) {
            (void) status;
        }
	    /// \brief Callback event when a user requests local recording privilege.
	    /// \param handler A pointer to the IRequestLocalRecordingPrivilegeHandler. For more details, see \link IRequestLocalRecordingPrivilegeHandler \endlink.
	    void onLocalRecordingPrivilegeRequested(ZOOMSDK::IRequestLocalRecordingPrivilegeHandler* handler) {
            (void) handler;
        }
	    /// \brief Callback event received only by the host when a user requests to start cloud recording.
	    /// \param handler A pointer to the IRequestStartCloudRecordingHandler. For more details, see \link IRequestStartCloudRecordingHandler \endlink.
	    void onStartCloudRecordingRequested(ZOOMSDK::IRequestStartCloudRecordingHandler* handler) {
            (void) handler;
        }
	    /// \brief Callback event that the cloud recording storage is full.
	    /// \param gracePeriodDate a point in time, in milliseconds, in UTC. You can use the cloud recording storage until the gracePeriodDate.
	    void onCloudRecordingStorageFull(time_t gracePeriodDate) {
            (void) gracePeriodDate;
        }
	    /// \brief Callback event received only by the host when a user requests to enable and start smart cloud recording.
	    /// \param handler A pointer to the IRequestEnableAndStartSmartRecordingHandler. For more details, see \link IRequestEnableAndStartSmartRecordingHandler \endlink.
	    void onEnableAndStartSmartRecordingRequested(ZOOMSDK::IRequestEnableAndStartSmartRecordingHandler* handler) {
             (void) handler;
        }
	    /// \brief Callback event received when you call \link EnableSmartRecording \endlink. You can use the handler to confirm or cancel enabling the smart recording.
	    /// \param handler A pointer to the ISmartRecordingEnableActionHandler. For more details, see \link ISmartRecordingEnableActionHandler \endlink.
	    void onSmartRecordingEnableActionCallback(ZOOMSDK::ISmartRecordingEnableActionHandler* handler) {
             (void) handler;
        }
	    void onTranscodingStatusChanged(ZOOMSDK::TranscodingStatus status,const zchar_t* path) {
            (void) status;
            (void) path;
        }
    private:
        void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError recording_set_event(ZOOMSDK::IMeetingRecordingController *ctrl, void *arc_ptr) {
    printf("SetEvent begin\n");
    auto* obj = new C_MeetingRecordingCtrlEvent(arc_ptr); // TODO : Fix memory leak
    auto o = ctrl->SetEvent(obj);
    printf("SetEvent end\n");
    return o;
}