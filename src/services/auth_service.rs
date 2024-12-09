use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::ptr;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use crate::SdkError;
use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

pub trait AuthServiceEvent: Debug {
    /// Authentication result callback.  
    /// - [AuthResult] Authentication result value.
    fn on_authentification_return(&mut self, _ret: AuthResult) {}

    /// Callback of login result with fail reason.  
    /// - [LoginStatus] Login status.  
    /// - [AccountInfo] Valid when the ret is [LoginStatus::LoginSuccess]. Otherwise None.  
    /// - [LoginFailReason] Login fail reason. Valid when the ret is [LoginStatus::LoginFailed]. Otherwise None.
    fn on_login_return_with_reason(
        &mut self,
        _ret: LoginStatus,
        _account_info: Option<AccountInfo>,
        _log_fail_reason: Option<LoginFailReason>,
    ) {
    }

    /// Logout result callback.
    fn on_logout(&mut self) {}

    /// Zoom identity has expired, please re-login or generate a new zoom access token via REST Api.
    fn on_zoom_identity_expired(&mut self) {}

    /// Zoom authentication identity will be expired in 10 minutes, please re-auth.
    fn on_zoom_auth_identity_expired(&mut self) {}
}

#[derive(Debug)]
pub struct AuthService<'a> {
    pub evt_mutex: Option<Arc<Mutex<Box<dyn AuthServiceEvent>>>>,
    pub ptr_auth_service: &'a mut ZOOMSDK_IAuthService,
}

impl<'a> AuthService<'a> {
    /// Create auth service interface
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [SdkError] for details.
    pub fn new() -> SdkResult<Self> {
        let mut ptr = ptr::null_mut();
        let ret = unsafe { ZOOMSDK_CreateAuthService(&mut ptr) };
        if ret == ZOOMSDK_SDKError_SDKERR_SUCCESS {
            Ok(Self {
                evt_mutex: None,
                ptr_auth_service: unsafe { ptr.as_mut() }.unwrap(),
            })
        } else {
            Err(ZoomRsError::Sdk(ret.into()))
        }
    }
    /// Set the authentication service callback event handler.  
    /// - [AuthServiceEvent] A pointer to receive authentication event.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.  
    pub fn set_event(&mut self, ctx: Box<dyn AuthServiceEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        ZoomSdkResult(unsafe { auth_set_event(self.ptr_auth_service, ptr) }, ()).into()
    }
    /// SDK Authentication with jwt token.  
    /// - [JwtToken] The parameter to be used for authentication SDK.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.  
    pub fn sdk_auth(&mut self, jwt_token: JwtToken) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe { auth_sdk_auth(self.ptr_auth_service, jwt_token.0.as_ptr()) },
            (),
        )
        .into()
    }
    /// Get authentication status.  
    /// - The return value is [AuthResult].
    pub fn get_auth_result(&mut self) -> AuthResult {
        let res = unsafe { auth_get_auth_result(self.ptr_auth_service) };
        res.into()
    }
    /// Get SDK identity.  
    /// - Return The SDK identity.
    pub fn get_sdk_identity(&mut self) -> Option<&CStr> {
        let ptr = unsafe { auth_get_sdk_identity(self.ptr_auth_service) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }
    /// Get SSO login web url.  
    /// - prefix_of_vanity_url, prefix of vanity url.  
    /// - return SSO login web url
    pub fn generate_sso_login_web_url(&mut self, prefix_of_vanity_url: &CStr) -> &CStr {
        unsafe {
            CStr::from_ptr(auth_generate_sso_login_web_url(
                self.ptr_auth_service,
                prefix_of_vanity_url.as_ptr(),
            ) as *mut _)
        }
    }
    /// Account login.  
    /// - uri_protocol For the parameter to be used for sso account login  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.    
    /// - You need to call this APIs after [AuthServiceEvent::on_authentification_return] return AuthretSuccess.
    pub fn sso_login_with_web_uri_protocol(&mut self, uri_protocol: &CStr) -> SdkResult<()> {
        ZoomSdkResult(
            unsafe {
                auth_sso_login_with_web_uri_protocol(self.ptr_auth_service, uri_protocol.as_ptr())
            },
            (),
        )
        .into()
    }
    /// Account logout.  
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [SdkError] for details.  
    pub fn log_out(&mut self) -> SdkResult<()> {
        ZoomSdkResult(unsafe { auth_log_out(self.ptr_auth_service) }, ()).into()
    }
    /// Get login account information.  
    /// - [AccountInfo] If you has logged in your account successfully, otherwise is None.
    pub fn get_account_info(&mut self) -> Option<AccountInfo> {
        let ptr = unsafe { auth_get_account_info(self.ptr_auth_service) };
        account_info_from_ptr_accountinfo(ptr)
    }
    /// Get login status.  
    /// - The return value [LoginStatus]
    pub fn get_login_status(&mut self) -> LoginStatus {
        unsafe { auth_get_login_status(self.ptr_auth_service) }.into()
    }
}

