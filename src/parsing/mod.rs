use defmt::Format;
use heapless::Vec;

use crate::{data, register::*};

const REPORT_LENGTHS: &[(ReportId, u8)] = &[
    (ReportId::AccelerometerRaw, 16),
    (ReportId::AccelerometerCalibrated, 10),
    (ReportId::AccelerationLinear, 10),
    (ReportId::Gravity, 10),
    (ReportId::GyroscopeRaw, 16),
    (ReportId::GyroscopeCalibrated, 10),
    (ReportId::GyroscopeUncalibrated, 16),
    (ReportId::MagnetometerRaw, 16),
    (ReportId::MagFieldCalibrated, 10),
    (ReportId::MagFieldUncalibrated, 16),
    (ReportId::RotationVector, 14),
    (ReportId::GameRotationVector, 12),
    (ReportId::GeomagneticRotVector, 14),
    (ReportId::TapDetector, 5),
    (ReportId::StepDetector, 8),
    (ReportId::StepCounter, 12),
    (ReportId::SignificantMotion, 6),
    (ReportId::StabilityClassifier, 6),
    (ReportId::ShakeDetector, 6),
    (ReportId::FlipDetector, 6),
    (ReportId::PickupDetector, 6),
    (ReportId::StabilityDetector, 6),
    (ReportId::PersonalActClassifier, 16),
    (ReportId::SleepDetector, 6),
    (ReportId::TiltDetector, 6),
    (ReportId::CircleDetector, 6),
    (ReportId::ARVRStabilizedRotVec, 14),
    (ReportId::ARVRStabilizedGameVec, 12),
    (ReportId::GyroIntegratedRotVec, 14),
];

const FEATURE_DEPENDENCIES: &[(ReportId, &[ReportId])] = &[
    (
        ReportId::AccelerometerRaw,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::AccelerationLinear,
        &[ReportId::AccelerometerCalibrated],
    ),
    (ReportId::Gravity, &[ReportId::AccelerometerCalibrated]),
    (
        ReportId::GyroscopeCalibrated,
        &[ReportId::AccelerometerCalibrated],
    ),
    (ReportId::GyroscopeRaw, &[ReportId::GyroscopeCalibrated]),
    (
        ReportId::GyroscopeUncalibrated,
        &[
            ReportId::GyroscopeCalibrated,
            ReportId::AccelerometerCalibrated,
        ],
    ),
    (ReportId::MagnetometerRaw, &[ReportId::MagFieldCalibrated]),
    (
        ReportId::MagFieldCalibrated,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::MagFieldUncalibrated,
        &[
            ReportId::MagFieldCalibrated,
            ReportId::AccelerometerCalibrated,
        ],
    ),
    (
        ReportId::RotationVector,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
            ReportId::MagFieldCalibrated,
        ],
    ),
    (
        ReportId::GameRotationVector,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
        ],
    ),
    (
        ReportId::GeomagneticRotVector,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::MagFieldCalibrated,
        ],
    ),
    (ReportId::TapDetector, &[ReportId::AccelerometerCalibrated]),
    (ReportId::StepDetector, &[ReportId::AccelerometerCalibrated]),
    (ReportId::StepCounter, &[ReportId::AccelerometerCalibrated]),
    (
        ReportId::SignificantMotion,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::StabilityClassifier,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
        ],
    ),
    (
        ReportId::ShakeDetector,
        &[ReportId::AccelerometerCalibrated],
    ),
    (ReportId::FlipDetector, &[ReportId::AccelerometerCalibrated]),
    (
        ReportId::PickupDetector,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::StabilityDetector,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::PersonalActClassifier,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::SleepDetector,
        &[ReportId::AccelerometerCalibrated],
    ),
    (ReportId::TiltDetector, &[ReportId::AccelerometerCalibrated]),
    (
        ReportId::CircleDetector,
        &[ReportId::AccelerometerCalibrated],
    ),
    (
        ReportId::ARVRStabilizedRotVec,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
            ReportId::MagFieldCalibrated,
        ],
    ),
    (
        ReportId::ARVRStabilizedGameVec,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
        ],
    ),
    (
        ReportId::GyroIntegratedRotVec,
        &[
            ReportId::AccelerometerCalibrated,
            ReportId::GyroscopeCalibrated,
            ReportId::MagFieldCalibrated,
        ],
    ),
];

