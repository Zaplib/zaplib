//! Bindings for OSX Core Foundation and Metal.

// Clippy TODO
#![allow(clippy::enum_variant_names)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::ptr::NonNull;

pub(crate) use zaplib_objc_sys::declare::ClassDecl;
pub(crate) use zaplib_objc_sys::runtime::{Class, Object, Protocol, Sel, BOOL, NO, YES};
pub(crate) use zaplib_objc_sys::{class, msg_send, sel, sel_impl};
pub(crate) use zaplib_objc_sys::{Encode, Encoding};
//use bitflags::bitflags;

pub(crate) type id = *mut zaplib_objc_sys::runtime::Object;
pub(crate) const nil: id = 0 as id;

pub struct RcObjcId(NonNull<Object>);

impl RcObjcId {
    pub fn from_owned(id: NonNull<Object>) -> Self {
        Self(id)
    }

    pub fn from_unowned(id: NonNull<Object>) -> Self {
        unsafe {
            let _: () = msg_send![id.as_ptr(), retain];
        }
        Self::from_owned(id)
    }

    pub fn as_id(&self) -> id {
        self.0.as_ptr()
    }

    pub fn forget(self) -> NonNull<Object> {
        unsafe {
            let _: () = msg_send![self.0.as_ptr(), retain];
        }
        self.0
    }
}

#[link(name = "system")]
extern "C" {
    pub static _NSConcreteStackBlock: [*const c_void; 32];
    pub static _NSConcreteBogusBlock: [*const c_void; 32];
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub(crate) static NSRunLoopCommonModes: id;
    pub(crate) static NSDefaultRunLoopMode: id;
    pub(crate) fn NSStringFromClass(cls: id) -> id;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    pub(crate) static NSStringPboardType: id;
    pub(crate) static NSFilenamesPboardType: id;
    pub(crate) static NSURLPboardType: id;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub(crate) fn CGMainDisplayID() -> u32;
    pub(crate) fn CGDisplayPixelsHigh(display: u32) -> u64;
    pub(crate) fn CGColorCreateSRGB(red: f64, green: f64, blue: f64, alpha: f64) -> id;
}

#[link(name = "Metal", kind = "framework")]
extern "C" {
    fn MTLCreateSystemDefaultDevice() -> id;
    #[cfg(not(target_os = "ios"))]
    fn MTLCopyAllDevices() -> id; //TODO: Array
}

