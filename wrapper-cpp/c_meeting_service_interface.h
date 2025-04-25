#ifndef _C_MEETING_SERVICE_INTERFACE_H_
#define _C_MEETING_SERVICE_INTERFACE_H_
#include "../zoom-meeting-sdk-linux/h/meeting_service_interface.h"

/// \brief Set meeting service callback event handler.
/// \param pEvent A pointer to the IMeetingServiceEvent that receives the meeting service callback event.
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError meeting_set_event(ZOOMSDK::IMeetingService* meeting_service, void *arc_ptr);

/// \brief Join the meeting.
/// \param joinParam The parameter is used to join meeting. For more details, see \link JoinParam \endlink structure.
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError meeting_join(
    ZOOMSDK::IMeetingService* meeting_service,
    unsigned long int mid,
    zchar_t *userName,
    zchar_t *psw
);

/// \brief Leave meeting.
/// \param leaveCmd Leave meeting command. For more details, see \link LeaveMeetingCmd \endlink enum.
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError meeting_leave(
    ZOOMSDK::IMeetingService* meeting_service,
    ZOOMSDK::LeaveMeetingCmd leaveCmd
);

/// \brief Get the chat controller interface.
/// \return If the function succeeds, the return value is a pointer to IMeetingChatController. Otherwise returns NULL.
extern "C" ZOOMSDK::IMeetingChatController* meeting_get_meeting_chat_controller(
    ZOOMSDK::IMeetingService* meeting_service
);

/// \brief Get the participants controller interface.
/// \return If the function succeeds, the return value is a pointer to IMeetingParticipantsController. Otherwise returns NULL.
extern "C" ZOOMSDK::IMeetingParticipantsController* meeting_get_meeting_participants_controller(
    ZOOMSDK::IMeetingService* meeting_service
);


/// \brief Get the recording controller interface.
/// \return If the function succeeds, the return value is a pointer to IMeetingRecordingController. Otherwise returns NULL.
extern "C" ZOOMSDK::IMeetingRecordingController* meeting_get_meeting_recording_controller(
    ZOOMSDK::IMeetingService* meeting_service
);

/// \brief Get the sharing controller interface.
/// \return If the function succeeds, the return value is a pointer to IMeetingVideoController. Otherwise returns NULL.
extern "C" ZOOMSDK::IMeetingShareController* meeting_get_meeting_share_controller(
    ZOOMSDK::IMeetingService* meeting_service
);

/// \brief Get the audio controller interface.
/// \return If the function succeeds, the return value is a pointer to IMeetingAudioController. Otherwise returns NULL.
extern "C" ZOOMSDK::IMeetingAudioController *meeting_get_meeting_audio_controller(
    ZOOMSDK::IMeetingService* meeting_service
);

#endif