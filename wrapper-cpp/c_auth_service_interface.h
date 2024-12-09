#ifndef _C_AUTH_SERVICE_INTERFACE_H_
#define _C_AUTH_SERVICE_INTERFACE_H_
#include "../zoom-meeting-sdk-linux/h/auth_service_interface.h"

/// \brief Set the authentication service callback event handler.
/// \param pEvent A pointer to receive authentication event. 
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError auth_set_event(ZOOMSDK::IAuthService* auth_service, void *arc_ptr);

/// \brief SDK Authentication with jwt token.
/// \param authContext The parameter to be used for authentication SDK, see \link AuthContext \endlink structure. 
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError auth_sdk_auth(ZOOMSDK::IAuthService* auth_service, const zchar_t* jwt_token);

/// \brief Get authentication status.
/// \return The return value is authentication status. To get extended error information, see \link AuthResult \endlink enum.
extern "C" ZOOMSDK::AuthResult auth_get_auth_result(ZOOMSDK::IAuthService* auth_service);

/// \brief Get SDK identity.
/// \return The SDK identity.
extern "C" const zchar_t* auth_get_sdk_identity(ZOOMSDK::IAuthService* auth_service);

/// \brief Get SSO login web url.
/// \param prefix_of_vanity_url, prefix of vanity url. 
/// \return SSO login web url
extern "C" const zchar_t* auth_generate_sso_login_web_url(
    ZOOMSDK::IAuthService* auth_service,
    const zchar_t* prefix_of_vanity_url
);

/// \brief Account login.
/// \param uri_protocol For the parameter to be used for sso account login
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
///You need to call this APIs after IAuthServiceEvent::onAuthenticationReturn() return SDKErr_Success.
extern "C" ZOOMSDK::SDKError auth_sso_login_with_web_uri_protocol(
    ZOOMSDK::IAuthService* auth_service,
    const zchar_t* uri_protocol
);
	
/// \brief Account logout.
/// \return If the function succeeds, the return value is SDKErr_Success.
///Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError auth_log_out(ZOOMSDK::IAuthService* auth_service);

/// \brief Get login account information.
/// \return If you has logged in your account successfully, the return value is a pointer to IAccountInfo, otherwise is NULL.
extern "C" ZOOMSDK::IAccountInfo* auth_get_account_info(ZOOMSDK::IAuthService* auth_service);

/// \brief Get login status.
/// \return The return value is login status. To get extended error information, see \link LOGINSTATUS \endlink enum.
extern "C" ZOOMSDK::LOGINSTATUS auth_get_login_status(ZOOMSDK::IAuthService* auth_service);

extern "C" const zchar_t* auth_get_acount_info_display_name(ZOOMSDK::IAccountInfo *account_info);

extern "C" ZOOMSDK::LoginType auth_get_account_info_login_type(ZOOMSDK::IAccountInfo *account_info);

#endif