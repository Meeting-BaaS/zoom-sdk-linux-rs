#ifndef _C_MEETING_SHARE_INTERFACE_H_
#define _C_MEETING_SHARE_INTERFACE_H_

#include "../../zoom-meeting-sdk-linux/h/meeting_service_components/meeting_sharing_interface.h"

/// \brief Set meeting share controller callback event handler.
/// \param pEvent A pointer to the IMeetingShareCtrlEvent that receives sharing event.
/// \return If the function succeeds, the return value is SDKErr_Success.
/// Otherwise failed. To get extended error information, see \link SDKError \endlink enum.
ZOOMSDK::SDKError sharing_set_event(ZOOMSDK::IMeetingShareController* controller, void *arc_ptr);

#endif