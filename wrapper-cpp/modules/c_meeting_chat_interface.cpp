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
    return builder->SetContent(content)->SetReceiver(0)->SetMessageType(msg_type)->Build();
}

// Callback declaration for Rust
extern "C" void on_chat_msg_notification(
    void *ptr_to_rust,
    const char *message_id,
    unsigned int sender_user_id,
    const char *sender_display_name,
    const char *content,
    time_t timestamp
);

class C_MeetingChatCtrlEvent : public ZOOMSDK::IMeetingChatCtrlEvent {
public:
    C_MeetingChatCtrlEvent(void *ptr) {
        ptr_to_rust = ptr;
    }

protected:
    void onChatMsgNotification(ZOOMSDK::IChatMsgInfo* chatMsg, const zchar_t* content = nullptr) override {
        (void)content;
        if (!chatMsg) return;

        const zchar_t* msg_id = chatMsg->GetMessageID();
        unsigned int sender_id = chatMsg->GetSenderUserId();
        const zchar_t* sender_name = chatMsg->GetSenderDisplayName();
        const zchar_t* msg_content = chatMsg->GetContent();
        time_t ts = chatMsg->GetTimeStamp();

        on_chat_msg_notification(
            ptr_to_rust,
            msg_id ? msg_id : "",
            sender_id,
            sender_name ? sender_name : "",
            msg_content ? msg_content : "",
            ts
        );
    }

    void onChatStatusChangedNotification(ZOOMSDK::ChatStatus* status_) override { (void)status_; }
    void onChatMsgDeleteNotification(const zchar_t* msgID, ZOOMSDK::SDKChatMessageDeleteType deleteBy) override { (void)msgID; (void)deleteBy; }
    void onChatMessageEditNotification(ZOOMSDK::IChatMsgInfo* chatMsg) override { (void)chatMsg; }
    void onShareMeetingChatStatusChanged(bool isStart) override { (void)isStart; }
    void onFileSendStart(ZOOMSDK::ISDKFileSender* sender) override { (void)sender; }
    void onFileReceived(ZOOMSDK::ISDKFileReceiver* receiver) override { (void)receiver; }
    void onFileTransferProgress(ZOOMSDK::SDKFileTransferInfo* info) override { (void)info; }

private:
    void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError chat_set_event(ZOOMSDK::IMeetingChatController *controller, void *arc_ptr) {
    auto* obj = new C_MeetingChatCtrlEvent(arc_ptr);
    return controller->SetEvent(obj);
}
