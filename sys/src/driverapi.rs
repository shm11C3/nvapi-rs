use crate::status::NvAPI_Status;
use crate::types::NvAPI_ShortString;
use crate::handles;

nvapi! {
    pub type SYS_GetDriverAndBranchVersionFn = extern "C" fn(pDriverVersion: *mut u32, szBuildBranchString: *mut NvAPI_ShortString) -> NvAPI_Status;

    /// This API returns display driver version and driver-branch string.
    pub unsafe fn NvAPI_SYS_GetDriverAndBranchVersion;
}

nvstruct! {
    /// Used in NvAPI_GPU_GetMemoryInfo().
    pub struct NV_DISPLAY_DRIVER_MEMORY_INFO_V1 {
        /// Version info
        pub version: u32,
        /// Size(in kb) of the physical framebuffer.
        pub dedicatedVideoMemory: u32,
        /// Size(in kb) of the available physical framebuffer for allocating video memory surfaces.
        pub availableDedicatedVideoMemory: u32,
        /// Size(in kb) of system memory the driver allocates at load time.
        pub systemVideoMemory: u32,
        /// Size(in kb) of shared system memory that driver is allowed to commit for surfaces across all allocations.
        pub sharedSystemMemory: u32,
    }
}

nvstruct! {
    /// Used in NvAPI_GPU_GetMemoryInfo().
    pub struct NV_DISPLAY_DRIVER_MEMORY_INFO_V2 {
        pub v1: NV_DISPLAY_DRIVER_MEMORY_INFO_V1,
        /// Size(in kb) of the current available physical framebuffer for allocating video memory surfaces.
        pub curAvailableDedicatedVideoMemory: u32,
    }
}
nvinherit! { NV_DISPLAY_DRIVER_MEMORY_INFO_V2(v1: NV_DISPLAY_DRIVER_MEMORY_INFO_V1) }

nvstruct! {
    /// Used in NvAPI_GPU_GetMemoryInfo().
    pub struct NV_DISPLAY_DRIVER_MEMORY_INFO_V3 {
        pub v2: NV_DISPLAY_DRIVER_MEMORY_INFO_V2,
        /// Size(in kb) of the total size of memory released as a result of the evictions.
        pub dedicatedVideoMemoryEvictionsSize: u32,
        /// Indicates the number of eviction events that caused an allocation to be removed from dedicated video memory to free GPU
        /// video memory to make room for other allocations.
        pub dedicatedVideoMemoryEvictionCount: u32,
    }
}
nvinherit! { NV_DISPLAY_DRIVER_MEMORY_INFO_V3(v2: NV_DISPLAY_DRIVER_MEMORY_INFO_V2) }

pub type NV_DISPLAY_DRIVER_MEMORY_INFO = NV_DISPLAY_DRIVER_MEMORY_INFO_V3;
nvversion! { NV_DISPLAY_DRIVER_MEMORY_INFO_VER_1(NV_DISPLAY_DRIVER_MEMORY_INFO_V1 = 4 * 5, 1) }
nvversion! { NV_DISPLAY_DRIVER_MEMORY_INFO_VER_2(NV_DISPLAY_DRIVER_MEMORY_INFO_V2 = 4 * 6, 2) }
nvversion! { NV_DISPLAY_DRIVER_MEMORY_INFO_VER_3(NV_DISPLAY_DRIVER_MEMORY_INFO_V3 = 4 * 8, 3) }
nvversion! { NV_DISPLAY_DRIVER_MEMORY_INFO_VER = NV_DISPLAY_DRIVER_MEMORY_INFO_VER_3 }

nvapi! {
    pub type GPU_GetMemoryInfoFn = extern "C" fn(hPhysicalGpu: handles::NvPhysicalGpuHandle, pMemoryInfo: *mut NV_DISPLAY_DRIVER_MEMORY_INFO) -> NvAPI_Status;

    /// This function retrieves the available driver memory footprint for the specified GPU.
    /// If the GPU is in TCC Mode, only dedicatedVideoMemory will be returned in pMemoryInfo (NV_DISPLAY_DRIVER_MEMORY_INFO).
    pub unsafe fn NvAPI_GPU_GetMemoryInfo;
}

/// Undocumented API
pub mod private {
    use crate::status::NvAPI_Status;
    use crate::handles;

    nvapi! {
        /// This has a different offset than the NvAPI_GPU_GetMemoryInfo function despite both returning the same struct
        pub unsafe fn NvAPI_GetDisplayDriverMemoryInfo(hPhysicalGpu: handles::NvPhysicalGpuHandle, pMemoryInfo: *mut super::NV_DISPLAY_DRIVER_MEMORY_INFO) -> NvAPI_Status;
    }
}

nvstruct! {
    /// Structure used in NvAPI_GPU_GetMemoryInfoEx.
    ///
    /// Detailed memory footprint for the GPU including usage and eviction info.
    pub struct NV_DISPLAY_DRIVER_MEMORY_INFO_EX_V1 {
        /// Version field must be set before calling the API.
        pub version: u32,

        /// Amount of dedicated video memory available for allocations (in KiB).
        pub availableDedicatedVideoMemory: u32,

        /// Total dedicated video memory (in KiB).
        pub dedicatedVideoMemory: u32,

        /// Amount of system video memory used (in KiB).
        pub systemVideoMemory: u32,

        /// Total shared system memory (in KiB).
        pub sharedSystemMemory: u32,

        /// Amount of dedicated video memory currently in use (in KiB).
        pub dedicatedVideoMemoryUsed: u32,

        /// Amount of shared system memory currently in use (in KiB).
        pub sharedSystemMemoryUsed: u32,
    }
}

pub type NV_GPU_MEMORY_INFO_EX_V1 = NV_DISPLAY_DRIVER_MEMORY_INFO_EX_V1;

nvapi! {
    pub type GPU_GetMemoryInfoExFn = extern "C" fn(hPhysicalGpu: handles::NvPhysicalGpuHandle, pMemoryInfo: *mut NV_GPU_MEMORY_INFO_EX_V1) -> NvAPI_Status;

    /// This function retrieves the available driver memory footprint for the specified GPU.
    /// If the GPU is in TCC Mode, only dedicatedVideoMemory will be returned in pMemoryInfo (NV_DISPLAY_DRIVER_MEMORY_INFO).
    pub unsafe fn NvAPI_GPU_GetMemoryInfoEx;
}
