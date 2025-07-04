use crate::parse::pff_sys::*;
use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use libc::{c_char, c_int, size_t};
use mailparse::{MailHeader, MailHeaderMap};
use std::collections::HashSet;
use std::path::Path;

const ROOT_FOLDER_NAME: &str = "root";

pub struct PstFile {
    pub inner: *mut LibPffFile,
}

impl PstFile {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("Could not find input file"));
        }

        let filename = std::ffi::CString::new(path.to_str().context("Could not get path as str")?)
            .context("Could not convert path to Cstring")?;

        let mut file: *mut LibPffFile = std::ptr::null_mut();
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assume file exists and is a valid pst
        unsafe { libpff_file_initialize(&mut file, &mut error) };
        handle_error("Error initializing pst file", error)?;

        // Assume file exists and is a valid pst
        unsafe { libpff_file_open(file, filename.as_ptr(), 1, &mut error) };
        handle_error("Error opening pst file", error)?;

        Ok(Self { inner: file })
    }

    pub fn get_root_folder(&self) -> Result<PstFolder> {
        let mut root_folder: *mut LibPffItem = std::ptr::null_mut();

        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a valid pst file
        unsafe { libpff_file_get_root_folder(self.inner, &mut root_folder, &mut error) };
        handle_error("Error getting root folder", error)?;

        Ok(PstFolder {
            inner: root_folder,
            parent_folder_path: FolderPath::default(),
        })
    }
}

#[derive(Clone)]
pub struct PstFolder {
    pub inner: *mut LibPffItem,
    pub parent_folder_path: FolderPath,
}

impl PstFolder {
    pub fn all_items_iter(self) -> Result<AllItemsIterator> {
        AllItemsIterator::new(self)
    }

    pub fn get_item_count(self) -> Result<usize> {
        Ok(self.all_items_iter()?.count())
    }

    pub fn sub_messages_iter(self) -> Result<SubMessagesIter> {
        SubMessagesIter::new(self)
    }

    pub fn sub_folders_iter(self) -> Result<SubFoldersIter> {
        SubFoldersIter::new(self)
    }

    pub fn get_number_of_sub_messages(&self) -> Result<usize> {
        let mut number_of_sub_messages: c_int = -1;
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a folder in a valid pst file
        unsafe {
            libpff_folder_get_number_of_sub_messages(
                self.inner,
                &mut number_of_sub_messages,
                &mut error,
            )
        };
        handle_error("Error getting number of sub messages", error)?;

        Ok(number_of_sub_messages as usize)
    }

    pub fn get_sub_message(&self, sub_message_index: usize) -> Result<PstMessage> {
        let mut sub_message: *mut LibPffItem = std::ptr::null_mut();
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a folder in a valid pst file and index is in range
        unsafe {
            libpff_folder_get_sub_message(
                self.inner,
                sub_message_index as i32,
                &mut sub_message,
                &mut error,
            )
        };
        handle_error("Error getting sub message", error)?;

        Ok(PstMessage {
            inner: sub_message,
            folder: self.get_folder_path()?,
        })
    }

    pub fn get_number_of_sub_folders(&self) -> Result<usize> {
        let mut number_of_sub_folders: c_int = -1;
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a folder in a valid pst file
        unsafe {
            libpff_folder_get_number_of_sub_folders(
                self.inner,
                &mut number_of_sub_folders,
                &mut error,
            )
        };
        handle_error("Error getting number of sub folders", error)?;

        Ok(number_of_sub_folders as usize)
    }

    pub fn get_sub_folder(&self, folder_index: usize) -> Result<PstFolder> {
        let mut sub_folder: *mut LibPffItem = std::ptr::null_mut();
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a folder in a valid pst file and index is in range
        unsafe {
            libpff_folder_get_sub_folder(
                self.inner,
                folder_index as i32,
                &mut sub_folder,
                &mut error,
            )
        };
        handle_error("Error getting sub folder", error)?;

        Ok(PstFolder {
            inner: sub_folder,
            parent_folder_path: self.get_folder_path()?,
        })
    }

