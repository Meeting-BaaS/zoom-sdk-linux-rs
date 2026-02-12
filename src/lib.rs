//! **RUST BINDINGS FOR ZOOM SDK LINUX**
//!
//! This project provides Rust bindings for the Zoom SDK on Linux, specifically tailored for integration with the Zoom meeting functionalities. Below are instructions to set up the environment and get everything working smoothly on a Debian-based system.
//!
//! ## **Installation**
//!
//! ### Step 1: Install Required Dependencies
//! Before starting, make sure you have the necessary libraries installed:
//!
//! ```bash
//! apt install libxcb libglib2.0-0 libglib2.0-dev patchelf
//! ```
//!
//! ### Step 2: Obtain the Zoom Linux SDK
//! Download the Linux version of the Zoom SDK from the official [Zoom Marketplace](https://zoom.us) and place the files in the `zoom-meeting-sdk-linux` directory of your project.
//!
//! ### Step 3: Create a Symbolic Link for the Main Library
//! Inside the `zoom-meeting-sdk-linux` directory, create a symbolic link to the main library file `libmeetingsdk.so`:
//!
//! ```bash
//! cd zoom-meeting-sdk-linux && ln -s libmeetingsdk.so libmeetingsdk.so.1
//! ```
//!
//! ### Step 4: Add Missing Dependency to the Library
//! The Zoom SDK library may have a missing dependency. Add it manually using `patchelf`:
//!
//! ```bash
//! patchelf --add-needed /usr/lib/x86_64-linux-gnu/libgio-2.0.so libmeetingsdk.so
//! ```
//!
//! ## **Launching the Application**
//!
//! The Zoom SDK includes modified Qt libraries that are essential for proper execution. Use the following command to launch your Rust project, ensuring the modified Qt libraries are included in the library path:
//!
//! ```bash
//! LD_LIBRARY_PATH=zoom-meeting-sdk-linux/qt_libs/Qt/lib:$LD_LIBRARY_PATH cargo test
//! ```
//!
//! With these steps, you should be able to compile and run your Rust application with the Zoom SDK on Linux. Enjoy integrating advanced video conferencing features with Rust!
//!
//! ### **Modification to regenerate bindings with bindgen**
//! ```c
//! ```
//! Add `#include <ctime>` to meeting_ai_companion_interface.h & meeting_chat_interface.h
#![deny(missing_docs)]

#[allow(nonstandard_style)]
#[allow(unused)]
mod bindings;

use std::ffi::{CStr, CString};
use std::ops::Drop;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};

use auth_service::AuthService;
use bindings::*;

/// Global flag set when the SDK fires MeetingStatusDisconnecting.
/// After this point, the SDK begins internal teardown and frees renderer/audio objects.
/// All code that touches SDK raw data objects (renderer, audio helper) must check this
/// flag and skip operations on potentially-freed pointers.
///
/// Set directly in the C callback (on the SDK thread, before returning) to avoid
/// the 50ms glib main-loop latency that caused a race condition / double-free.
static SDK_TEARDOWN_STARTED: AtomicBool = AtomicBool::new(false);

/// Mark that the SDK has started teardown (called from `on_meeting_status_changed`
/// when `MeetingStatusDisconnecting` is received).
pub fn mark_sdk_teardown() {
    SDK_TEARDOWN_STARTED.store(true, Ordering::SeqCst);
    tracing::warn!("SDK teardown flag set — SDK is disconnecting, skipping further raw-data operations");
}

/// Check whether the SDK teardown has started.
pub fn is_sdk_tearing_down() -> bool {
    SDK_TEARDOWN_STARTED.load(Ordering::SeqCst)
}

/// Allows obtaining a new JWT token.
pub mod jwt_helper;
pub use jwt_helper::generate_jwt;
/// Parse the meeting URL.
pub mod meeting_url;
pub use meeting_url::parse;

/// This module contains the types exported by the library.
pub mod public_types;
use meeting_service::MeetingService;
pub use public_types::*;

/// This module contains all the root services of the SDK.
pub mod services;
pub use services::*;

/// This module handles the raw audio and video data received by the SDK.
pub mod rawdata;

pub use glib;
use setting_service::SettingService;

/// Get the version of ZOOM SDK. Return The version of ZOOM SDK
/// ```rust
/// use zoom_sdk_linux_rs::get_sdk_version;
///
/// assert_eq!(&get_sdk_version(), "6.2.5 (2487)");
/// ```
pub fn get_sdk_version() -> String {
    let a = unsafe { CStr::from_ptr(ZOOMSDK_GetSDKVersion()) };
    a.to_str().unwrap().to_owned()
}

