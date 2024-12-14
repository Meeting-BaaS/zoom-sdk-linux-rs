#ifndef _C_RECORDING_CONTROLLER_H_
#define _C_RECORDING_CONTROLLER_H_

#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_recording_interface.h"

/// \brief Send a request to enable the SDK to start local recording.
/// \return If the function succeeds, the return value is SDKErr_Success and the SDK will send the request.
/// Otherwise it fails and the request will not be sent. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_request_local_recording_privilege(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Send a request to ask the host to start cloud recording.
/// \return If the function succeeds, the return value is SDKErr_Success and the SDK sends the request.
/// Otherwise it fails and the request is not sent. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_request_start_cloud_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Start recording.
/// \param [out] startTimestamp The timestamps when start recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_start_recording(ZOOMSDK::IMeetingRecordingController *ctrl, time_t *startTimestamp);

/// \brief Stop recording.
/// \param [out] stopTimestamp The timestamps when stop recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_stop_recording(ZOOMSDK::IMeetingRecordingController *ctrl, time_t *stopTimestamp);

/// \brief Pause recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_pause_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Resume recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_resume_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Set meeting recording callback event handler.
/// \param pEvent A pointer to the IMeetingRecordingCtrlEvent that receives the recording event.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_set_event(ZOOMSDK::IMeetingRecordingController *ctrl, void *arc_ptr);

/// \brief Determine if the specified user is enabled to start raw recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" bool recording_can_start_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Start rawdata recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_start_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

/// \brief Stop rawdata recording.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
extern "C" ZOOMSDK::SDKError recording_stop_raw_recording(ZOOMSDK::IMeetingRecordingController *ctrl);

#endif