#include "c_auth_service_interface.h"

extern "C" void auth_on_authentification_return(void* ptr, ZOOMSDK::AuthResult auth_result);

extern "C" void auth_on_login_return_with_reason(
    void *ptr,
    ZOOMSDK::LOGINSTATUS ret,
    ZOOMSDK::IAccountInfo* pAccountInfo,
    ZOOMSDK::LoginFailReason reason
);

extern "C" void auth_on_logout(void *ptr);

extern "C" void auth_on_zoom_identity_expired(void *ptr);

extern "C" void auth_on_zoom_auth_identity_expired(void *ptr);

class C_AuthServiceEvent: public ZOOMSDK::IAuthServiceEvent {
    public:
        ~C_AuthServiceEvent() override {}

        C_AuthServiceEvent(void *ptr) {
            ptr_to_rust = ptr;
        }

	    void onAuthenticationReturn(ZOOMSDK::AuthResult ret) override {
            auth_on_authentification_return(ptr_to_rust, ret);
        }

        void onLoginReturnWithReason(
            ZOOMSDK::LOGINSTATUS ret,
            ZOOMSDK::IAccountInfo* pAccountInfo,
            ZOOMSDK::LoginFailReason reason) override {
            auth_on_login_return_with_reason(ptr_to_rust, ret, pAccountInfo, reason);
        }

	    void onLogout() override {
            auth_on_logout(ptr_to_rust);
        }

	    void onZoomIdentityExpired() override {
            auth_on_zoom_identity_expired(ptr_to_rust);
        }

	    void onZoomAuthIdentityExpired() override {
            auth_on_zoom_auth_identity_expired(ptr_to_rust);
        }
    private:
        void *ptr_to_rust;
};

extern "C" ZOOMSDK::SDKError auth_set_event(ZOOMSDK::IAuthService* auth_service, void *arc_ptr) {
    auto* obj = new C_AuthServiceEvent(arc_ptr); // TODO : Fix memory leak
    return auth_service->SetEvent(obj);
}

extern "C" ZOOMSDK::SDKError auth_sdk_auth(ZOOMSDK::IAuthService* auth_service, const zchar_t* jwt_token ){
    ZOOMSDK::AuthContext authContext;
    authContext.jwt_token = jwt_token;

    return auth_service->SDKAuth(authContext);
}

extern "C" ZOOMSDK::AuthResult auth_get_auth_result(ZOOMSDK::IAuthService* auth_service) {
    return auth_service->GetAuthResult();
}

extern "C" const zchar_t* auth_get_sdk_identity(ZOOMSDK::IAuthService* auth_service) {
    return auth_service->GetSDKIdentity();
}

extern "C" const zchar_t* auth_generate_sso_login_web_url(
    ZOOMSDK::IAuthService* auth_service,
    const zchar_t* prefix_of_vanity_url
) {
    return auth_service->GenerateSSOLoginWebURL(prefix_of_vanity_url);
}

extern "C" ZOOMSDK::SDKError auth_sso_login_with_web_uri_protocol(
    ZOOMSDK::IAuthService* auth_service,
    const zchar_t* uri_protocol
) {
    return auth_service->SSOLoginWithWebUriProtocol(uri_protocol);
}
	
extern "C" ZOOMSDK::SDKError auth_log_out(ZOOMSDK::IAuthService* auth_service) {
    return auth_service->LogOut();
}

extern "C" ZOOMSDK::IAccountInfo* auth_get_account_info(ZOOMSDK::IAuthService* auth_service) {
    return auth_service->GetAccountInfo();
}

extern "C" ZOOMSDK::LOGINSTATUS auth_get_login_status(ZOOMSDK::IAuthService* auth_service) {
    return auth_service->GetLoginStatus();
}

extern "C" const zchar_t* auth_get_acount_info_display_name(ZOOMSDK::IAccountInfo *account_info) {
    return account_info->GetDisplayName();
}

extern "C" ZOOMSDK::LoginType auth_get_account_info_login_type(ZOOMSDK::IAccountInfo *account_info) {
    return account_info->GetLoginType();

}