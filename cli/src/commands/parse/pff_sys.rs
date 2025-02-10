use libc::{c_char, c_int, size_t};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibPffItem {
    _unused: c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibPffRecordEntry {
    _unused: c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibPffRecordSet {
    _unused: c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibPffFile {
    _unused: c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibcError {
    pub number_of_messages: c_int,
    pub messages: *const *const c_char,
}

#[derive(Clone)]
pub enum LibPffEntryType {
    MessageTransportHeaders = 0x007d,
    AttachmentFilenameLong = 0x3707,
}

#[derive(Clone)]
pub enum LibPffEntryValueFlags {
    MatchAnyValueType = 0x01,
}

#[link(name = "pff")]
extern "C" {
    pub fn libpff_file_open(
        file: *mut LibPffFile,
        filename: *const c_char,
        access_flags: c_int,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_entry_value_utf8_string_size(
        message: *mut LibPffItem,
        entry_type: u32,
        utf8_string_size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_entry_value_utf8_string(
        message: *mut LibPffItem,
        entry_type: u32,
        utf8_string: *mut c_char,
        utf8_string_size: size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_file_initialize(file: *mut *mut LibPffFile, error: *mut *mut LibcError) -> c_int;

    pub fn libpff_file_get_root_folder(
        file: *mut LibPffFile,
        root_folder: *mut *mut LibPffItem,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_utf8_name_size(
        folder: *mut LibPffItem,
        utf8_string_size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_utf8_name(
        folder: *mut LibPffItem,
        utf8_string: *mut c_char,
        utf8_string_size: size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_number_of_sub_folders(
        folder: *mut LibPffItem,
        number_of_sub_folders: *mut c_int,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_sub_folder(
        folder: *mut LibPffItem,
        sub_folder_index: c_int,
        sub_folder: *mut *mut LibPffItem,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_number_of_sub_messages(
        folder: *mut LibPffItem,
        number_of_sub_messages: *mut c_int,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_folder_get_sub_message(
        folder: *mut LibPffItem,
        sub_message_index: c_int,
        sub_message: *mut *mut LibPffItem,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_html_body_size(
        message: *mut LibPffItem,
        size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_html_body(
        message: *mut LibPffItem,
        message_body: *mut c_char,
        size: size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_plain_text_body_size(
        message: *mut LibPffItem,
        size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_plain_text_body(
        message: *mut LibPffItem,
        message_body: *mut c_char,
        size: size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_number_of_attachments(
        message: *mut LibPffItem,
        number_of_attachments: *mut c_int,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_message_get_attachment(
        message: *mut LibPffItem,
        attachment_index: c_int,
        attachment: *mut *mut LibPffItem,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_item_get_record_set_by_index(
        item: *mut LibPffItem,
        record_set_index: c_int,
        record_set: *mut *mut LibPffRecordSet,
        error: *mut *mut LibcError,
    );

    pub fn libpff_record_set_get_entry_by_type(
        record_set: *mut LibPffRecordSet,
        entry_type: u32,
        value_type: u32,
        record_entry: *mut *mut LibPffRecordEntry,
        flags: u8,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_record_entry_get_data_as_utf8_string_size(
        record_entry: *mut LibPffRecordEntry,
        utf8_string_size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_record_entry_get_data_as_utf8_string(
        record_entry: *mut LibPffRecordEntry,
        utf8_string: *mut c_char,
        utf8_string_size: size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_attachment_get_data_size(
        attachment: *mut LibPffItem,
        size: *mut size_t,
        error: *mut *mut LibcError,
    ) -> c_int;

    pub fn libpff_attachment_get_type(
        attachment: *mut LibPffItem,
        attachment_type: *mut c_int,
        error: *mut *mut LibcError,
    ) -> c_int;

}
