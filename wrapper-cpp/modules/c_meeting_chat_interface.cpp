#include "c_meeting_chat_interface.h"

extern "C" ZOOMSDK::IChatMsgInfoBuilder *meeting_get_chat_message_builder(
    ZOOMSDK::IMeetingChatController *chat_controler
) {
    return chat_controler->GetChatMessageBuilder();
}

extern "C" ZOOMSDK::SDKError meeting_send_chat_message_to(
    ZOOMSDK::IMeetingChatController *chat_controler,
    ZOOMSDK::IChatMsgInfo *msg
) {
    return chat_controler->SendChatMsgTo(msg);
}

extern "C" ZOOMSDK::IChatMsgInfo *meeting_chat_build(
    ZOOMSDK::IChatMsgInfoBuilder *builder,
    zchar_t *content,
    ZOOMSDK::SDKChatMessageType msg_type
) {
    // TODO : Fix this unsafe Sequence
    return builder->SetContent(content)->SetReceiver(0)->SetMessageType(msg_type)->Build();
}