/// Main Zoom instance
#[derive(Debug)]
pub struct Instance<'a> {
    raw_init_parameters: ZOOMSDK_tagInitParam,

    // Managed impl
    meeting_service: Option<MeetingService<'a>>,
    auth_service: Option<AuthService<'a>>,
    setting_service: Option<SettingService<'a>>,

    // Not managed impl
    ptr_network_conn_helper: *mut ZOOMSDK_INetworkConnectionHelper,

    // Dont use it
    new_domain: Option<Pin<CString>>,
}

/// Initialize ZOOM SDK  
/// - [SdkInitParam] Initialize the parameter of ZOOM SDK.  
/// - If the function succeeds, the return value is Ok([`Instance`]), otherwise failed, see [SdkError] for details.  
/// ```rust
/// use std::ffi::CString;
/// use zoom_sdk_linux_rs::{Instance, SdkInitParam, SdkLanguageId, init_sdk};
///
/// let mut instance = init_sdk(SdkInitParam {
///     str_web_domain: CString::new("https://zoom.us/").unwrap(),
///     str_support_url: CString::new("https://zoom.us/").unwrap(),
///     em_language_id: SdkLanguageId::French,
///     enable_generate_dump: true,
///     enable_log_by_default: true,
///     ui_log_file_size: 4,
///     ..Default::default()
/// });
/// ```
pub fn init_sdk<'a, 'b>(init_param: SdkInitParam) -> SdkResult<Pin<Box<Instance<'a>>>> {
    let mut out = Box::pin(Instance {
        raw_init_parameters: ZOOMSDK_tagInitParam {
            strWebDomain: init_param.str_web_domain.into_raw(),
            strBrandingName: init_param.str_branding_name.into_raw(),
            strSupportUrl: init_param.str_support_url.into_raw(),
            emLanguageID: init_param.em_language_id as u32,
            enableGenerateDump: init_param.enable_generate_dump,
            enableLogByDefault: init_param.enable_log_by_default,
            uiLogFileSize: init_param.ui_log_file_size,
            rawdataOpts: init_param.rawdata_opts.into(),
            obConfigOpts: ZOOMSDK_tagConfigurableOptions {
                sdkPathPostfix: std::ptr::null(),
            },
            wrapperType: init_param.wrapper_type as i32,
        },
        meeting_service: None,
        auth_service: None,
        setting_service: None,
        ptr_network_conn_helper: std::ptr::null_mut(),
        new_domain: None,
    });
    let ref_init = &mut out.raw_init_parameters;
    ZoomSdkResult(unsafe { ZOOMSDK_InitSDK(ref_init) }, out).into()
}

impl<'a> Instance<'a> {
    /// Get meeting service interface
    pub fn meeting(&mut self) -> &mut MeetingService<'a> {
        if self.meeting_service.is_none() {
            self.meeting_service = MeetingService::new()
                .map_err(|err| {
                    panic!("Unexpected Error : {:?}", err);
                })
                .ok();
        }
        self.meeting_service.as_mut().unwrap()
    }
    /// Get meeting service interface
    pub fn auth(&mut self) -> &mut AuthService<'a> {
        if self.auth_service.is_none() {
            self.auth_service = AuthService::new()
                .map_err(|err| {
                    panic!("Unexpected Error : {:?}", err);
                })
                .ok();
        }
        self.auth_service.as_mut().unwrap()
    }
    /// Get meeting service interface
    pub fn setting(&mut self) -> &mut SettingService<'a> {
        if self.setting_service.is_none() {
            self.setting_service = SettingService::new()
                .map_err(|err| {
                    panic!("Unexpected Error : {:?}", err);
                })
                .ok();
        }
        self.setting_service.as_mut().unwrap()
    }
    /// Destroy the meeting service Interface
    pub fn clear_meeting(&mut self) {
        let _trash = self.meeting_service.take();
    }
    /// Destroy the auth service Interface
    pub fn clear_auth(&mut self) {
        let _trash = self.auth_service.take();
    }
    /// Destroy the setting service Interface
    pub fn clear_setting(&mut self) {
        let _trash = self.setting_service.take();
    }
    /// Create network connection helper interface  
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [SdkError] for details.  
    /// - [ZoomRsError::NullPtr] if network connection helper interface was already created.  
    pub fn create_network_connection_helper(&mut self) -> SdkResult<()> {
        if !self.ptr_network_conn_helper.is_null() {
            tracing::warn!("Network connection helper interface was already created!");
            return Err(ZoomRsError::NullPtr);
        }
        let ref_service = &mut self.ptr_network_conn_helper;
        ZoomSdkResult(
            unsafe { ZOOMSDK_CreateNetworkConnectionHelper(ref_service) },
            (),
        )
        .into()
    }
    /// Destroy the specified network connection helper interface   
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [SdkError] for details.  
    /// - [ZoomRsError::NullPtr] if network connection helper interface was never created.  
    pub fn destroy_network_connection_helper(&mut self) -> SdkResult<()> {
        if self.ptr_network_conn_helper.is_null() {
            tracing::warn!("Network connection helper interface was never created!");
            return Err(ZoomRsError::NullPtr);
        }
        let result =
            unsafe { ZOOMSDK_DestroyNetworkConnectionHelper(self.ptr_network_conn_helper) };
        let output: SdkResult<()> = ZoomSdkResult(result, ()).into();
        if output.is_ok() {
            self.ptr_network_conn_helper = std::ptr::null_mut();
        }
        output
    }
    /// Get ZOOM last error interface.
    /// - If the function succeeds, the return value is an interface of ZOOM last error.
    /// - If the function fails or there is no error, the return value is NULL.
    /// TODO : Implements the corresponding cpp boilerplate.
    pub fn get_zoom_last_error(&self) -> SdkResult<()> {
        let ptr: *const ZOOMSDK_IZoomLastError = unsafe { ZOOMSDK_GetZoomLastError() };
        if ptr.is_null() {
            Ok(())
        } else {
            unimplemented!()
        }
    }
    /// Call the method to switch sdk domain
    /// - If the function succeeds, the return value is Ok(()), otherwise failed, see [SdkError] for details.
    /// TODO : Check function behavior.
    pub fn switch_domain(&mut self, new_domain: Pin<CString>, force: bool) -> SdkResult<()> {
        let res: SdkResult<()> = ZoomSdkResult(
            unsafe { ZOOMSDK_SwitchDomain(new_domain.as_ptr(), force) },
            (),
        )
        .into();
        if res.is_ok() {
            self.new_domain = Some(new_domain);
        }
        res
    }
}