#[derive(Format)]
pub enum DataVals {
    I16(i16),
    I32(i32),
    Reserved,
    U8(u8),
    U16(u16),
    U32(u32),
}

impl DataVals {
    pub fn get_value(bytes: &[u8], kind: DataTypes) -> Self {
        match kind {
            DataTypes::I16 => {
                DataVals::I16(i16::from_le_bytes(bytes.try_into().unwrap_or([0_u8; 2])))
            }
            DataTypes::I32 => {
                DataVals::I32(i32::from_le_bytes(bytes.try_into().unwrap_or([0_u8; 4])))
            }
            DataTypes::Reserved => DataVals::Reserved,
            DataTypes::U8 => DataVals::U8(if bytes.len() > 0 { bytes[1] } else { 0 }),
            DataTypes::U16 => {
                DataVals::U16(u16::from_le_bytes(bytes.try_into().unwrap_or([0_u8; 2])))
            }
            DataTypes::U32 => {
                DataVals::U32(u32::from_le_bytes(bytes.try_into().unwrap_or([0_u8; 4])))
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum DataTypes {
    I16,
    I32,
    Reserved,
    U8,
    U16,
    U32,
}

// Report data types after first 4 status bytes
const REPORT_VALS: &[(ReportId, &[DataTypes])] = &[
    (
        ReportId::AccelerometerCalibrated,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
        ],
    ),
    (
        ReportId::AccelerationLinear,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
        ],
    ),
    (
        ReportId::Gravity,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
        ],
    ),
    (
        ReportId::AccelerometerRaw,
        &[
            DataTypes::U16, // X
            DataTypes::U16, // Y
            DataTypes::U16, // Z
            DataTypes::Reserved,
            DataTypes::Reserved,
            DataTypes::U32, // Timestamp
        ],
    ),
    (
        ReportId::GyroscopeCalibrated,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
        ],
    ),
    (
        ReportId::GyroscopeUncalibrated,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
            DataTypes::I16, // X Bias
            DataTypes::I16, // Y Bias
            DataTypes::I16, // Z Bias
        ],
    ),
    (
        ReportId::GyroscopeRaw,
        &[
            DataTypes::U16, // X
            DataTypes::U16, // Y
            DataTypes::U16, // Z
            DataTypes::U16, // Gyro Temp
            DataTypes::U32, // Timestamp
        ],
    ),
    (
        ReportId::MagFieldCalibrated,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
        ],
    ),
    (
        ReportId::MagFieldUncalibrated,
        &[
            DataTypes::I16, // X
            DataTypes::I16, // Y
            DataTypes::I16, // Z
            DataTypes::I16, // X Bias
            DataTypes::I16, // Y Bias
            DataTypes::I16, // Z Bias
        ],
    ),
    (
        ReportId::MagnetometerRaw,
        &[
            DataTypes::U16, // X
            DataTypes::U16, // Y
            DataTypes::U16, // Z
            DataTypes::Reserved,
            DataTypes::Reserved,
            DataTypes::U32, // Timestamp
        ],
    ),
    (
        ReportId::GeomagneticRotVector,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
            DataTypes::I16, // Accuracy Estimate
        ],
    ),
    (
        ReportId::GameRotationVector,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
        ],
    ),
    (
        ReportId::ARVRStabilizedGameVec,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
        ],
    ),
    (
        ReportId::RotationVector,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
            DataTypes::I16, // Accuracy Estimate
        ],
    ),
    (
        ReportId::ARVRStabilizedRotVec,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
            DataTypes::I16, // Accuracy Estimate
        ],
    ),
    (
        ReportId::GyroIntegratedRotVec,
        &[
            DataTypes::I16, // I
            DataTypes::I16, // J
            DataTypes::I16, // K
            DataTypes::I16, // Real
            DataTypes::I16, // Ang Vel X
            DataTypes::I16, // Ang Vel Y
            DataTypes::I16, // Ang Vel Z
        ],
    ),
    (
        ReportId::StabilityClassifier,
        &[DataTypes::U8, DataTypes::Reserved],
    ),
    (ReportId::StabilityDetector, &[DataTypes::U16]),
    (ReportId::TapDetector, &[DataTypes::U8]),
    (ReportId::StepDetector, &[DataTypes::U32]),
    (
        ReportId::StepCounter,
        &[
            DataTypes::U32,
            DataTypes::U16,
            DataTypes::Reserved,
            DataTypes::Reserved,
        ],
    ),
    (
        ReportId::PersonalActClassifier,
        &[
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
            DataTypes::U8,
        ],
    ),
    (ReportId::SignificantMotion, &[DataTypes::U16]),
    (ReportId::ShakeDetector, &[DataTypes::U16]),
];

