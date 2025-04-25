#include "c_meeting_audio_interface.h"

extern "C" ZOOMSDK::SDKError meeting_unmute_microphone(
    ZOOMSDK::IMeetingAudioController *audio_controler,
    unsigned int userid
) {
    return audio_controler->UnMuteAudio(userid);
}