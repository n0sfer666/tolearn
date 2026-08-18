use std::ffi::{c_char, c_float, c_int, c_void};

unsafe extern "C" {
    pub fn tolearn_speech_hush();
    pub fn tolearn_speech_open(model: *const c_char, gpu: c_int) -> *mut c_void;
    pub fn tolearn_speech_close(self_: *mut c_void);
    pub fn tolearn_speech_hear(
        self_: *mut c_void,
        pcm: *const c_float,
        samples: c_int,
        language: *const c_char,
        threads: c_int,
        heard: *mut *mut c_char,
    ) -> c_int;
    pub fn tolearn_speech_forget(heard: *mut c_char);
}
