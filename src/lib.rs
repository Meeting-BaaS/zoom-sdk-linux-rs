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
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

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

/// Tracks whether the SDK ever reached MeetingStatusInMeeting. Used to distinguish
/// a real meeting disconnect (raw-data objects exist, need teardown protection) from
/// a pre-meeting disconnect (e.g. OBF "authorized user not in meeting" — no raw-data
/// objects, SDK can be reused for retry).
static MEETING_WAS_IN_MEETING: AtomicBool = AtomicBool::new(false);

/// Raw pointer to the GLib MainLoop. Stored so that `mark_sdk_teardown()` can quit
/// the main loop from the SDK callback thread. `g_main_loop_quit()` is thread-safe
/// and wakes up the poll, allowing the main thread to exit the event loop and proceed
/// to post-processing before the SDK's internal teardown can crash the process.
static GLIB_MAIN_LOOP_PTR: AtomicPtr<glib::ffi::GMainLoop> =
    AtomicPtr::new(std::ptr::null_mut());

/// Register the GLib MainLoop so `mark_sdk_teardown()` can quit it from the SDK thread.
/// Must be called from the main thread after creating the MainLoop.
pub fn set_main_loop(main_loop: &glib::MainLoop) {
    let ptr = glib::translate::ToGlibPtr::to_glib_none(main_loop).0;
    GLIB_MAIN_LOOP_PTR.store(ptr, Ordering::SeqCst);
}

/// Mark that the SDK has started teardown (called from `on_meeting_status_changed`
/// when `MeetingStatusDisconnecting` is received).
pub fn mark_sdk_teardown() {
    SDK_TEARDOWN_STARTED.store(true, Ordering::SeqCst);
    tracing::warn!("SDK teardown flag set — SDK is disconnecting, skipping further raw-data operations");

    // Quit the GLib main loop from the SDK thread. This is thread-safe and wakes up
    // the poll, ensuring the main thread exits main_loop.run() promptly. Without this,
    // the SDK's internal teardown can run on the main thread and prevent the GLib tick
    // handler from ever firing, leaving the bot stuck until the SDK crashes (SIGSEGV).
    let ptr = GLIB_MAIN_LOOP_PTR.load(Ordering::SeqCst);
    if !ptr.is_null() {
        unsafe { glib::ffi::g_main_loop_quit(ptr) };
        tracing::info!("GLib main loop quit requested from SDK teardown callback");
    }
}

/// Check whether the SDK teardown has started.
pub fn is_sdk_tearing_down() -> bool {
    SDK_TEARDOWN_STARTED.load(Ordering::SeqCst)
}

/// Mark that the meeting has reached InMeeting status.
/// Called from `on_meeting_status_changed` when `MeetingStatusInMeeting` is received.
pub fn mark_meeting_entered() {
    MEETING_WAS_IN_MEETING.store(true, Ordering::SeqCst);
}

/// Check whether the meeting was ever entered (reached InMeeting status).
/// Returns false for pre-meeting disconnects (e.g. OBF join failures).
pub fn was_meeting_entered() -> bool {
    MEETING_WAS_IN_MEETING.load(Ordering::SeqCst)
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
/// assert_eq!(&get_sdk_version(), "6.7.5 (7391)");
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

    // True after cleanup_sdk() has run — prevents Drop from double-cleaning.
    cleaned_up: bool,
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
        cleaned_up: false,
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
    /// Destroy all SDK services and call CleanUPSDK().
    ///
    /// Must be called as soon as the meeting ends, BEFORE any long-running
    /// post-processing (video thread join, FFmpeg, S3 uploads).  This matches
    /// the Attendee pattern: adapter.cleanup() runs right after meeting ends,
    /// then file uploads happen afterward.
    ///
    /// After this, Drop is a no-op (safe to let Instance go out of scope).
    pub fn cleanup_sdk(&mut self) {
        if self.cleaned_up {
            tracing::info!("cleanup_sdk: already cleaned up, skipping");
            return;
        }
        self.cleaned_up = true;
        tracing::info!("cleanup_sdk: destroying services and cleaning up SDK");

        // 1. Destroy meeting service (MeetingService::drop -> DestroyMeetingService)
        let _ = self.meeting_service.take();

        // 2. Destroy setting service (SettingService::drop -> DestroySettingService)
        let _ = self.setting_service.take();

        // 3. Destroy auth service (AuthService::drop -> DestroyAuthService)
        let _ = self.auth_service.take();

        // 4. Destroy network connection helper
        if !self.ptr_network_conn_helper.is_null() {
            let result = unsafe { ZOOMSDK_DestroyNetworkConnectionHelper(self.ptr_network_conn_helper) };
            if result != ZOOMSDK_SDKError_SDKERR_SUCCESS {
                tracing::warn!("DestroyNetworkConnectionHelper returned {:?}", result);
            }
            self.ptr_network_conn_helper = std::ptr::null_mut();
        }

        // 5. Mark teardown so any subsequent Drop impls (AudioRawDataHelper,
        //    Renderer) that run after this point skip SDK calls — the SDK
        //    is no longer in a valid state after CleanUPSDK().
        mark_sdk_teardown();

        // 6. CleanUPSDK — global SDK cleanup after all services are destroyed
        let err = unsafe { ZOOMSDK_CleanUPSDK() };
        if err != ZOOMSDK_SDKError_SDKERR_SUCCESS {
            tracing::warn!("ZOOMSDK_CleanUPSDK returned {:?}", err);
        } else {
            tracing::info!("ZOOMSDK_CleanUPSDK succeeded");
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
/// Teardown (destroy services, then CleanUPSDK) is performed by [`Instance::cleanup_sdk`].
pub fn cleanup_sdk(_this: Pin<Box<Instance>>) -> SdkResult<()> {
    // Instance::drop calls cleanup_sdk() which handles everything. Just dropping is sufficient.
    Ok(())
}

/// Drop delegates to cleanup_sdk(). If cleanup_sdk() was already called
/// explicitly (the normal path), this is a no-op.
impl<'a> Drop for Instance<'a> {
    fn drop(&mut self) {
        self.cleanup_sdk();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_version() {
        assert_eq!(&get_sdk_version(), "6.7.5 (7391)");
    }
}
