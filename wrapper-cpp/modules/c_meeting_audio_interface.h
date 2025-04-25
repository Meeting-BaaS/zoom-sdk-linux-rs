#ifndef _C_MEETING_AUDIO_INTERFACE_H_
#define _C_MEETING_AUDIO_INTERFACE_H_

#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_audio_interface.h"

extern "C" ZOOMSDK::SDKError meeting_unmute_microphone(
    ZOOMSDK::IMeetingAudioController *audio_controler,
    unsigned int userid
);

#endif