pub(crate) fn get_default_metal_device() -> Option<id> {
    unsafe {
        let dev = MTLCreateSystemDefaultDevice();
        if dev == nil {
            None
        } else {
            Some(dev)
        }
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {}

pub(crate) fn get_all_metal_devices() -> Vec<id> {
    #[cfg(target_os = "ios")]
    {
        MTLCreateSystemDefaultDevice().into_iter().collect()
    }
    #[cfg(not(target_os = "ios"))]
    unsafe {
        let array = MTLCopyAllDevices();
        let count: u64 = msg_send![array, count];
        let ret = (0..count)
            .map(|i| msg_send![array, objectAtIndex: i])
            // The elements of this array are references---we convert them to owned references
            // (which just means that we increment the reference count here, and it is
            // decremented in the `Drop` impl for `Device`)
            .map(|device: *mut Object| msg_send![device, retain])
            .collect();
        let () = msg_send![array, release];
        ret
    }
}

pub(crate) fn nsstring_to_string(string: id) -> String {
    unsafe {
        let utf8_string: *const std::os::raw::c_uchar = msg_send![string, UTF8String];
        let utf8_len: usize = msg_send![string, lengthOfBytesUsingEncoding: UTF8_ENCODING];
        let slice = std::slice::from_raw_parts(utf8_string, utf8_len);
        std::str::from_utf8_unchecked(slice).to_owned()
    }
}

pub(crate) fn str_to_nsstring(val: &str) -> id {
    unsafe {
        let ns_string: id = msg_send![class!(NSString), alloc];
        let ns_string: id = msg_send![
            ns_string,
            initWithBytes: val.as_ptr()
            length: val.len()
            encoding: UTF8_ENCODING as id
        ];
        ns_string
    }
}

pub(crate) fn load_native_cursor(cursor_name: &str) -> id {
    let sel = Sel::register(cursor_name);
    let id: id = unsafe { msg_send![class!(NSCursor), performSelector: sel] };
    id
}

pub(crate) fn load_undocumented_cursor(cursor_name: &str) -> id {
    unsafe {
        let class = class!(NSCursor);
        let sel = Sel::register(cursor_name);
        let sel: id = msg_send![class, respondsToSelector: sel];
        let id: id = msg_send![class, performSelector: sel];
        id
    }
}

pub(crate) fn load_webkit_cursor(cursor_name_str: &str) -> id {
    unsafe {
        static CURSOR_ROOT: &str = "/System/Library/Frameworks/ApplicationServices.framework/Versions/A/Frameworks/HIServices.\
                                    framework/Versions/A/Resources/cursors";
        let cursor_root = str_to_nsstring(CURSOR_ROOT);
        let cursor_name = str_to_nsstring(cursor_name_str);
        let cursor_pdf = str_to_nsstring("cursor.pdf");
        let cursor_plist = str_to_nsstring("info.plist");
        let key_x = str_to_nsstring("hotx");
        let key_y = str_to_nsstring("hoty");

        let cursor_path: id = msg_send![cursor_root, stringByAppendingPathComponent: cursor_name];
        let pdf_path: id = msg_send![cursor_path, stringByAppendingPathComponent: cursor_pdf];
        let info_path: id = msg_send![cursor_path, stringByAppendingPathComponent: cursor_plist];

        let ns_image: id = msg_send![class!(NSImage), alloc];
        let () = msg_send![ns_image, initByReferencingFile: pdf_path];
        let info: id = msg_send![class!(NSDictionary), dictionaryWithContentsOfFile: info_path];
        //let image = NSImage::alloc(nil).initByReferencingFile_(pdf_path);
        // let info = NSDictionary::dictionaryWithContentsOfFile_(nil, info_path);

        let x: id = msg_send![info, valueForKey: key_x]; //info.valueForKey_(key_x);
        let y: id = msg_send![info, valueForKey: key_y]; //info.valueForKey_(key_y);
        let point = NSPoint { x: msg_send![x, doubleValue], y: msg_send![y, doubleValue] };
        let cursor: id = msg_send![class!(NSCursor), alloc];
        msg_send![cursor, initWithImage: ns_image hotSpot: point]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct NSPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

unsafe impl Encode for NSPoint {
    fn encode() -> Encoding {
        let encoding = format!("{{CGPoint={}{}}}", f64::encode().as_str(), f64::encode().as_str());
        unsafe { Encoding::from_str(&encoding) }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct NSSize {
    pub(crate) width: f64,
    pub(crate) height: f64,
}

unsafe impl Encode for NSSize {
    fn encode() -> Encoding {
        let encoding = format!("{{CGSize={}{}}}", f64::encode().as_str(), f64::encode().as_str());
        unsafe { Encoding::from_str(&encoding) }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct NSRect {
    pub(crate) origin: NSPoint,
    pub(crate) size: NSSize,
}

unsafe impl Encode for NSRect {
    fn encode() -> Encoding {
        let encoding = format!("{{CGRect={}{}}}", NSPoint::encode().as_str(), NSSize::encode().as_str());
        unsafe { Encoding::from_str(&encoding) }
    }
}

#[repr(u64)] // NSUInteger
pub(crate) enum NSEventModifierFlags {
    NSAlphaShiftKeyMask = 1 << 16,
    NSShiftKeyMask = 1 << 17,
    NSControlKeyMask = 1 << 18,
    NSAlternateKeyMask = 1 << 19,
    NSCommandKeyMask = 1 << 20,
    NSNumericPadKeyMask = 1 << 21,
    NSHelpKeyMask = 1 << 22,
    NSFunctionKeyMask = 1 << 23,
    NSDeviceIndependentModifierFlagsMask = 0xffff0000,
}

const UTF8_ENCODING: usize = 4;

#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NSWindowTitleVisibility {
    NSWindowTitleVisible = 0,
    NSWindowTitleHidden = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u64)] // NSUInteger
pub(crate) enum NSEventType {
    NSLeftMouseDown = 1,
    NSLeftMouseUp = 2,
    NSRightMouseDown = 3,
    NSRightMouseUp = 4,
    NSMouseMoved = 5,
    NSLeftMouseDragged = 6,
    NSRightMouseDragged = 7,
    NSMouseEntered = 8,
    NSMouseExited = 9,
    NSKeyDown = 10,
    NSKeyUp = 11,
    NSFlagsChanged = 12,
    NSAppKitDefined = 13,
    NSSystemDefined = 14,
    NSApplicationDefined = 15,
    NSPeriodic = 16,
    NSCursorUpdate = 17,
    NSScrollWheel = 22,
    NSTabletPoint = 23,
    NSTabletProximity = 24,
    NSOtherMouseDown = 25,
    NSOtherMouseUp = 26,
    NSOtherMouseDragged = 27,
    NSEventTypeGesture = 29,
    NSEventTypeMagnify = 30,
    NSEventTypeSwipe = 31,
    NSEventTypeRotate = 18,
    NSEventTypeBeginGesture = 19,
    NSEventTypeEndGesture = 20,
    NSEventTypePressure = 34,
}

#[repr(u64)] // NSUInteger
pub(crate) enum NSEventMask {
    NSLeftMouseDownMask = 1 << NSEventType::NSLeftMouseDown as u64,
    NSLeftMouseUpMask = 1 << NSEventType::NSLeftMouseUp as u64,
    NSRightMouseDownMask = 1 << NSEventType::NSRightMouseDown as u64,
    NSRightMouseUpMask = 1 << NSEventType::NSRightMouseUp as u64,
    NSMouseMovedMask = 1 << NSEventType::NSMouseMoved as u64,
    NSLeftMouseDraggedMask = 1 << NSEventType::NSLeftMouseDragged as u64,
    NSRightMouseDraggedMask = 1 << NSEventType::NSRightMouseDragged as u64,
    NSMouseEnteredMask = 1 << NSEventType::NSMouseEntered as u64,
    NSMouseExitedMask = 1 << NSEventType::NSMouseExited as u64,
    NSKeyDownMask = 1 << NSEventType::NSKeyDown as u64,
    NSKeyUpMask = 1 << NSEventType::NSKeyUp as u64,
    NSFlagsChangedMask = 1 << NSEventType::NSFlagsChanged as u64,
    NSAppKitDefinedMask = 1 << NSEventType::NSAppKitDefined as u64,
    NSSystemDefinedMask = 1 << NSEventType::NSSystemDefined as u64,
    NSApplicationDefinedMask = 1 << NSEventType::NSApplicationDefined as u64,
    NSPeriodicMask = 1 << NSEventType::NSPeriodic as u64,
    NSCursorUpdateMask = 1 << NSEventType::NSCursorUpdate as u64,
    NSScrollWheelMask = 1 << NSEventType::NSScrollWheel as u64,
    NSTabletPointMask = 1 << NSEventType::NSTabletPoint as u64,
    NSTabletProximityMask = 1 << NSEventType::NSTabletProximity as u64,
    NSOtherMouseDownMask = 1 << NSEventType::NSOtherMouseDown as u64,
    NSOtherMouseUpMask = 1 << NSEventType::NSOtherMouseUp as u64,
    NSOtherMouseDraggedMask = 1 << NSEventType::NSOtherMouseDragged as u64,
    NSEventMaskGesture = 1 << NSEventType::NSEventTypeGesture as u64,
    NSEventMaskSwipe = 1 << NSEventType::NSEventTypeSwipe as u64,
    NSEventMaskRotate = 1 << NSEventType::NSEventTypeRotate as u64,
    NSEventMaskBeginGesture = 1 << NSEventType::NSEventTypeBeginGesture as u64,
    NSEventMaskEndGesture = 1 << NSEventType::NSEventTypeEndGesture as u64,
    NSEventMaskPressure = 1 << NSEventType::NSEventTypePressure as u64,
    NSAnyEventMask = 0xffffffffffffffff,
}

#[repr(u64)] // NSUInteger
pub(crate) enum NSWindowStyleMask {
    NSBorderlessWindowMask = 0,
    NSTitledWindowMask = 1 << 0,
    NSClosableWindowMask = 1 << 1,
    NSMiniaturizableWindowMask = 1 << 2,
    NSResizableWindowMask = 1 << 3,

    NSTexturedBackgroundWindowMask = 1 << 8,

    NSUnifiedTitleAndToolbarWindowMask = 1 << 12,

    NSFullScreenWindowMask = 1 << 14,

    NSFullSizeContentViewWindowMask = 1 << 15,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NSApplicationActivationOptions {
    NSApplicationActivateAllWindows = 1 << 0,
    NSApplicationActivateIgnoringOtherApps = 1 << 1,
}

#[repr(i64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NSApplicationActivationPolicy {
    NSApplicationActivationPolicyRegular = 0,
    NSApplicationActivationPolicyAccessory = 1,
    NSApplicationActivationPolicyProhibited = 2,
    NSApplicationActivationPolicyERROR = -1,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NSBackingStoreType {
    NSBackingStoreRetained = 0,
    NSBackingStoreNonretained = 1,
    NSBackingStoreBuffered = 2,
}

#[repr(C)]
pub(crate) struct NSRange {
    pub(crate) location: u64,
    pub(crate) length: u64,
}
/*
impl NSRange {
    #[inline]
    pub(crate) fn new(location: u64, length: u64) -> NSRange {
        NSRange {location, length}
    }
}*/

unsafe impl Encode for NSRange {
    fn encode() -> Encoding {
        let encoding = format!(
            // TODO: Verify that this is correct
            "{{NSRange={}{}}}",
            u64::encode().as_str(),
            u64::encode().as_str(),
        );
        unsafe { Encoding::from_str(&encoding) }
    }
}

pub(crate) trait NSMutableAttributedString: Sized {
    unsafe fn alloc(_: Self) -> id {
        msg_send![class!(NSMutableAttributedString), alloc]
    }

    unsafe fn init(self) -> id;
    // *mut NSMutableAttributedString
    unsafe fn init_with_string(self, string: id) -> id;
    unsafe fn init_with_attributed_string(self, string: id) -> id;

    unsafe fn string(self) -> id;
    // *mut NSString
    unsafe fn mutable_string(self) -> id;
    // *mut NSMutableString
    unsafe fn length(self) -> u64;
}

impl NSMutableAttributedString for id {
    unsafe fn init(self) -> id {
        msg_send![self, init]
    }

    unsafe fn init_with_string(self, string: id) -> id {
        msg_send![self, initWithString: string]
    }

    unsafe fn init_with_attributed_string(self, string: id) -> id {
        msg_send![self, initWithAttributedString: string]
    }

    unsafe fn string(self) -> id {
        msg_send![self, string]
    }

    unsafe fn mutable_string(self) -> id {
        msg_send![self, mutableString]
    }

    unsafe fn length(self) -> u64 {
        msg_send![self, length]
    }
}

#[repr(u64)] // NSUInteger
pub(crate) enum NSApplicationDelegateReply {
    NSApplicationDelegateReplySuccess = 0,
    NSApplicationDelegateReplyCancel = 1,
    NSApplicationDelegateReplyFailure = 2,
}

// Metal API

#[repr(u64)]
#[derive(Clone, Debug)]
pub(crate) enum MTLLoadAction {
    DontCare = 0,
    Load = 1,
    Clear = 2,
}

#[repr(u64)]
#[derive(Clone, Debug)]
pub(crate) enum MTLStoreAction {
    DontCare = 0,
    Store = 1,
    MultisampleResolve = 2,
    StoreAndMultisampleResolve = 3,
    Unknown = 4,
    CustomSampleDepthStore = 5,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct MTLClearColor {
    pub(crate) red: f64,
    pub(crate) green: f64,
    pub(crate) blue: f64,
    pub(crate) alpha: f64,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) enum MTLPixelFormat {
    RGBA8Unorm = 70,
    BGRA8Unorm = 80,
    Depth32Float = 252,
    Stencil8 = 253,
    Depth24Unorm_Stencil8 = 255,
    Depth32Float_Stencil8 = 260,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum MTLBlendFactor {
    Zero = 0,
    One = 1,
    SourceColor = 2,
    OneMinusSourceColor = 3,
    SourceAlpha = 4,
    OneMinusSourceAlpha = 5,
    DestinationColor = 6,
    OneMinusDestinationColor = 7,
    DestinationAlpha = 8,
    OneMinusDestinationAlpha = 9,
    SourceAlphaSaturated = 10,
    BlendColor = 11,
    OneMinusBlendColor = 12,
    BlendAlpha = 13,
    OneMinusBlendAlpha = 14,
    Source1Color = 15,
    OneMinusSource1Color = 16,
    Source1Alpha = 17,
    OneMinusSource1Alpha = 18,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum MTLBlendOperation {
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum MTLPrimitiveType {
    Point = 0,
    Line = 1,
    LineStrip = 2,
    Triangle = 3,
    TriangleStrip = 4,
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum MTLIndexType {
    UInt16 = 0,
    UInt32 = 1,
}

#[repr(u64)]
pub(crate) enum MTLCompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum MTLTextureType {
    D1 = 0,
    D1Array = 1,
    D2 = 2,
    D2Array = 3,
    D2Multisample = 4,
    Cube = 5,
    CubeArray = 6,
    D3 = 7,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
pub(crate) enum MTLTextureUsage {
    Unknown = 0x0000,
    ShaderRead = 0x0001,
    ShaderWrite = 0x0002,
    RenderTarget = 0x0004,
    PixelFormatView = 0x0010,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum MTLStorageMode {
    Shared = 0,
    Managed = 1,
    Private = 2,
    Memoryless = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct MTLOrigin {
    pub(crate) x: u64,
    pub(crate) y: u64,
    pub(crate) z: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct MTLSize {
    pub(crate) width: u64,
    pub(crate) height: u64,
    pub(crate) depth: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct MTLRegion {
    pub(crate) origin: MTLOrigin,
    pub(crate) size: MTLSize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct CGSize {
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum MTLCPUCacheMode {
    DefaultCache = 0,
    WriteCombined = 1,
}

pub(crate) const MTLResourceCPUCacheModeShift: u64 = 0;
pub(crate) const MTLResourceCPUCacheModeMask: u64 = 0xf << MTLResourceCPUCacheModeShift;
pub(crate) const MTLResourceStorageModeShift: u64 = 4;
pub(crate) const MTLResourceStorageModeMask: u64 = 0xf << MTLResourceStorageModeShift;

#[allow(non_upper_case_globals)]
#[repr(u64)]
pub(crate) enum MTLResourceOptions {
    //CPUCacheModeDefaultCache  = (MTLCPUCacheMode::DefaultCache as u64) << MTLResourceCPUCacheModeShift,
    CPUCacheModeWriteCombined = (MTLCPUCacheMode::WriteCombined as u64) << MTLResourceCPUCacheModeShift,

    StorageModeShared = (MTLStorageMode::Shared as u64) << MTLResourceStorageModeShift,
    StorageModeManaged = (MTLStorageMode::Managed as u64) << MTLResourceStorageModeShift,
    StorageModePrivate = (MTLStorageMode::Private as u64) << MTLResourceStorageModeShift,
    StorageModeMemoryless = (MTLStorageMode::Memoryless as u64) << MTLResourceStorageModeShift,
}
