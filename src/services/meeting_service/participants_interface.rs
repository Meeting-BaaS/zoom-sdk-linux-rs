use std::cell::Cell;
use std::ffi::CStr;
use std::fmt;

use crate::{bindings::*, SdkResult, ZoomRsError};

/// User role in a Zoom meeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum UserRole {
    /// No role assigned.
    None = 0,
    /// Meeting host.
    Host = 1,
    /// Co-host with elevated privileges.
    CoHost = 2,
    /// Panelist in a webinar.
    Panelist = 3,
    /// Moderator of a breakout room.
    BreakoutModerator = 4,
    /// Regular attendee.
    Attendee = 5,
}

impl UserRole {
    /// Convert from raw SDK value to enum.
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Host,
            2 => Self::CoHost,
            3 => Self::Panelist,
            4 => Self::BreakoutModerator,
            5 => Self::Attendee,
            _ => Self::None,
        }
    }

    /// Convert to a human-readable string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Host => "host",
            Self::CoHost => "cohost",
            Self::Panelist => "panelist",
            Self::BreakoutModerator => "breakout_moderator",
            Self::Attendee => "attendee",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// How a user joined audio in a Zoom meeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AudioJoinType {
    /// Unknown audio connection type.
    Unknown = 0,
    /// Voice over IP (computer audio).
    Voip = 1,
    /// Phone dial-in.
    Phone = 2,
    /// H.323 or SIP (type not yet determined).
    UnknownH323OrSip = 3,
    /// H.323 video conferencing protocol.
    H323 = 4,
    /// SIP (Session Initiation Protocol).
    Sip = 5,
}

impl AudioJoinType {
    /// Convert from raw SDK value to enum.
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Voip,
            2 => Self::Phone,
            3 => Self::UnknownH323OrSip,
            4 => Self::H323,
            5 => Self::Sip,
            _ => Self::Unknown,
        }
    }

    /// Convert to a human-readable string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Voip => "voip",
            Self::Phone => "phone",
            Self::UnknownH323OrSip => "unknown_h323_or_sip",
            Self::H323 => "h323",
            Self::Sip => "sip",
        }
    }
}

impl fmt::Display for AudioJoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// External C functions for participant info methods
extern "C" {
    /// Get the avatar file path for a user.
    pub fn meeting_participants_get_avatar_path(user_info: *mut ZOOMSDK_IUserInfo) -> *const zchar_t;
    /// Get the persistent ID for a user (unique across meetings).
    pub fn meeting_participants_get_persistent_id(user_info: *mut ZOOMSDK_IUserInfo) -> *const zchar_t;
    /// Get the customer key for a user (if assigned in join parameters).
    pub fn meeting_participants_get_customer_key(user_info: *mut ZOOMSDK_IUserInfo) -> *const zchar_t;
    /// Get the user role (0=NONE, 1=HOST, 2=COHOST, 3=PANELIST, 4=BREAKOUT_MODERATOR, 5=ATTENDEE).
    pub fn meeting_participants_get_user_role(user_info: *mut ZOOMSDK_IUserInfo) -> i32;
    /// Get audio join type (0=UNKNOWN, 1=VOIP, 2=PHONE, 3=UNKNOWN_H323_OR_SIP, 4=H323, 5=SIP).
    pub fn meeting_participants_get_audio_join_type(user_info: *mut ZOOMSDK_IUserInfo) -> i32;
    /// Check if user is a pure phone user (dialed in, no app).
    pub fn meeting_participants_is_pure_phone_user(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
    /// Check if user has a camera device.
    pub fn meeting_participants_has_camera(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
    /// Check if user's audio is muted.
    pub fn meeting_participants_is_audio_muted(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
    /// Check if user's video is on.
    pub fn meeting_participants_is_video_on(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
    /// Check if user is in waiting room.
    pub fn meeting_participants_is_in_waiting_room(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
    /// Check if user has hand raised.
    pub fn meeting_participants_is_raise_hand(user_info: *mut ZOOMSDK_IUserInfo) -> bool;
}

/// Main interface to get info and to manipulate [Participant].
pub struct ParticipantsInterface<'a> {
    ref_participants_controler: &'a mut ZOOMSDK_IMeetingParticipantsController,
}

impl<'a> fmt::Debug for ParticipantsInterface<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParticipantsInterface")
            .field(
                "ref_participants_controler",
                self.ref_participants_controler,
            )
            .finish()
    }
}

/// This struct represents a [Participant].
pub struct Participant<'a> {
    inner: &'a Cell<ZOOMSDK_IUserInfo>, // Interior mutability garanted by inner UnsafeCell
    user_id: i32,
}

impl<'a> fmt::Debug for Participant<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Participant").finish()
    }
}