/// Clean up ZOOM SDK by dropping the instance.
/// Teardown (destroy services, then CleanUPSDK) is performed by [`Instance::drop`].
/// This function must not call ZOOMSDK_CleanUPSDK() itself, or it would run twice when the passed instance is dropped.
pub fn cleanup_sdk(_this: Pin<Box<Instance>>) -> SdkResult<()> {
    // Instance::drop will destroy services and call CleanUPSDK once. Just dropping is sufficient.
    Ok(())
}

/// Drop boilerplate for Instance.
/// Teardown order must match Zoom samples to avoid SIGSEGV: destroy services explicitly, then CleanUPSDK().
/// See: meetingsdk-headless-linux-sample (Zoom::clean), meetingsdk-linux-raw-recording-sample (CleanSDK).
impl<'a> Drop for Instance<'a> {
    fn drop(&mut self) {
        tracing::info!("Zoom SDK instance teardown starting (sdk_tearing_down={})", is_sdk_tearing_down());

        if is_sdk_tearing_down() {
            // SDK already ran its own internal teardown after MeetingStatusDisconnecting.
            // Calling DestroyMeetingService / CleanUPSDK again would double-free.
            // Just drop Rust-side state without calling into the SDK.
            tracing::warn!("Skipping SDK service destruction and CleanUPSDK (SDK already tore down)");
            self.meeting_service = None;
            self.setting_service = None;
            self.auth_service = None;
            self.ptr_network_conn_helper = std::ptr::null_mut();
            return;
        }

        // Teardown order matches Zoom demos: destroy services first, then CleanUPSDK() once.

        // 1. Destroy meeting service first (releases meeting/recording state). Drop runs MeetingService::drop -> DestroyMeetingService.
        let _ = self.meeting_service.take();

        // 2. Destroy setting service (SettingService::drop -> DestroySettingService)
        let _ = self.setting_service.take();

        // 3. Destroy auth service (AuthService::drop -> DestroyAuthService)
        let _ = self.auth_service.take();

        // 4. Destroy network connection helper if it was created (main.rs may have already destroyed it)
        if !self.ptr_network_conn_helper.is_null() {
            let result = unsafe { ZOOMSDK_DestroyNetworkConnectionHelper(self.ptr_network_conn_helper) };
            if result != ZOOMSDK_SDKError_SDKERR_SUCCESS {
                tracing::warn!("DestroyNetworkConnectionHelper returned {:?}", result);
            }
            self.ptr_network_conn_helper = std::ptr::null_mut();
        }

        // 5. Required by Zoom SDK: global cleanup after all services are destroyed (avoids post-exit SIGSEGV)
        let err = unsafe { ZOOMSDK_CleanUPSDK() };
        if err != ZOOMSDK_SDKError_SDKERR_SUCCESS {
            tracing::warn!("ZOOMSDK_CleanUPSDK returned {:?}", err);
        } else {
            tracing::info!("ZOOMSDK_CleanUPSDK succeeded");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_version() {
        assert_eq!(&get_sdk_version(), "6.2.5 (2487)");
    }
}
