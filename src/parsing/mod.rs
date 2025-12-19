use defmt::debug;

use crate::register::*;

const REPORT_LENGTHS: &[(ReportID, u8)] = &[
    (ReportID::AccelerometerCalibrated, 10),
    (ReportID::GyroscopeCalibrated, 10),
    (ReportID::MagFieldCalibrated, 10),
    (ReportID::RotationVector, 14),
];

pub fn get_report_length(report_id: u8) -> Option<(ReportID, u8)> {
    for (r_id, data) in REPORT_LENGTHS {
        if *r_id as u8 == report_id {
            return Some((*r_id, *data));
        }
    }
    None
}