impl<'a> ParticipantsInterface<'a> {
    /// Get the participants interface.
    /// - If the function succeeds, the return value is [ParticipantsInterface]. Otherwise returns None.
    pub fn new(meeting_service: &mut ZOOMSDK_IMeetingService) -> Option<Self> {
        let ptr = unsafe { meeting_get_meeting_participants_controller(meeting_service) };

        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ref_participants_controler: unsafe { ptr.as_mut() }.unwrap(),
            })
        }
    }
    /// Get user id of the bot
    pub fn get_my_self_user_id(&mut self) -> i32 {
        let this = unsafe { get_my_self_user(self.ref_participants_controler) };
        unsafe { get_user_id(this) as i32 }
    }

    /// Checj if participants can request local recording
    pub fn is_participant_request_local_recording_allowed(&mut self) -> bool {
        unsafe { is_participant_request_local_recording_allowed(self.ref_participants_controler) }
    }
}

impl<'a> Participant<'a> {
    /// Retrieve the user ID of the participant.
    #[inline(always)]
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    /// Check if a participant is the host
    /// - [bool], true is the user is the host.
    pub fn is_host(&self) -> bool {
        unsafe { is_host(self.inner.as_ptr()) }
    }

    /// Check if a participant is talking.
    /// - [bool], true is the user is talking.
    pub fn is_talking(&self) -> bool {
        unsafe { meeting_participants_is_talking(self.inner.as_ptr()) }
    }
    /// Get the username matched with the current user information.
    /// - [str] The return value is the username.
    /// - Valid for both normal user and webinar attendee.
    pub fn get_user_name(&self) -> SdkResult<&str> {
        Ok(unsafe {
            let ptr = meeting_participants_get_user_name(self.inner.as_ptr());
            if ptr.is_null() {
                return Err(ZoomRsError::NullPtr);
            }
            CStr::from_ptr(ptr)
        }
        .to_str()
        .unwrap())
    }
    /// Get the Mic level of the user corresponding to the current information.
    /// - [i32] The mic level of the user.
    pub fn get_audio_voice_level(&self) -> i32 {
        unsafe { meeting_participants_get_audio_voice_level(self.inner.as_ptr()) }
    }

