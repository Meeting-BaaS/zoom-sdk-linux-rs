#include "c_meeting_service_interface.h"

extern "C" void on_meeting_status_changed(void *ptr, ZOOMSDK::MeetingStatus status, int iResult);

extern "C" void on_meeting_statistics_warning_notification(void *ptr, ZOOMSDK::StatisticsWarningType type);

extern "C" void on_meeting_parameter_notification(void *ptr, const ZOOMSDK::MeetingParameter* meeting_param);

extern "C" void on_suspend_participants_activities(void *ptr);

extern "C" void on_ai_companion_active_change_notice(void *ptr, int bActive);

extern "C" void on_meeting_topic_changed(void *ptr, const zchar_t* sTopic);

extern "C" void on_meeting_full_to_watch_live_stream(void *ptr, const zchar_t* sLiveStreamUrl);

class C_MeetingServiceEvent: public ZOOMSDK::IMeetingServiceEvent {
    public:
        ~C_MeetingServiceEvent() override {}

        C_MeetingServiceEvent(void *ptr) {
            ptr_to_rust = ptr;
        }

	    void onMeetingStatusChanged(ZOOMSDK::MeetingStatus status, int iResult = 0) {
            return on_meeting_status_changed(ptr_to_rust, status, iResult);
        }

	    void onMeetingStatisticsWarningNotification(ZOOMSDK::StatisticsWarningType type) {
            return on_meeting_statistics_warning_notification(ptr_to_rust, type);
        }

	    void onMeetingParameterNotification(const ZOOMSDK::MeetingParameter* meeting_param) {
            return on_meeting_parameter_notification(ptr_to_rust, meeting_param);
        }

	    void onSuspendParticipantsActivities() {
            return on_suspend_participants_activities(ptr_to_rust);
        }

	    void onAICompanionActiveChangeNotice(bool bActive) {
            return on_ai_companion_active_change_notice(ptr_to_rust, bActive);
        }

	    void onMeetingTopicChanged(const zchar_t* sTopic) {
            return on_meeting_topic_changed(ptr_to_rust, sTopic);
        }

	    void onMeetingFullToWatchLiveStream(const zchar_t* sLiveStreamUrl) {
            return on_meeting_full_to_watch_live_stream(ptr_to_rust, sLiveStreamUrl);
        }

    private:
        void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError meeting_set_event(ZOOMSDK::IMeetingService* meeting_service, void *arc_ptr) {
    auto* obj = new C_MeetingServiceEvent(arc_ptr); // TODO : Fix memory leak
    return meeting_service->SetEvent(obj);
}

extern "C" ZOOMSDK::SDKError meeting_join(
    ZOOMSDK::IMeetingService* meeting_service,
    unsigned long int mid,
    zchar_t *userName,
    zchar_t *psw
) {
    ZOOMSDK::JoinParam joinParam;
    joinParam.userType = ZOOM_SDK_NAMESPACE::SDK_UT_WITHOUT_LOGIN;

    ZOOMSDK::JoinParam4WithoutLogin& param = joinParam.param.withoutloginuserJoin;

    param.meetingNumber = mid;
    param.userName = userName;
    param.psw = psw;
    param.vanityID = nullptr;
    param.customer_key = nullptr;
    param.webinarToken = nullptr;
    param.isVideoOff = false;
    param.isAudioOff = false;

    return meeting_service->Join(joinParam);
}

extern "C" ZOOMSDK::SDKError meeting_leave(
    ZOOMSDK::IMeetingService* meeting_service,
    ZOOMSDK::LeaveMeetingCmd leaveCmd
) {
    return meeting_service->Leave(leaveCmd);
}

extern "C" ZOOMSDK::IMeetingChatController* meeting_get_meeting_chat_controller(
    ZOOMSDK::IMeetingService* meeting_service)
{
    return meeting_service->GetMeetingChatController();
}

extern "C" ZOOMSDK::IMeetingParticipantsController* meeting_get_meeting_participants_controller(
    ZOOMSDK::IMeetingService* meeting_service)
{
    return meeting_service->GetMeetingParticipantsController();
}

extern "C" ZOOMSDK::IMeetingRecordingController* meeting_get_meeting_recording_controller(
    ZOOMSDK::IMeetingService* meeting_service)
{
    return meeting_service->GetMeetingRecordingController();
}