impl<'a> Drop for AuthService<'a> {
    fn drop(&mut self) {
        let ret = unsafe { ZOOMSDK_DestroyAuthService(self.ptr_auth_service) };
        if ret != ZOOMSDK_SDKError_SDKERR_SUCCESS {
            tracing::warn!("Error when droping AuthService : {:?}", ret);
        } else {
            tracing::info!("Auth instance droped!");
        }
    }
}

fn account_info_from_ptr_accountinfo<'a>(
    ptr: *mut ZOOMSDK_IAccountInfo,
) -> Option<AccountInfo<'a>> {
    if ptr.is_null() {
        None
    } else {
        unsafe {
            Some(AccountInfo {
                display_name: {
                    let s = auth_get_acount_info_display_name(ptr);
                    if s.is_null() {
                        None
                    } else {
                        Some(CStr::from_ptr(ptr as _))
                    }
                },
                login_type: auth_get_account_info_login_type(ptr).into(),
            })
        }
    }
}
#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn auth_on_authentification_return(ptr: *const u8, ret: ZOOMSDK_AuthResult) {
    (*convert(ptr).lock().unwrap()).on_authentification_return(ret.into());
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn auth_on_login_return_with_reason(
    ptr: *const u8,
    ret: ZOOMSDK_LOGINSTATUS,
    p_account_info: *mut ZOOMSDK_IAccountInfo,
    reason: ZOOMSDK_LoginFailReason,
) {
    let ret = ret.into();
    let account_info = if let LoginStatus::LoginSuccess = ret {
        account_info_from_ptr_accountinfo(p_account_info)
    } else {
        None
    };
    let reason = if ret == LoginStatus::LoginFailed {
        Some(reason.into())
    } else {
        None
    };
    (*convert(ptr).lock().unwrap()).on_login_return_with_reason(ret, account_info, reason);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn auth_on_logout(ptr: *const u8) {
    (*convert(ptr).lock().unwrap()).on_logout();
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn auth_on_zoom_identity_expired(ptr: *const u8) {
    (*convert(ptr).lock().unwrap()).on_zoom_identity_expired();
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn auth_on_zoom_auth_identity_expired(ptr: *const u8) {
    (*convert(ptr).lock().unwrap()).on_zoom_auth_identity_expired();
}

#[inline]
fn convert(ptr: *const u8) -> Arc<Mutex<Box<dyn AuthServiceEvent>>> {
    let ptr: *const Mutex<Box<dyn AuthServiceEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Account information interface.
#[derive(Debug)]
pub struct AccountInfo<'a> {
    display_name: Option<&'a CStr>,
    login_type: LoginType,
}

impl<'a> AccountInfo<'a> {
    /// Get the screen name of user.  
    /// The return value is the displayed username.  
    /// If there is no screen name of user, the return value is None.
    pub fn get_display_name(&self) -> Option<&CStr> {
        self.display_name
    }
    /// Get login type.  
    /// The return value is the account [LoginType].
    pub fn get_login_type(&self) -> LoginType {
        self.login_type
    }
}

/// SDK Authentication Result.
///
/// This enumeration represents the different SDK auth messages.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum AuthResult {
    /// Authentication is successful.
    AuthretSuccess = ZOOMSDK_AuthResult_AUTHRET_SUCCESS,
    /// The key or secret to authenticate is empty.
    AuthretKeyOrSecretEmpty = ZOOMSDK_AuthResult_AUTHRET_KEYORSECRETEMPTY,
    /// The key or secret to authenticate is wrong.
    AuthretKeyOrSecretWrong = ZOOMSDK_AuthResult_AUTHRET_KEYORSECRETWRONG,
    /// The user account does not support.
    AuthretAccountNotSupport = ZOOMSDK_AuthResult_AUTHRET_ACCOUNTNOTSUPPORT,
    /// The user account is not enabled for SDK.
    AuthretAccountNotEnableSDK = ZOOMSDK_AuthResult_AUTHRET_ACCOUNTNOTENABLESDK,
    /// Unknown error.
    AuthretUnknown = ZOOMSDK_AuthResult_AUTHRET_UNKNOWN,
    /// Service is busy.
    AuthretServiceBusy = ZOOMSDK_AuthResult_AUTHRET_SERVICE_BUSY,
    /// Initial status.
    AuthretNone = ZOOMSDK_AuthResult_AUTHRET_NONE,
    /// Time out.
    AuthretOvertime = ZOOMSDK_AuthResult_AUTHRET_OVERTIME,
    /// Network issues.
    AuthretNetworkIssue = ZOOMSDK_AuthResult_AUTHRET_NETWORKISSUE,
    /// Client is incompatible.
    AuthretClientIncompatible = ZOOMSDK_AuthResult_AUTHRET_CLIENT_INCOMPATIBLE,
    /// The jwt token to authenticate is wrong.
    AuthretJwtTokenWrong = ZOOMSDK_AuthResult_AUTHRET_JWTTOKENWRONG,
    /// The authentication rate limit is exceeded.
    AuthretLimitExceededException = ZOOMSDK_AuthResult_AUTHRET_LIMIT_EXCEEDED_EXCEPTION,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_AuthResult_AUTHRET_LIMIT_EXCEEDED_EXCEPTION + 1,
}

impl From<u32> for AuthResult {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_AuthResult_AUTHRET_LIMIT_EXCEEDED_EXCEPTION => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// SDK Login Failure Reasons.
///
/// This enumeration represents the different reasons for SDK login failure.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)] // Using `repr(u32)` to match the C representation.
pub enum LoginFailReason {
    /// No login failure.
    LoginFailNone = ZOOMSDK_LoginFailReason_LoginFail_None,
    /// Email login is disabled.
    LoginFailEmailLoginDisable = ZOOMSDK_LoginFailReason_LoginFail_EmailLoginDisable,
    /// User does not exist.
    LoginFailUserNotExist = ZOOMSDK_LoginFailReason_LoginFail_UserNotExist,
    /// Incorrect password.
    LoginFailWrongPassword = ZOOMSDK_LoginFailReason_LoginFail_WrongPassword,
    /// Account is locked.
    LoginFailAccountLocked = ZOOMSDK_LoginFailReason_LoginFail_AccountLocked,
    /// SDK requires an update.
    LoginFailSdkNeedUpdate = ZOOMSDK_LoginFailReason_LoginFail_SDKNeedUpdate,
    /// Too many failed attempts.
    LoginFailTooManyFailedAttempts = ZOOMSDK_LoginFailReason_LoginFail_TooManyFailedAttempts,
    /// SMS code error.
    LoginFailSmsCodeError = ZOOMSDK_LoginFailReason_LoginFail_SMSCodeError,
    /// SMS code has expired.
    LoginFailSmsCodeExpired = ZOOMSDK_LoginFailReason_LoginFail_SMSCodeExpired,
    /// Phone number format is invalid.
    LoginFailPhoneNumberFormatInvalid = ZOOMSDK_LoginFailReason_LoginFail_PhoneNumberFormatInValid,
    /// Login token is invalid.
    LoginFailLoginTokenInvalid = ZOOMSDK_LoginFailReason_LoginFail_LoginTokenInvalid,
    /// User disagrees with the login disclaimer.
    LoginFailUserDisagreeLoginDisclaimer =
        ZOOMSDK_LoginFailReason_LoginFail_UserDisagreeLoginDisclaimer,
    /// Multi-factor authentication required.
    LoginFailMfaRequired = ZOOMSDK_LoginFailReason_LoginFail_Mfa_Required,
    /// Need to ask for the user's birthday.
    LoginFailNeedBirthdayAsk = ZOOMSDK_LoginFailReason_LoginFail_Need_Bitrthday_ask,
    /// Other issue.
    LoginFailOtherIssue = ZOOMSDK_LoginFailReason_LoginFail_OtherIssue,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_LoginFailReason_LoginFail_Need_Bitrthday_ask + 1,
}

impl From<u32> for LoginFailReason {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_LoginFailReason_LoginFail_Need_Bitrthday_ask
                || u == ZOOMSDK_LoginFailReason_LoginFail_OtherIssue =>
            unsafe { std::mem::transmute::<u32, Self>(u) },
            _ => Self::Unexpected,
        }
    }
}

/// SDK Login Status.
///
/// This enumeration represents the different statuses of the SDK login process.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)] // Using `repr(u32)` to match the C representation.
pub enum LoginStatus {
    /// Unlogged in.
    LoginIdle = ZOOMSDK_LOGINSTATUS_LOGIN_IDLE,
    /// In process of login.
    LoginProcessing = ZOOMSDK_LOGINSTATUS_LOGIN_PROCESSING,
    /// Login successful.
    LoginSuccess = ZOOMSDK_LOGINSTATUS_LOGIN_SUCCESS,
    /// Login failed.
    LoginFailed = ZOOMSDK_LOGINSTATUS_LOGIN_FAILED,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_LOGINSTATUS_LOGIN_FAILED + 1,
}

impl From<u32> for LoginStatus {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_LOGINSTATUS_LOGIN_FAILED => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// SDK Login Type.
///
/// This enumeration represents the different types of user login.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)] // Using `repr(u32)` to match the C representation.
pub enum LoginType {
    /// Unknown type.
    Unknown = ZOOMSDK_LoginType_LoginType_Unknown,
    /// Login with SSO token.
    SSO = ZOOMSDK_LoginType_LoginType_SSO,
    /// Unexpected result from SDK
    Unexpected = ZOOMSDK_LoginType_LoginType_SSO + 1,
}

impl From<u32> for LoginType {
    fn from(u: u32) -> Self {
        match u {
            u if u <= ZOOMSDK_LoginType_LoginType_SSO => unsafe {
                std::mem::transmute::<u32, Self>(u)
            },
            _ => Self::Unexpected,
        }
    }
}

/// !JWT token. You may generate your JWT token using the online tool.
pub struct JwtToken(pub CString);
