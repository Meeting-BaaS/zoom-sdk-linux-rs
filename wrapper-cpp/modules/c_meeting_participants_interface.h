#ifndef _C_MEETING_PARTICIPANTS_INTERFACE_H_
#define _C_MEETING_PARTICIPANTS_INTERFACE_H_

#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_audio_interface.h"
#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_participants_ctrl_interface.h"

/// @brief This structure represents an user with ID and virtual interface.
extern "C" struct participant {
    ZOOMSDK::IUserInfo *user_info;
    int user_id;
};

/// @brief Get Particpants list
/// @param controller A Pointer to ZOOMSDK::IMeetingParticipantsController
/// @param len A Pointer to then length of the returned array
/// @return struct participant* An Array of UserInfo
extern "C" struct participant *meeting_participants_get_users(
    ZOOMSDK::IMeetingParticipantsController *controller,
    unsigned int *len);

/// \brief Get the information of specified user.
/// \param userid Specify the user ID for which you want to get the information.
/// \return If the function succeeds, the return value is a pointer to the IUserInfo. For more details, see \link IUserInfo \endlink.
/// Otherwise the function fails, and the return value is NULL.
/// \remarks Valid for both ZOOM style and user custom interface mode. Valid for both normal user and webinar attendee.
extern "C" ZOOMSDK::IUserInfo *meeting_participants_get_user_by_id(
    ZOOMSDK::IMeetingParticipantsController *controller,
    unsigned int userid);

/// @brief Free Participants list
/// @param m A Pointer to the struct participant array
/// @return void
extern "C" void meeting_participants_free_memory(struct participant *m);

/// @brief Check if a participant is talking
/// @param user_info A Pointer to ZOOMSDK::IUserInfo
/// @return Boolean, true is the user is talking
extern "C" bool meeting_participants_is_talking(ZOOMSDK::IUserInfo *user_info);

/// @brief Get the username matched with the current user information.
/// @param user_info A Pointer to ZOOMSDK::IUserInfo
/// @return If the function succeeds, the return value is the username.
/// Otherwise failed, the return value is NULL.
/// @remarks Valid for both normal user and webinar attendee.
extern "C" const zchar_t* meeting_participants_get_user_name(ZOOMSDK::IUserInfo *user_info);


/// @brief Get the Mic level of the user corresponding to the current information.
/// @return The mic level of the user.
extern "C" int meeting_participants_get_audio_voice_level(ZOOMSDK::IUserInfo *user_info);

/// \brief Get the user ID matched with the current user information.
/// \return If the function succeeds, the return value is the user ID.
/// Otherwise the function fails, and the return value is ZERO(0).
/// \remarks Valid for both normal user and webinar attendee.
extern "C" unsigned int get_user_id(ZOOMSDK::IUserInfo *user_info);

/// @brief Check if a participant is the host
/// @param user_info A Pointer to ZOOMSDK::IUserInfo
/// @return Boolean, true is the user is the host
extern "C" bool is_host(ZOOMSDK::IUserInfo *user_info);

/// \brief Get the information of current user.
/// \return If the function succeeds, the return value is a pointer to the IUserInfo. For more details, see \link IUserInfo \endlink.
/// Otherwise failed, the return value is NULL.
/// \remarks Valid for both ZOOM style and user custom interface mode..
extern "C" ZOOMSDK::IUserInfo *get_my_self_user(ZOOMSDK::IMeetingParticipantsController *controller);

/// \brief Check whether the current meeting allows participants to send local recording privilege request, it can only be used in regular meeetings(no webinar or bo).
/// \return If allows participants to send request, the return value is true.
extern "C" bool is_participant_request_local_recording_allowed(ZOOMSDK::IMeetingParticipantsController *controller);

#endif