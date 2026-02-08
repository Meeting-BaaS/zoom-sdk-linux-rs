#include "c_meeting_participants_interface.h"

#include <stdio.h>
#include <cstdlib>

using namespace std;

// template<class T>
// class IList
// {
// public:
// 	virtual ~IList(){};
// 	virtual int GetCount() = 0;
// 	virtual T   GetItem(int index) = 0;
// };

extern "C" struct participant *meeting_participants_get_users(
    ZOOMSDK::IMeetingParticipantsController *controller,
    unsigned int *len
    ) {
    auto id_list = controller->GetParticipantsList();
    if (!id_list) {
        printf("NullPtr GetParticipantsList\n");
        return NULL;
    }
    unsigned int count = id_list->GetCount();
    struct participant *m = (struct participant*)malloc(sizeof(struct participant) * (size_t)count);
    for (unsigned int i = 0; i < count; i += 1) {
        int user_id = id_list->GetItem(i);
        m[i].user_id = user_id;
        m[i].user_info = controller->GetUserByUserID(user_id);
        if (!m[i].user_info) {
            printf("NullPtr GetUserByUserID\n");
            return NULL;
        }
    }
    *len = count;
    return m;
}

extern "C" ZOOMSDK::IUserInfo *meeting_participants_get_user_by_id(
    ZOOMSDK::IMeetingParticipantsController *controller,
    unsigned int userid) {
        return controller->GetUserByUserID(userid);
    }

extern "C" void meeting_participants_free_memory(struct participant *m) {
    free(m);
}

extern "C" bool meeting_participants_is_talking(ZOOMSDK::IUserInfo *user_info) {
    return (user_info->IsTalking());
}

extern "C" const zchar_t* meeting_participants_get_user_name(ZOOMSDK::IUserInfo *user_info) {
    return (user_info->GetUserName());
}

extern "C" int meeting_participants_get_audio_voice_level(ZOOMSDK::IUserInfo *user_info) {
    return (user_info->GetAudioVoiceLevel());
}

extern "C" unsigned int get_user_id(ZOOMSDK::IUserInfo *user_info) {
    return user_info->GetUserID();
}


extern "C" bool is_host(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsHost();
}

extern "C" ZOOMSDK::IUserInfo *get_my_self_user(ZOOMSDK::IMeetingParticipantsController *controller) {
    return controller->GetMySelfUser();
}

extern "C" bool is_participant_request_local_recording_allowed(ZOOMSDK::IMeetingParticipantsController *controller) {
    return controller->IsParticipantRequestLocalRecordingAllowed();
}

// Callback declarations for Rust
extern "C" void on_user_join(void *ptr_to_rust, unsigned int *user_ids, unsigned int count);
extern "C" void on_user_left(void *ptr_to_rust, unsigned int *user_ids, unsigned int count);
extern "C" void on_host_change(void *ptr_to_rust, unsigned int new_host_id);

class C_MeetingParticipantsCtrlEvent : public ZOOMSDK::IMeetingParticipantsCtrlEvent {
public:
    C_MeetingParticipantsCtrlEvent(void *ptr) {
        ptr_to_rust = ptr;
    }

protected:
    void onUserJoin(ZOOMSDK::IList<unsigned int>* lstUserID, const zchar_t* strUserList = nullptr) override {
        (void)strUserList;
        if (!lstUserID) return;

        int count = lstUserID->GetCount();
        if (count <= 0) return;

        unsigned int *user_ids = (unsigned int*)malloc(sizeof(unsigned int) * count);
        for (int i = 0; i < count; i++) {
            user_ids[i] = lstUserID->GetItem(i);
        }
        on_user_join(ptr_to_rust, user_ids, count);
        free(user_ids);
    }

    void onUserLeft(ZOOMSDK::IList<unsigned int>* lstUserID, const zchar_t* strUserList = nullptr) override {
        (void)strUserList;
        if (!lstUserID) return;

        int count = lstUserID->GetCount();
        if (count <= 0) return;

        unsigned int *user_ids = (unsigned int*)malloc(sizeof(unsigned int) * count);
        for (int i = 0; i < count; i++) {
            user_ids[i] = lstUserID->GetItem(i);
        }
        on_user_left(ptr_to_rust, user_ids, count);
        free(user_ids);
    }

    void onHostChangeNotification(unsigned int userId) override {
        on_host_change(ptr_to_rust, userId);
    }

    // Implement other required virtual methods with empty bodies
    void onUserNamesChanged(ZOOMSDK::IList<unsigned int>* lstUserID) override { (void)lstUserID; }
    void onCoHostChangeNotification(unsigned int userId, bool isCoHost) override { (void)userId; (void)isCoHost; }
    void onLowOrRaiseHandStatusChanged(bool bLow, unsigned int userid) override { (void)bLow; (void)userid; }
    void onAllHandsLowered() override {}
    void onLocalRecordingStatusChanged(unsigned int user_id, ZOOMSDK::RecordingStatus status) override { (void)user_id; (void)status; }
    void onInMeetingUserAvatarPathUpdated(unsigned int userID) override { (void)userID; }
    void onParticipantProfilePictureStatusChange(bool bHidden) override { (void)bHidden; }
    void onFocusModeStateChanged(bool bEnabled) override { (void)bEnabled; }
    void onFocusModeShareTypeChanged(ZOOMSDK::FocusModeShareType shareType) override { (void)shareType; }
    void onInvalidReclaimHostkey() override {}
    void onAllowParticipantsRenameNotification(bool bAllow) override { (void)bAllow; }
    void onAllowParticipantsUnmuteSelfNotification(bool bAllow) override { (void)bAllow; }
    void onAllowParticipantsStartVideoNotification(bool bAllow) override { (void)bAllow; }
    void onAllowParticipantsShareWhiteBoardNotification(bool bAllow) override { (void)bAllow; }
    void onRequestLocalRecordingPrivilegeChanged(ZOOMSDK::LocalRecordingRequestPrivilegeStatus status) override { (void)status; }
    void onAllowParticipantsRequestCloudRecording(bool bAllow) override { (void)bAllow; }
    void onBotAuthorizerRelationChanged(unsigned int authorizeUserID) override { (void)authorizeUserID; }
    void onVirtualNameTagStatusChanged(bool bOn, unsigned int userID) override { (void)bOn; (void)userID; }
    void onVirtualNameTagRosterInfoUpdated(unsigned int userID) override { (void)userID; }
    void onGrantCoOwnerPrivilegeChanged(bool canGrantOther) override { (void)canGrantOther; }

private:
    void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError participants_set_event(ZOOMSDK::IMeetingParticipantsController *controller, void *arc_ptr) {
    auto* obj = new C_MeetingParticipantsCtrlEvent(arc_ptr);
    return controller->SetEvent(obj);
}
