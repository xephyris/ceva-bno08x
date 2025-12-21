use crate::register::*;

const REPORT_LENGTHS: &[(ReportId, u8)] = &[
    (ReportId::AccelerometerCalibrated, 10),
    (ReportId::AccelerationLinear, 10),
    (ReportId::Gravity, 10),
    (ReportId::AccelerometerRaw, 16),
    (ReportId::GyroscopeCalibrated, 10),
    (ReportId::GyroscopeUncalibrated, 16),
    (ReportId::GyroscopeRaw, 16),
    (ReportId::MagFieldCalibrated, 10),
    (ReportId::MagFieldUncalibrated, 16),
    (ReportId::MagnetometerRaw, 16),
    (ReportId::GeomagneticRotVector, 14),
    (ReportId::GameRotationVector, 12),
    (ReportId::ARVRStabilizedGameVec, 12),
    (ReportId::RotationVector, 14),
    (ReportId::ARVRStabilizedRotVec, 14),
    (ReportId::GyroIntegratedRotVec, 14),
    (ReportId::StabilityClassifier, 6),
    (ReportId::StabilityDetector, 5),
    (ReportId::TapDetector, 5),
    (ReportId::StepDetector, 8),
    (ReportId::StepCounter, 12),
    (ReportId::PersonalActClassifier, 16),
    (ReportId::SignificantMotion, 6),
    (ReportId::ShakeDetector, 6),
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

pub fn get_control_report_length(report_id: u8) -> Option<(SH2Read, u8)> {
    for (r_id, data) in CONTROL_REPORT_LENGTHS {
        if *r_id as u8 == report_id {
            return Some((*r_id, *data));
        }
    }
    None
}