    fn get_name_size(&self) -> Result<usize> {
        let mut utf8_string_size: size_t = 0;
        let mut error: *mut LibcError = std::ptr::null_mut();
        // Assumes ptr is to a folder in a valid pst file
        unsafe { libpff_folder_get_utf8_name_size(self.inner, &mut utf8_string_size, &mut error) };
        handle_error("Error getting folder name size", error)?;

        Ok(utf8_string_size)
    }

    pub fn get_name(&self) -> Result<String> {
        if self.parent_folder_path.is_root() {
            return Ok(ROOT_FOLDER_NAME.to_string());
        }

        let size = self.get_name_size()?;

        let mut utf8_string: Vec<c_char> = vec![0; size + 1];
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a message within a valid pst file
        unsafe {
            libpff_folder_get_utf8_name(self.inner, utf8_string.as_mut_ptr(), size, &mut error);
        }
        handle_error("Error getting folder name", error)?;

        // Assumes ptr is a valid utf8 string
        let string = unsafe { std::ffi::CStr::from_ptr(utf8_string.as_ptr()) };
        Ok(string.to_string_lossy().to_string())
    }

    pub fn get_folder_path(&self) -> Result<FolderPath> {
        let name = self.get_name()?;
        Ok(self.parent_folder_path.sub_path(name))
    }
}

pub struct SubMessagesIter {
    idx: usize,
    total_number_of_messages: usize,
    folder: PstFolder,
}

impl SubMessagesIter {
    fn new(folder: PstFolder) -> Result<Self> {
        let total_number_of_messages = folder.get_number_of_sub_messages()?;

        Ok(Self {
            idx: 0,
            total_number_of_messages,
            folder,
        })
    }
}