    /// Get the avatar file path matched with the current user information.
    /// - [Option<&str>] The avatar file path, or None if not available.
    pub fn get_avatar_path(&self) -> Option<&str> {
        unsafe {
            let ptr = meeting_participants_get_avatar_path(self.inner.as_ptr());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    /// Get the user persistent id matched with the current user information.
    /// This is a unique identifier that persists across meetings.
    /// - [Option<&str>] The persistent ID, or None if not available.
    pub fn get_persistent_id(&self) -> Option<&str> {
        unsafe {
            let ptr = meeting_participants_get_persistent_id(self.inner.as_ptr());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    /// Get the customer_key matched with the current user information.
    /// This is a custom key assigned in the join meeting parameter.
    /// - [Option<&str>] The customer key, or None if not assigned.
    pub fn get_customer_key(&self) -> Option<&str> {
        unsafe {
            let ptr = meeting_participants_get_customer_key(self.inner.as_ptr());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    /// Get the type of role of the user.
    pub fn get_user_role(&self) -> UserRole {
        UserRole::from_raw(unsafe { meeting_participants_get_user_role(self.inner.as_ptr()) })
    }

    /// Get the audio join type of the user.
    pub fn get_audio_join_type(&self) -> AudioJoinType {
        AudioJoinType::from_raw(unsafe { meeting_participants_get_audio_join_type(self.inner.as_ptr()) })
    }

    /// Check if user is a pure phone user (dialed in by phone, no app).
    pub fn is_pure_phone_user(&self) -> bool {
        unsafe { meeting_participants_is_pure_phone_user(self.inner.as_ptr()) }
    }

    /// Check if user has a camera device.
    pub fn has_camera(&self) -> bool {
        unsafe { meeting_participants_has_camera(self.inner.as_ptr()) }
    }

    /// Check if user's audio is muted.
    pub fn is_audio_muted(&self) -> bool {
        unsafe { meeting_participants_is_audio_muted(self.inner.as_ptr()) }
    }

    /// Check if user's video is on.
    pub fn is_video_on(&self) -> bool {
        unsafe { meeting_participants_is_video_on(self.inner.as_ptr()) }
    }

    /// Check if user is in waiting room.
    pub fn is_in_waiting_room(&self) -> bool {
        unsafe { meeting_participants_is_in_waiting_room(self.inner.as_ptr()) }
    }

    /// Check if user has hand raised.
    pub fn is_raise_hand(&self) -> bool {
        unsafe { meeting_participants_is_raise_hand(self.inner.as_ptr()) }
    }
}

impl<'a> ParticipantsInterface<'a> {
    /// Create a new mutable [ParticipantsIterator] over participants.
    pub fn iter(&mut self) -> SdkResult<ParticipantsIterator<'a>> {
        let mut len: u32 = 0;
        let user_infos: *mut participant =
            unsafe { meeting_participants_get_users(self.ref_participants_controler, &mut len) }
                as _;
        if user_infos.is_null() {
            return Err(ZoomRsError::NullPtr);
        }
        // Sadly unsafe : We avoid to take ownership of the underlying *mut participant
        Ok(ParticipantsIterator {
            internal_userinfo_vec: Some(
                (0..len as usize)
                    .map(|idx| unsafe {
                        let v = *user_infos.offset(idx as isize);
                        InternalUserInfo {
                            cell: Cell::from_mut(v.user_info.as_mut().expect("All is bullshit")),
                            user_id: v.user_id,
                        }
                    })
                    .collect::<Vec<InternalUserInfo>>(),
            ),
            internal_userinfo_raw_pointer: user_infos,
            count: 0,
        })
    }
}

struct InternalUserInfo<'a> {
    cell: &'a Cell<ZOOMSDK_IUserInfo>,
    user_id: i32,
}

/// Public [Iterator] over [Participant].
pub struct ParticipantsIterator<'a> {
    internal_userinfo_vec: Option<Vec<InternalUserInfo<'a>>>,
    internal_userinfo_raw_pointer: *mut participant,
    count: usize,
}

impl<'a> Iterator for ParticipantsIterator<'a> {
    type Item = Participant<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.internal_userinfo_vec.as_mut().unwrap();
        if self.count < v.len() {
            let output = Participant {
                inner: v[self.count].cell,
                user_id: v[self.count].user_id,
            };
            self.count += 1;
            Some(output)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for ParticipantsIterator<'a> {
    fn len(&self) -> usize {
        self.internal_userinfo_vec.as_ref().unwrap().len() - self.count
    }
}

impl<'a> Drop for ParticipantsIterator<'a> {
    fn drop(&mut self) {
        self.internal_userinfo_vec = None;
        // Finaly, free the underlying pointer
        unsafe { meeting_participants_free_memory(self.internal_userinfo_raw_pointer) }
    }
}
