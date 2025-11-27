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

extern "C" const zchar_t* meeting_participants_get_avatar_path(ZOOMSDK::IUserInfo *user_info) {
    return user_info->GetAvatarPath();
}

extern "C" const zchar_t* meeting_participants_get_persistent_id(ZOOMSDK::IUserInfo *user_info) {
    return user_info->GetPersistentId();
}

extern "C" const zchar_t* meeting_participants_get_customer_key(ZOOMSDK::IUserInfo *user_info) {
    return user_info->GetCustomerKey();
}

extern "C" int meeting_participants_get_user_role(ZOOMSDK::IUserInfo *user_info) {
    return static_cast<int>(user_info->GetUserRole());
}

extern "C" int meeting_participants_get_audio_join_type(ZOOMSDK::IUserInfo *user_info) {
    return static_cast<int>(user_info->GetAudioJoinType());
}

extern "C" bool meeting_participants_is_pure_phone_user(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsPurePhoneUser();
}

extern "C" bool meeting_participants_has_camera(ZOOMSDK::IUserInfo *user_info) {
    return user_info->HasCamera();
}

extern "C" bool meeting_participants_is_audio_muted(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsAudioMuted();
}

extern "C" bool meeting_participants_is_video_on(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsVideoOn();
}

extern "C" bool meeting_participants_is_in_waiting_room(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsInWaitingRoom();
}

extern "C" bool meeting_participants_is_raise_hand(ZOOMSDK::IUserInfo *user_info) {
    return user_info->IsRaiseHand();
}

extern "C" ZOOMSDK::IUserInfo *get_my_self_user(ZOOMSDK::IMeetingParticipantsController *controller) {
    return controller->GetMySelfUser();
}

extern "C" bool is_participant_request_local_recording_allowed(ZOOMSDK::IMeetingParticipantsController *controller) {
    return controller->IsParticipantRequestLocalRecordingAllowed();
}