impl Iterator for SubMessagesIter {
    type Item = Result<PstMessage>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.total_number_of_messages {
            let item = self.folder.get_sub_message(self.idx);
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct SubFoldersIter {
    idx: usize,
    total_number_of_folders: usize,
    folder: PstFolder,
}

impl SubFoldersIter {
    fn new(folder: PstFolder) -> Result<Self> {
        let total_number_of_folders = folder.get_number_of_sub_folders()?;

        Ok(Self {
            idx: 0,
            total_number_of_folders,
            folder,
        })
    }
}

impl Iterator for SubFoldersIter {
    type Item = Result<PstFolder>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.total_number_of_folders {
            let item = self.folder.get_sub_folder(self.idx);
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct AttachmentsIter {
    idx: usize,
    total_num_attachments: usize,
    message: PstMessage,
}

impl Iterator for AttachmentsIter {
    type Item = Result<PstAttachment>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.total_num_attachments {
            let next_attachment = self.message.get_attachment(self.idx);
            self.idx += 1;
            Some(next_attachment)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct PstMessage {
    pub inner: *mut LibPffItem,
    pub folder: FolderPath,
}

impl PstMessage {
    pub fn expect_header(headers: &[MailHeader], header: &str) -> Result<String> {
        let value = headers
            .get_first_value(header)
            .map(|value| value.trim().to_owned())
            .context(format!("{header} header is missing"))?;

        if value.is_empty() {
            Err(anyhow!("{} header is empty", header))
        } else {
            Ok(value)
        }
    }

    pub fn attachments_iter(&self) -> Result<AttachmentsIter> {
        Ok(AttachmentsIter {
            idx: 0,
            // Sometimes when there are no attachments we fail to count them
            total_num_attachments: self.get_number_of_attachments().unwrap_or(0),
            message: self.clone(),
        })
    }

    fn get_number_of_attachments(&self) -> Result<usize> {
        let mut number_of_attachments: c_int = -1;
        let mut error: *mut LibcError = std::ptr::null_mut();

        // Assumes ptr is to a message in a valid pst file
        unsafe {
            libpff_message_get_number_of_attachments(
                self.inner,
                &mut number_of_attachments,
                &mut error,
            )
        };
        handle_error("Error getting number of attachments", error)?;

        Ok(number_of_attachments as usize)
    }

    fn get_attachment(&self, index: usize) -> Result<PstAttachment> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut attachment: *mut LibPffItem = std::ptr::null_mut();

        // Assumes ptr is to a message in a valid pst file
        unsafe {
            libpff_message_get_attachment(self.inner, index as i32, &mut attachment, &mut error)
        };
        handle_error("Error getting attachment", error)?;

        PstAttachment::new(attachment)
    }

    fn get_entry_size(&self, entry_type: LibPffEntryType) -> Result<Option<usize>> {
        let mut utf8_string_size: size_t = 0;
        let mut error: *mut LibcError = std::ptr::null_mut();
        // Assumes ptr is to a message in a valid pst file
        let result = unsafe {
            libpff_message_get_entry_value_utf8_string_size(
                self.inner,
                entry_type as u32,
                &mut utf8_string_size,
                &mut error,
            )
        };
        handle_error("Error getting message record entry size", error)?;

        Ok(if result == 0 {
            return Ok(None);
        } else {
            Some(utf8_string_size)
        })
    }

    fn get_entry_string(&self, entry_type: LibPffEntryType) -> Result<Option<String>> {
        let size = self.get_entry_size(entry_type.clone())?;

        if size.is_none() {
            return Ok(None);
        }

        let size = size.context("Could not get size")?;

        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut utf8_string: Vec<c_char> = vec![0; size + 1];

        // Assumes ptr is to a message in a valid pst file
        let result = unsafe {
            libpff_message_get_entry_value_utf8_string(
                self.inner,
                entry_type as u32,
                utf8_string.as_mut_ptr(),
                size,
                &mut error,
            )
        };

        handle_error("Error getting message entry string", error)?;

        Ok(if result == 0 {
            return Ok(None);
        } else {
            // Assumes ptr is to a valid utf8 string
            let string = unsafe { std::ffi::CStr::from_ptr(utf8_string.as_ptr()) };
            Some(string.to_string_lossy().to_string())
        })
    }

    pub fn get_transport_headers(&self) -> Result<Option<String>> {
        self.get_entry_string(LibPffEntryType::MessageTransportHeaders)
    }

    fn get_plain_text_body_size(&self) -> Result<Option<usize>> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut size: size_t = 0;
        let result =
            // Assumes ptr is to a message in a valid pst file
            unsafe { libpff_message_get_plain_text_body_size(self.inner, &mut size, &mut error) };

        handle_error("Error getting plain text body size", error)?;

        Ok(if result == 0 {
            return Ok(None);
        } else {
            Some(size)
        })
    }

    pub fn get_plain_text_body(&self) -> Result<Option<String>> {
        let size = self.get_plain_text_body_size()?;

        if size.is_none() {
            return Ok(None);
        }

        let size = size.context("Could not get html body size")?;

        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut utf8_string: Vec<c_char> = vec![0; size + 1];

        let result = unsafe {
            // Assumes ptr is to a message in a valid pst file and size is correct
            libpff_message_get_plain_text_body(
                self.inner,
                utf8_string.as_mut_ptr(),
                size,
                &mut error,
            )
        };
        handle_error("Error getting plain text body", error)?;

        Ok(if result == 0 {
            None
        } else {
            // Assumes ptr is to a valid ut8 string
            let string = unsafe { std::ffi::CStr::from_ptr(utf8_string.as_ptr()) };
            Some(string.to_string_lossy().to_string())
        })
    }

    fn get_html_body_size(&self) -> Result<Option<usize>> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut size: size_t = 0;
        let result =
            // Assumes that ptr is to a message in a valid pst file
            unsafe { libpff_message_get_html_body_size(self.inner, &mut size, &mut error) };

        handle_error("Error getting html body size", error)?;

        Ok(if result == 0 {
            return Ok(None);
        } else {
            Some(size)
        })
    }

    pub fn get_html_body(&self) -> Result<Option<String>> {
        let size = self.get_html_body_size()?;

        if size.is_none() {
            return Ok(None);
        }

        let size = size.context("Could not get html body size")?;

        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut utf8_string: Vec<c_char> = vec![0; size + 1];

        // Assumes that ptr is to a message in a valid pst file and size is correct
        let result = unsafe {
            libpff_message_get_html_body(self.inner, utf8_string.as_mut_ptr(), size, &mut error)
        };
        handle_error("Error getting html body", error)?;

        Ok(if result == 0 {
            None
        } else {
            // Assumes ptr is to a valid ut8 string
            let string = unsafe { std::ffi::CStr::from_ptr(utf8_string.as_ptr()) };
            Some(string.to_string_lossy().to_string())
        })
    }
}

fn handle_error(friendly_message: &str, error: *mut LibcError) -> Result<()> {
    if !error.is_null() {
        // Assumes ptr to error is aligned
        let error_ref = unsafe { &*error };
        Err(anyhow!(format!(
            "{friendly_message}:\n{}",
            error_ref.as_string()
        )))
    } else {
        Ok(())
    }
}
#[derive(Clone, Default)]
pub struct FolderPath(pub Vec<String>);

impl FolderPath {
    fn sub_path(&self, name: String) -> Self {
        let mut path = self.0.clone();
        path.push(name);
        Self(path)
    }

    fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct AllItemsIterator {
    sub_messages_iter: SubMessagesIter,
    sub_folders_iter: SubFoldersIter,
    sub_all_items_iter: Option<Box<AllItemsIterator>>,
}

impl Iterator for AllItemsIterator {
    type Item = Result<PstMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(message) = self.sub_messages_iter.next() {
            return Some(message);
        }

        if let Some(sub_all_items_iter) = &mut self.sub_all_items_iter {
            if let Some(message) = sub_all_items_iter.next() {
                return Some(message);
            }
        }

        for folder in &mut self.sub_folders_iter {
            match folder {
                Ok(folder) => match folder.all_items_iter() {
                    Ok(mut iter) => {
                        let next_item = iter.next();
                        if next_item.is_some() {
                            self.sub_all_items_iter = Some(Box::new(iter));
                            return next_item;
                        }
                    }
                    Err(e) => return Some(Err(e)),
                },
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

impl AllItemsIterator {
    fn new(folder: PstFolder) -> Result<Self> {
        Ok(Self {
            sub_folders_iter: folder.clone().sub_folders_iter()?,
            sub_messages_iter: folder.sub_messages_iter()?,
            sub_all_items_iter: None,
        })
    }
}

pub struct PstAttachment {
    pub inner: *mut LibPffItem,
    pub attachment_type: LibPffAttachmentType,
}

impl PstAttachment {
    pub fn new(inner: *mut LibPffItem) -> Result<Self> {
        let attachment_type = Self::get_attachment_type(inner)?;
        Ok(Self {
            inner,
            attachment_type,
        })
    }

    fn get_attachment_type(inner: *mut LibPffItem) -> Result<LibPffAttachmentType> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut attachment_type: c_int = 0;

        // Assumes ptr is to an attachment in a valid pst file
        unsafe { libpff_attachment_get_type(inner, &mut attachment_type, &mut error) };
        handle_error("Error getting attachment type", error)?;

        LibPffAttachmentType::from_isize(attachment_type as isize)
    }

    pub fn get_content_type(&self) -> Result<String> {
        let name = self.get_name()?;
        let guess = mime_guess::from_path(name).first_or_octet_stream();
        Ok(guess.to_string())
    }

    pub fn get_size(&self) -> Result<usize> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut size: size_t = 0;

        // Assumes ptr is to an attachment in a valid pst file
        unsafe { libpff_attachment_get_data_size(self.inner, &mut size, &mut error) };
        handle_error("Error getting attachment data size", error)?;

        Ok(size)
    }

    fn get_record_set(&self) -> Result<*mut LibPffRecordSet> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut record_set: *mut LibPffRecordSet = std::ptr::null_mut();

        // Assumes ptr is to an attachment in a valid pst file
        unsafe {
            libpff_item_get_record_set_by_index(self.inner, 0, &mut record_set, &mut error);
        };
        handle_error("Error getting record set", error)?;
        Ok(record_set)
    }

    fn get_name_record_entry(
        &self,
        record_set: *mut LibPffRecordSet,
    ) -> Result<*mut LibPffRecordEntry> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut record_entry: *mut LibPffRecordEntry = std::ptr::null_mut();

        // Assumes ptr to record set is valid
        unsafe {
            libpff_record_set_get_entry_by_type(
                record_set,
                LibPffEntryType::AttachmentFilenameLong as u32,
                0,
                &mut record_entry,
                LibPffEntryValueFlags::MatchAnyValueType as u8,
                &mut error,
            );
        };

        handle_error("Error getting attachment name record entry", error)?;
        Ok(record_entry)
    }

    fn get_record_entry_size(&self, record_entry: *mut LibPffRecordEntry) -> Result<size_t> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let mut size: size_t = 0;

        // Assumes ptr to record entry is valid
        unsafe {
            libpff_record_entry_get_data_as_utf8_string_size(record_entry, &mut size, &mut error)
        };

        handle_error("Error getting attachment record entry size", error)?;
        Ok(size)
    }

    pub fn get_name(&self) -> Result<String> {
        let mut error: *mut LibcError = std::ptr::null_mut();
        let record_set = self.get_record_set()?;
        let record_entry = self.get_name_record_entry(record_set)?;
        let size = self.get_record_entry_size(record_entry)?;

        let mut utf8_string: Vec<c_char> = vec![0; size + 1];

        // Assumes record entry and size are valid
        unsafe {
            libpff_record_entry_get_data_as_utf8_string(
                record_entry,
                utf8_string.as_mut_ptr(),
                size,
                &mut error,
            )
        };
        handle_error("Error getting attachment name", error)?;

        // Assumes ptr is to a valid utf8 string
        let string = unsafe { std::ffi::CStr::from_ptr(utf8_string.as_ptr()) };

        Ok(string.to_string_lossy().to_string())
    }
}

#[derive(PartialEq)]
pub enum LibPffAttachmentType {
    Undefined = 0,
    Data = 'd' as isize,
    Item = 'i' as isize,
    Reference = 'r' as isize,
}

impl LibPffAttachmentType {
    fn from_isize(value: isize) -> Result<Self> {
        match value {
            0 => Ok(LibPffAttachmentType::Undefined),
            x if x == 'd' as isize => Ok(LibPffAttachmentType::Data),
            x if x == 'i' as isize => Ok(LibPffAttachmentType::Item),
            x if x == 'r' as isize => Ok(LibPffAttachmentType::Reference),
            _ => Err(anyhow!("Unknown Attachment type {}", value)),
        }
    }
}

impl LibcError {
    pub fn as_string(&self) -> String {
        let mut errors: Vec<String> = Vec::new();
        for i in 0..self.number_of_messages {
            // Assumes offset if valid
            let message_ptr = unsafe { self.messages.offset(i as isize) };

            if message_ptr.is_null() {
                errors.push("Unknown Error: message_ptr null".to_string());
                continue;
            }

            if !message_ptr.is_aligned() {
                errors.push("Unknown Error: message_ptr misaligned".to_string());
                continue;
            }

            // Assumes ptr is to a valid string
            let message = unsafe { std::ffi::CStr::from_ptr(*message_ptr) }
                .to_string_lossy()
                .into_owned();
            errors.push(message);
        }

        // Return errors deduplicated
        errors
            .into_iter()
            .collect::<HashSet<String>>()
            .iter()
            .join("\n")
    }
}
