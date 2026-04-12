use limine::request::{
    MemmapRequest, HhdmRequest, ExecutableAddressRequest, FramebufferRequest,
};
use limine::{BaseRevision, RequestsStartMarker, RequestsEndMarker};

/// Limine base revision (protocol version).
#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Limine requests section markers.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

/// Request the memory map from the bootloader.
#[used]
#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP: MemmapRequest = MemmapRequest::new();

/// Request the Higher Half Direct Map offset.
#[used]
#[unsafe(link_section = ".requests")]
pub static HHDM: HhdmRequest = HhdmRequest::new();

/// Request the kernel's physical and virtual base addresses.
#[used]
#[unsafe(link_section = ".requests")]
pub static KERNEL_ADDRESS: ExecutableAddressRequest = ExecutableAddressRequest::new();

/// Request a framebuffer for graphical output.
#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();
