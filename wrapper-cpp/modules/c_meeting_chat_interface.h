#ifndef _C_MEETING_CHAT_INTERFACE_H_
#define _C_MEETING_CHAT_INTERFACE_H_

#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_chat_interface.h"

extern "C" ZOOMSDK::IChatMsgInfoBuilder *meeting_get_chat_message_builder(
    ZOOMSDK::IMeetingChatController *chat_controler
);

extern "C" ZOOMSDK::SDKError meeting_send_chat_message_to(
    ZOOMSDK::IMeetingChatController *chat_controler,
    ZOOMSDK::IChatMsgInfo *msg
);

extern "C" ZOOMSDK::IChatMsgInfo *meeting_chat_build(
    ZOOMSDK::IChatMsgInfoBuilder *builder,
    zchar_t *content,
    ZOOMSDK::SDKChatMessageType msg_type
);

#endif