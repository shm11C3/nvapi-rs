use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::{allowable_result, allowable_result_fallback};

use nvapi::{self, BaseVoltage, ClockEntry, ClockFrequencyType, ClockRange, ClockTable, Cooler, MemoryInfoEx, PStates, PowerInfoEntry, Sensor, ThermalInfo, ThermalLimit, VfpCurve, VfpEntry
};
pub use nvapi::{
    PhysicalGpu,
    Vendor, SystemType, RamType, RamMaker, Foundry,
    ClockFrequencies, ClockDomain, VoltageDomain, UtilizationDomain, Utilizations, ClockLockMode, ClockLockEntry,
    CoolerType, CoolerController, CoolerControl, CoolerPolicy, CoolerTarget, CoolerLevel,
    VoltageStatus, VoltageTable,
    PerfInfo, PerfStatus,
    ThermalController, ThermalTarget,
    MemoryInfo, PciIdentifiers, DriverModel,
    Percentage, Celsius,
    Range,
    Kibibytes, Microvolts, MicrovoltsDelta, Kilohertz, KilohertzDelta,
    PState,
};

pub struct Gpu {
    gpu: PhysicalGpu,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GpuInfo {
    pub name: String,
    pub codename: String,
    pub bios_version: String,
    pub driver_model: DriverModel,
    pub vendor: Vendor,
    pub pci: PciIdentifiers,
    pub memory: MemoryInfo,
    pub memory_ex: MemoryInfoEx,
    pub system_type: SystemType,
    pub ram_type: RamType,
    pub ram_maker: RamMaker,
    pub ram_bus_width: u32,
    pub ram_bank_count: u32,
    pub ram_partition_count: u32,
    pub foundry: Foundry,
    pub core_count: u32,
    pub shader_pipe_count: u32,
    pub shader_sub_pipe_count: u32,
    pub base_clocks: ClockFrequencies,
    pub boost_clocks: ClockFrequencies,
    pub sensors: Vec<SensorDesc>,
    pub coolers: Vec<CoolerDesc>,
    pub perf: PerfInfo,
    pub sensor_limits: Vec<SensorLimit>,
    pub power_limits: Vec<PowerLimit>,
    pub pstate_limits: BTreeMap<PState, BTreeMap<ClockDomain, PStateLimit>>,
    // TODO: pstate base_voltages
    pub overvolt_limits: Vec<OvervoltLimit>,
    pub vfp_limits: BTreeMap<ClockDomain, VfpRange>,
    pub vfp_locks: Vec<usize>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VfpRange {
    pub range: Range<KilohertzDelta>,
    pub temperature: Celsius,
}

impl From<ClockRange> for VfpRange {
    fn from(c: ClockRange) -> Self {
        VfpRange {
            range: Range::range_from(c.range),
            temperature: c.temp_max.into(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GpuStatus {
    pub pstate: PState,
    pub clocks: ClockFrequencies,
    pub memory: MemoryInfo,
    pub voltage: Option<Microvolts>,
    pub voltage_domains: Option<VoltageStatus>,
    pub voltage_step: Option<VoltageStatus>,
    pub voltage_table: Option<VoltageTable>,
    pub tachometer: Option<u32>,
    pub utilization: Utilizations,
    pub power: Vec<Percentage>,
    pub sensors: Vec<(SensorDesc, Celsius)>,
    pub coolers: Vec<(CoolerDesc, CoolerStatus)>,
    pub perf: PerfStatus,
    pub vfp: Option<VfpTable>,
    pub vfp_locks: BTreeMap<usize, Microvolts>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GpuSettings {
    pub voltage_boost: Option<Percentage>,
    pub sensor_limits: Vec<Celsius>,
    pub power_limits: Vec<Percentage>,
    pub coolers: Vec<(CoolerDesc, CoolerStatus)>,
    pub vfp: Option<VfpDeltas>,
    pub pstate_deltas: BTreeMap<PState, BTreeMap<ClockDomain, KilohertzDelta>>,
    pub overvolt: Vec<MicrovoltsDelta>,
    pub vfp_locks: BTreeMap<usize, ClockLockEntry>,
}

impl Gpu {
    pub fn new(gpu: PhysicalGpu) -> Self {
        Gpu {
            gpu: gpu,
        }
    }

    pub fn into_inner(self) -> PhysicalGpu {
        self.gpu
    }

    pub fn inner(&self) -> &PhysicalGpu {
        &self.gpu
    }

    pub fn enumerate() -> nvapi::Result<Vec<Self>> {
        PhysicalGpu::enumerate().map(|v| v.into_iter().map(Gpu::new).collect())
    }

    pub fn info(&self) -> nvapi::Result<GpuInfo> {
        let pstates = allowable_result(self.gpu.pstates())?;
        let (pstates, ov) = match pstates {
            Ok(PStates { editable: _editable, pstates, overvolt }) => (pstates, overvolt),
            Err(..) => (Default::default(), Default::default()),
        };
        let pci = self.gpu.pci_identifiers()?;

        Ok(GpuInfo {
            name: self.gpu.full_name()?,
            codename: self.gpu.short_name()?,
            bios_version: self.gpu.vbios_version_string()?,
            driver_model: self.gpu.driver_model()?,
            vendor: allowable_result_fallback(pci.vendor().map_err(From::from), Vendor::Unknown)?,
            pci: pci,
            memory: self.gpu.memory_info()?,
            memory_ex: self.gpu.memory_info_ex()?,
            system_type: allowable_result_fallback(self.gpu.system_type(), SystemType::Unknown)?,
            ram_type: allowable_result_fallback(self.gpu.ram_type(), RamType::Unknown)?,
            ram_maker: allowable_result_fallback(self.gpu.ram_maker(), RamMaker::Unknown)?,
            ram_bus_width: allowable_result_fallback(self.gpu.ram_bus_width(), 0)?,
            ram_bank_count: allowable_result_fallback(self.gpu.ram_bank_count(), 0)?,
            ram_partition_count: allowable_result_fallback(self.gpu.ram_partition_count(), 0)?,
            foundry: allowable_result_fallback(self.gpu.foundry(), Foundry::Unknown)?,
            core_count: self.gpu.core_count()?,
            shader_pipe_count: self.gpu.shader_pipe_count()?,
            shader_sub_pipe_count: self.gpu.shader_sub_pipe_count()?,
            base_clocks: self.gpu.clock_frequencies(ClockFrequencyType::Base)?,
            boost_clocks: self.gpu.clock_frequencies(ClockFrequencyType::Boost)?,
            sensors: match allowable_result(self.gpu.thermal_settings(None))? {
                Ok(s) => s.into_iter().map(From::from).collect(),
                Err(..) => Default::default(),
            },
            coolers: match allowable_result(self.gpu.cooler_settings(None))? {
                Ok(c) => c.into_iter().map(From::from).collect(),
                Err(..) => Default::default(),
            },
            perf: self.gpu.perf_info()?,
            sensor_limits: match allowable_result(self.gpu.thermal_limit_info())? {
                Ok((_, l)) => l.into_iter().map(From::from).collect(),
                Err(..) => Default::default(),
            },
            power_limits: match allowable_result(self.gpu.power_limit_info())? {
                Ok(p) => p.entries.into_iter().map(From::from).collect(),
                Err(..) => Default::default(),
            },
            pstate_limits: pstates.into_iter().map(|p| (p.id, p.clocks.into_iter().map(|p| (p.domain(), p.into())).collect())).collect(),
            overvolt_limits: ov.into_iter().map(From::from).collect(),
            vfp_limits: match allowable_result(self.gpu.vfp_ranges())? {
                Ok(l) => l.into_iter().map(|v| (v.domain, v.into())).collect(),
                Err(..) => Default::default(),
            },
            vfp_locks: match allowable_result(self.gpu.vfp_locks())? {
                Ok(v) => v.into_iter().map(|(id, _)| id).collect(),
                Err(..) => Default::default(),
            },
        })
    }

    pub fn status(&self) -> nvapi::Result<GpuStatus> {
        let mask = allowable_result(self.gpu.vfp_mask())?;

        Ok(GpuStatus {
            pstate: self.gpu.current_pstate()?,
            clocks: self.gpu.clock_frequencies(ClockFrequencyType::Current)?,
            memory: self.gpu.memory_info()?,
            voltage: allowable_result(self.gpu.core_voltage())?.ok(),
            voltage_domains: allowable_result(self.gpu.voltage_domains_status())?.ok(),
            voltage_step: allowable_result(self.gpu.voltage_step())?.ok(),
            voltage_table: allowable_result(self.gpu.voltage_table())?.ok(),
            tachometer: allowable_result(self.gpu.tachometer())?.ok(),
            utilization: self.gpu.dynamic_pstates_info()?,
            power: self.gpu.power_usage()?.into_iter().map(From::from).collect(),
            sensors: match allowable_result(self.gpu.thermal_settings(None))? {
                Ok(s) => s.into_iter().map(|s| (From::from(s), s.current_temperature)).collect(),
                Err(..) => Default::default(),
            },
            coolers: match allowable_result(self.gpu.cooler_settings(None))? {
                Ok(c) => c.into_iter().map(|c| (From::from(c), From::from(c))).collect(),
                Err(..) => Default::default(),
            },
            perf: self.gpu.perf_status()?,
            vfp: match mask {
                Ok(mask) => allowable_result(self.gpu.vfp_curve(mask.mask))?.map(From::from).ok(),
                Err(..) => None,
            },
            vfp_locks: match allowable_result(self.gpu.vfp_locks())? {
                Ok(l) => l.into_iter().filter_map(|(id, e)| if e.mode == ClockLockMode::Manual {
                    Some((id, e.voltage))
                } else {
                    None
                }).collect(),
                Err(..) => Default::default(),
            },
        })
    }

    pub fn settings(&self) -> nvapi::Result<GpuSettings> {
        let mask = allowable_result(self.gpu.vfp_mask())?;
        let pstates = allowable_result(self.gpu.pstates())?;
        let (pstates, ov) = match pstates {
            Ok(PStates { editable: _editable, pstates, overvolt }) => (pstates, overvolt),
            Err(..) => (Default::default(), Default::default()),
        };

        Ok(GpuSettings {
            voltage_boost: allowable_result(self.gpu.core_voltage_boost())?.ok(),
            sensor_limits: match allowable_result(self.gpu.thermal_limit())? {
                Ok(l) => l.into_iter().map(|l| l.value.into()).collect(),
                Err(..) => Default::default(),
            },
            power_limits: match allowable_result(self.gpu.power_limit())? {
                Ok(l) => l.into_iter().map(|l| l.into()).collect(),
                Err(..) => Default::default(),
            },
            coolers: match allowable_result(self.gpu.cooler_settings(None))? {
                Ok(c) => c.into_iter().map(|c| (From::from(c), From::from(c))).collect(),
                Err(..) => Default::default(),
            },
            vfp: match mask {
                Ok(mask) => allowable_result(self.gpu.vfp_table(mask.mask))?.map(From::from).ok(),
                Err(..) => None,
            },
            vfp_locks: match allowable_result(self.gpu.vfp_locks())? {
                Ok(l) => l,
                Err(..) => Default::default(),
            },
            pstate_deltas: pstates.into_iter().filter(|p| p.editable)
                .map(|p| (p.id, p.clocks.into_iter().filter(|p| p.editable())
                    .map(|p| (p.domain(), p.frequency_delta().value)).collect())
                ).collect(),
            overvolt: ov.into_iter().filter(|v| v.editable).map(|v| v.voltage_delta.value).collect(),
        })
    }

    pub fn set_voltage_boost(&self, boost: Percentage) -> nvapi::Result<()> {
        self.gpu.set_core_voltage_boost(boost)
    }

    pub fn set_power_limits<I: Iterator<Item=Percentage>>(&self, limits: I) -> nvapi::Result<()> {
        // TODO: match against power_limit_info, use range.min/max from there if it matches (can get fraction of a percent!)
        self.gpu.set_power_limit(limits.map(From::from))
    }

    pub fn set_sensor_limits<I: Iterator<Item=Celsius>>(&self, limits: I) -> nvapi::Result<()> {
        self.gpu.thermal_limit_info().and_then(|(_, info)| self.gpu.set_thermal_limit(
            limits.zip(info.into_iter()).map(|(limit, info)| ThermalLimit {
                controller: info.controller,
                flags: info.default_flags,
                value: limit.into(),
            })
        ))
    }

    pub fn set_cooler_levels<I: Iterator<Item=CoolerLevel>>(&self, levels: I) -> nvapi::Result<()> {
        self.gpu.set_cooler_levels(None, levels)
    }

    pub fn reset_cooler_levels(&self) -> nvapi::Result<()> {
        self.gpu.restore_cooler_settings(&[])
    }

    pub fn set_vfp<I: Iterator<Item=(usize, KilohertzDelta)>, M: Iterator<Item=(usize, KilohertzDelta)>>(&self, clock_deltas: I, mem_deltas: M) -> nvapi::Result<()> {
        self.gpu.set_vfp_table([0, 0, 0, 0], clock_deltas.map(|(i, d)| (i, d.into())), mem_deltas.map(|(i, d)| (i, d.into())))
    }

    pub fn set_vfp_lock(&self, voltage: Microvolts) -> nvapi::Result<()> {
        self.gpu.set_vfp_locks(self.gpu.vfp_locks()?
            .into_iter().max_by_key(|&(id, _)| id).into_iter()
            .map(|(id, entry)| (id, Some(voltage)))
        )
    }

    pub fn reset_vfp_lock(&self) -> nvapi::Result<()> {
        self.gpu.set_vfp_locks(self.gpu.vfp_locks()?.into_iter().map(|(id, _)| (id, None)))
    }

    pub fn reset_vfp(&self) -> nvapi::Result<()> {
        use std::iter;

        let mask = self.gpu.vfp_mask()?;
        self.gpu.set_vfp_table(mask.mask, iter::empty(), iter::empty())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct OvervoltLimit {
    pub domain: VoltageDomain,
    pub voltage: Microvolts,
    pub range: Option<Range<MicrovoltsDelta>>,
}

impl From<BaseVoltage> for OvervoltLimit {
    fn from(v: BaseVoltage) -> Self {
        OvervoltLimit {
            domain: v.voltage_domain,
            voltage: v.voltage,
            range: if v.editable {
                Some(v.voltage_delta.range)
            } else {
                None
            },
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PStateLimit {
    pub frequency_delta: Option<Range<KilohertzDelta>>,
    pub frequency: Range<Kilohertz>,
    pub voltage: Range<Microvolts>,
    pub voltage_domain: VoltageDomain,
}

impl From<ClockEntry> for PStateLimit {
    fn from(s: ClockEntry) -> Self {
        match s {
            ClockEntry::Range { domain: _, editable, frequency_delta, frequency_range, voltage_domain, voltage_range } => PStateLimit {
                frequency_delta: if editable { Some(frequency_delta.range) } else { None },
                frequency: frequency_range,
                voltage: voltage_range,
                voltage_domain: voltage_domain,
            },
            ClockEntry::Single { domain: _, editable, frequency_delta, frequency } => PStateLimit {
                frequency_delta: if editable { Some(frequency_delta.range) } else { None },
                frequency: Range::from_scalar(frequency),
                voltage: Default::default(),
                voltage_domain: VoltageDomain::Undefined,
            },
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PowerLimit {
    pub range: Range<Percentage>,
    pub default: Percentage,
}

impl From<PowerInfoEntry> for PowerLimit {
    fn from(info: PowerInfoEntry) -> Self {
        PowerLimit {
            range: Range::range_from(info.range),
            default: info.default_limit.into(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SensorLimit {
    pub range: Range<Celsius>,
    pub default: Celsius,
    pub flags: u32,
}

impl From<ThermalInfo> for SensorLimit {
    fn from(info: ThermalInfo) -> Self {
        SensorLimit {
            range: Range::range_from(info.temperature_range),
            default: info.default_temperature.into(),
            flags: info.default_flags,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SensorDesc {
    pub controller: ThermalController,
    pub target: ThermalTarget,
    pub range: Range<Celsius>,
}

impl From<Sensor> for SensorDesc {
    fn from(sensor: Sensor) -> Self {
        SensorDesc {
            controller: sensor.controller,
            target: sensor.target,
            range: sensor.default_temperature_range,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct CoolerDesc {
    pub kind: CoolerType,
    pub controller: CoolerController,
    pub range: Range<Percentage>,
    pub default_policy: CoolerPolicy,
    pub target: CoolerTarget,
    pub control: CoolerControl,
}

impl From<Cooler> for CoolerDesc {
    fn from(cooler: Cooler) -> Self {
        CoolerDesc {
            kind: cooler.kind,
            controller: cooler.controller,
            range: cooler.default_level_range,
            default_policy: cooler.default_policy,
            target: cooler.target,
            control: cooler.control,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct CoolerStatus {
    pub range: Range<Percentage>,
    pub level: Percentage,
    pub policy: CoolerPolicy,
    pub active: bool,
}

impl From<Cooler> for CoolerStatus {
    fn from(cooler: Cooler) -> Self {
        CoolerStatus {
            range: cooler.current_level_range,
            level: cooler.current_level,
            policy: cooler.current_policy,
            active: cooler.active,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VfpPoint {
    pub frequency: Kilohertz,
    pub voltage: Microvolts,
}

impl<T> From<VfpEntry<T>> for VfpPoint where Kilohertz: From<T> {
    fn from(v: VfpEntry<T>) -> Self {
        VfpPoint {
            frequency: v.frequency.into(),
            voltage: v.voltage,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VfpTable {
    pub graphics: BTreeMap<usize, VfpPoint>,
    pub memory: BTreeMap<usize, VfpPoint>,
}

impl From<VfpCurve> for VfpTable {
    fn from(v: VfpCurve) -> Self {
        VfpTable {
            graphics: v.graphics.into_iter().map(|(i, e)| (i, e.into())).collect(),
            memory: v.memory.into_iter().map(|(i, e)| (i, e.into())).collect(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VfpDeltas {
    pub graphics: BTreeMap<usize, KilohertzDelta>,
    pub memory: BTreeMap<usize, KilohertzDelta>,
}

impl From<ClockTable> for VfpDeltas {
    fn from(c: ClockTable) -> Self {
        VfpDeltas {
            graphics: c.gpu_delta.into_iter().map(|(i, d)| (i, d.into())).collect(),
            memory: c.mem_delta.into_iter().map(|(i, d)| (i, d.into())).collect(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VfPoint {
    pub voltage: Microvolts,
    pub frequency: Kilohertz,
    pub delta: KilohertzDelta,
}

impl VfPoint {
    pub fn new(point: VfpPoint, delta: KilohertzDelta) -> Self {
        VfPoint {
            voltage: point.voltage,
            frequency: point.frequency,
            delta: delta,
        }
    }
}
