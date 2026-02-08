use std::cell::Cell;
use std::ffi::CStr;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::{bindings::*, SdkResult, ZoomRsError, ZoomSdkResult};

/// This trait handles events related to participants.
pub trait ParticipantsEvent: std::fmt::Debug + Send {
    /// Callback event when users join the meeting.
    /// - [Vec<u32>] List of user IDs that joined.
    fn on_user_join(&mut self, _user_ids: Vec<u32>) {}

    /// Callback event when users leave the meeting.
    /// - [Vec<u32>] List of user IDs that left.
    fn on_user_left(&mut self, _user_ids: Vec<u32>) {}

    /// Callback event when the host changes.
    /// - [u32] The new host's user ID.
    fn on_host_change(&mut self, _new_host_id: u32) {}
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_user_join(ptr: *const u8, user_ids: *const u32, count: u32) {
    tracing::info!("Entering on_user_join with {} users", count);
    let user_ids_vec: Vec<u32> = (0..count as usize)
        .map(|i| unsafe { *user_ids.add(i) })
        .collect();
    (*convert_participants(ptr).lock().unwrap()).on_user_join(user_ids_vec);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_user_left(ptr: *const u8, user_ids: *const u32, count: u32) {
    tracing::info!("Entering on_user_left with {} users", count);
    let user_ids_vec: Vec<u32> = (0..count as usize)
        .map(|i| unsafe { *user_ids.add(i) })
        .collect();
    (*convert_participants(ptr).lock().unwrap()).on_user_left(user_ids_vec);
}

#[tracing::instrument(ret)]
#[no_mangle]
extern "C" fn on_host_change(ptr: *const u8, new_host_id: u32) {
    tracing::info!("Entering on_host_change with new_host_id={}", new_host_id);
    (*convert_participants(ptr).lock().unwrap()).on_host_change(new_host_id);
}

#[inline]
fn convert_participants(ptr: *const u8) -> Arc<Mutex<Box<dyn ParticipantsEvent>>> {
    let ptr: *const Mutex<Box<dyn ParticipantsEvent>> = ptr as *const _;
    unsafe { Arc::increment_strong_count(ptr) }; // Avoid freeing Arc after Drop
    unsafe { Arc::from_raw(ptr) }
}

/// Main interface to get info and to manipulate [Participant].
pub struct ParticipantsInterface<'a> {
    ref_participants_controler: &'a mut ZOOMSDK_IMeetingParticipantsController,
    evt_mutex: Option<Arc<Mutex<Box<dyn ParticipantsEvent>>>>,
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
                evt_mutex: None,
            })
        }
    }
    /// Get user id of the bot
    pub fn get_my_self_user_id(&mut self) -> i32 {
        let this = unsafe { get_my_self_user(self.ref_participants_controler) };
        unsafe { get_user_id(this) as i32 }
    }

    /// Check if participants can request local recording
    pub fn is_participant_request_local_recording_allowed(&mut self) -> bool {
        unsafe { is_participant_request_local_recording_allowed(self.ref_participants_controler) }
    }

    /// Set the participants controller callback event handler.
    /// - [ParticipantsEvent] A pointer to receive participant events (onUserJoin, onUserLeft, onHostChange).
    /// - If the function succeeds, the return value is Ok(), otherwise failed, see [crate::SdkError] for details.
    pub fn set_event(&mut self, ctx: Box<dyn ParticipantsEvent>) -> SdkResult<()> {
        self.evt_mutex = Some(Arc::new(Mutex::new(ctx)));
        let ptr = Arc::as_ptr(&self.evt_mutex.as_ref().unwrap()) as *mut _;
        tracing::info!("Setting participants event handler: {:?}", ptr);
        ZoomSdkResult(
            unsafe { participants_set_event(self.ref_participants_controler, ptr) },
            (),
        )
        .into()
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