const CONTROL_REPORT_LENGTHS: &[(SH2Read, u8)] = &[
    (SH2Read::CommandResponse, 16),
    (SH2Read::FrsReadResponse, 16),
    (SH2Read::GetFeatureResponse, 17),
    (SH2Read::ProductIDResponse, 16),
];

pub fn q_point_processing(data: i16, q_point: u8) -> f32 {
    data as f32 / 2_u16.pow(q_point as u32) as f32
}

pub fn get_report_length(report_id: u8) -> Option<(ReportId, u8)> {
    for (r_id, data) in REPORT_LENGTHS {
        if *r_id as u8 == report_id {
            return Some((*r_id, *data));
        }
    }
    None
}

pub fn get_report_format(report_id: ReportId) -> Option<(ReportId, &'static [DataTypes])> {
    for (r_id, data) in REPORT_VALS {
        if *r_id == report_id {
            return Some((*r_id, *data));
        }
    }
    None
}

pub fn process_buf(data_format: &[DataTypes], buf: &[u8]) -> Vec<DataVals, 20> {
    let mut buf_index = 0;
    let mut output: Vec<DataVals, 20> = Vec::new();

    for format in data_format {
        match format {
            DataTypes::I16 => {
                output
                    .push(DataVals::get_value(
                        &buf[(buf_index)..(buf_index + 2)],
                        *format,
                    ))
                    .ok();
                buf_index += 2;
            }
            DataTypes::I32 => {
                output
                    .push(DataVals::get_value(
                        &buf[(buf_index)..(buf_index + 4)],
                        *format,
                    ))
                    .ok();
                buf_index += 2;
            }
            DataTypes::U8 => {
                output
                    .push(DataVals::get_value(
                        &buf[(buf_index)..(buf_index + 1)],
                        *format,
                    ))
                    .ok();
                buf_index += 2;
            }
            DataTypes::U16 => {
                output
                    .push(DataVals::get_value(
                        &buf[(buf_index)..(buf_index + 2)],
                        *format,
                    ))
                    .ok();
                buf_index += 2;
            }
            DataTypes::U32 => {
                output
                    .push(DataVals::get_value(
                        &buf[(buf_index)..(buf_index + 4)],
                        *format,
                    ))
                    .ok();
                buf_index += 2;
            }
            DataTypes::Reserved => {}
        }
    }
    return output;
}

pub fn get_feature_dependencies(report_id: ReportId) -> &'static [ReportId] {
    for (r_id, deps) in FEATURE_DEPENDENCIES {
        if *r_id == report_id {
            return deps;
        }
    }
    &[]
}

pub fn get_control_report_length(report_id: u8) -> Option<(SH2Read, u8)> {
    for (r_id, data) in CONTROL_REPORT_LENGTHS {
        if *r_id as u8 == report_id {
            return Some((*r_id, *data));
        }
    }
    None
}
