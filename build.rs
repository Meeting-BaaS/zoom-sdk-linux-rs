use std::path::PathBuf;

const MEETINGSDK_LIBNAME: &'static str = "meetingsdk";
const MEETINGSDK_PATH: &'static str = "zoom-sdk-linux-rs/zoom-meeting-sdk-linux";
// const MEETINGSDK_PATH: &'static str = "zoom-meeting-sdk-linux"; // STANDALONE LIB

fn main() {
    // Link zoom sdk library
    println!("cargo:rustc-link-search=native={}", MEETINGSDK_PATH);
    println!("cargo:rustc-link-lib=dylib={}", MEETINGSDK_LIBNAME);
    println!(
        "cargo:warning=cargo:rustc-link-lib=dylib={}",
        MEETINGSDK_LIBNAME
    );
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", MEETINGSDK_PATH); // Hack for linking dyn lib.so

    let cpp_files = [
        "wrapper-cpp/c_auth_service_interface.cpp",
        "wrapper-cpp/c_meeting_service_interface.cpp",
        "wrapper-cpp/c_network_connection_handler_interface.cpp",
        "wrapper-cpp/c_setting_service_interface.cpp",
        "wrapper-cpp/modules/c_meeting_chat_interface.cpp",
        "wrapper-cpp/modules/c_meeting_share_interface.cpp",
        "wrapper-cpp/modules/c_meeting_participants_interface.cpp",
        "wrapper-cpp/modules/c_meeting_audio_interface.cpp",
        "wrapper-cpp/modules/c_rawdata_video_source.cpp",
        "wrapper-cpp/modules/c_rawdata_audio_helper.cpp",
        "wrapper-cpp/modules/c_rawdata_video_helper.cpp",
        "wrapper-cpp/modules/c_recording_controller.cpp",
    ];
    let cpp_headers = [
        "wrapper-cpp/c_auth_service_interface.h",
        "wrapper-cpp/c_meeting_service_interface.h",
        "wrapper-cpp/c_setting_service_interface.h",
        "wrapper-cpp/c_network_connection_handler_interface.h",
        "wrapper-cpp/modules/c_meeting_chat_interface.h",
        "wrapper-cpp/modules/c_meeting_share_interface.h",
        "wrapper-cpp/modules/c_meeting_audio_interface.h",
        "wrapper-cpp/modules/c_rawdata_video_source.h",
        "wrapper-cpp/modules/c_rawdata_audio_helper.h",
        "wrapper-cpp/modules/c_rawdata_video_helper.h",
        "wrapper-cpp/modules/c_meeting_participants_interface.h",
        "wrapper-cpp/modules/c_recording_controller.h",
        "zoom-meeting-sdk-linux/h/zoom_sdk.h",
    ];

    cpp_files.iter().chain(cpp_headers.iter()).for_each(|file| {
        println!("cargo:rerun-if-changed={}", *file);
    });

    // Build own wrapper library
    cc::Build::new()
        .cpp(true)
        .opt_level(3)
        .flag("-march=native")
        .files(cpp_files)
        .include("zoom-meeting-sdk-linux/h/")
        .compile("wrapper");

    // Bindgen configuration for c++
    let clang_args = vec![
        "-xc++".to_string(),
        "-std=c++17".to_string(),
        format!("-Izoom-meeting-sdk-linux/h/"),
    ];

    // Generate bindings and write file
    let bindings = bindgen::Builder::default()
        .clang_args(clang_args)
        .headers(cpp_headers)
        .generate()
        .expect("Cannot generate bindings");

    let out_path = PathBuf::from("src/bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Cannot write bindings.rs");
}
