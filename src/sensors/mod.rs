use defmt::*;

use crate::{
    config::*,
    parsing::{get_report_format, process_buf, q_point_processing},
    register::{ReportId, Status},
};
#[derive(Debug)]
pub struct Sensors {
    pub acceleration: (Status, f32, f32, f32), // Calibrated
    pub accel_raw: (Status, u16, u16, u16, u32),
    pub linear_accel: (Status, f32, f32, f32),
    pub gravity: (Status, f32, f32, f32),
    pub gyroscope: (Status, f32, f32, f32), // Calibrated
    pub gyro_raw: (Status, u16, u16, u16, u16, u32),
    pub magnetometer: (Status, f32, f32, f32), // Calibrated Mag Field
    pub mag_raw: (Status, u16, u16, u16, u32),
    pub quaternions: (Status, f32, f32, f32, f32),
    pub game_quaternions: (Status, f32, f32, f32, f32),
    pub geomag_quaternions: (Status, f32, f32, f32, f32),
}

impl Sensors {
    pub fn new() -> Sensors {
        Sensors {
            acceleration: (Status::Unknown, 0.0, 0.0, 0.0),
            accel_raw: (Status::Unknown, 0, 0, 0, 0),
            linear_accel: (Status::Unknown, 0.0, 0.0, 0.0),
            gravity: (Status::Unknown, 0.0, 0.0, 0.0),
            gyroscope: (Status::Unknown, 0.0, 0.0, 0.0),
            gyro_raw: (Status::Unknown, 0, 0, 0, 0, 0),
            magnetometer: (Status::Unknown, 0.0, 0.0, 0.0),
            mag_raw: (Status::Unknown, 0, 0, 0, 0),
            quaternions: (Status::Unknown, 0.0, 0.0, 0.0, 0.0),
            game_quaternions: (Status::Unknown, 0.0, 0.0, 0.0, 0.0),
            geomag_quaternions: (Status::Unknown, 0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn update_data(&mut self, report_id: ReportId, data_slice: &[u8], report_slice: &[u8]) {
        let data_types = get_report_format(report_id);
        let (status, _delay) = parse_status(report_slice);
        if let Some((report_id, data_format)) = data_types {
            match report_id {
                ReportId::AccelerometerCalibrated => {
                    let out = process_buf(data_format, data_slice);
                    let mut accel_vals = [0.0_f32; 3];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 3 {
                                    accel_vals[index] =
                                        q_point_processing(*num, ACCEL_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.acceleration.0 = status;
                    self.acceleration.1 = accel_vals[0];
                    self.acceleration.2 = accel_vals[1];
                    self.acceleration.3 = accel_vals[2];
                }
                ReportId::AccelerometerRaw => {
                    let out = process_buf(data_format, data_slice);
                    let mut accel_vals = [0_u32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::U16(num) => {
                                if index < 3 {
                                    accel_vals[index] = *num as u32;
                                }
                            }
                            crate::parsing::DataVals::U32(timestamp) => {
                                if index == 3 {
                                    accel_vals[3] = *timestamp;
                                }
                            }
                            _ => {}
                        }
                    }
                    self.accel_raw.0 = status;
                    self.accel_raw.1 = accel_vals[0] as u16;
                    self.accel_raw.2 = accel_vals[1] as u16;
                    self.accel_raw.3 = accel_vals[2] as u16;
                    self.accel_raw.4 = accel_vals[3];
                }
                ReportId::AccelerationLinear => {
                    let out = process_buf(data_format, data_slice);
                    let mut linear = [0.0_f32; 3];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 3 {
                                    linear[index] = q_point_processing(*num, ACCEL_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.linear_accel.0 = status;
                    self.linear_accel.1 = linear[0];
                    self.linear_accel.2 = linear[1];
                    self.linear_accel.3 = linear[2];
                }
                ReportId::Gravity => {
                    let out = process_buf(data_format, data_slice);
                    let mut grav = [0.0_f32; 3];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 3 {
                                    grav[index] = q_point_processing(*num, ACCEL_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.gravity.0 = status;
                    self.gravity.1 = grav[0];
                    self.gravity.2 = grav[1];
                    self.gravity.3 = grav[2];
                }
                ReportId::GyroscopeCalibrated => {
                    let out = process_buf(data_format, data_slice);
                    let mut gyro_vals = [0.0_f32; 3];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 3 {
                                    gyro_vals[index] = q_point_processing(*num, GYRO_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.gyroscope.0 = status;
                    self.gyroscope.1 = gyro_vals[0];
                    self.gyroscope.2 = gyro_vals[1];
                    self.gyroscope.3 = gyro_vals[2];
                }
                ReportId::GyroscopeRaw => {
                    let out = process_buf(data_format, data_slice);
                    let mut gyro_vals = [0_u32; 5];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::U16(num) => {
                                if index < 4 {
                                    gyro_vals[index] = *num as u32;
                                }
                            }
                            crate::parsing::DataVals::U32(timestamp) => {
                                if index == 4 {
                                    gyro_vals[index] = *timestamp;
                                }
                            }
                            _ => {}
                        }
                    }
                    self.gyro_raw.0 = status;
                    self.gyro_raw.1 = gyro_vals[0] as u16;
                    self.gyro_raw.2 = gyro_vals[1] as u16;
                    self.gyro_raw.3 = gyro_vals[2] as u16;
                    self.gyro_raw.4 = gyro_vals[3] as u16;
                    self.gyro_raw.5 = gyro_vals[4];
                }
                ReportId::MagFieldCalibrated => {
                    let out = process_buf(data_format, data_slice);
                    let mut mag_vals = [0.0_f32; 3];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 3 {
                                    mag_vals[index] = q_point_processing(*num, MAG_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.magnetometer.0 = status;
                    self.magnetometer.1 = mag_vals[0];
                    self.magnetometer.2 = mag_vals[1];
                    self.magnetometer.3 = mag_vals[2];
                }
                ReportId::MagnetometerRaw => {
                    let out = process_buf(data_format, data_slice);
                    let mut mag_vals = [0_u32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::U16(num) => {
                                if index < 3 {
                                    mag_vals[index] = *num as u32;
                                }
                            }
                            crate::parsing::DataVals::U32(timestamp) => {
                                if index == 3 {
                                    mag_vals[3] = *timestamp;
                                }
                            }
                            _ => {}
                        }
                    }
                    self.mag_raw.0 = status;
                    self.mag_raw.1 = mag_vals[0] as u16;
                    self.mag_raw.2 = mag_vals[1] as u16;
                    self.mag_raw.3 = mag_vals[2] as u16;
                    self.mag_raw.4 = mag_vals[3];
                }
                ReportId::RotationVector => {
                    let out = process_buf(data_format, data_slice);
                    let mut quat_vals = [0.0_f32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 4 {
                                    quat_vals[index] = q_point_processing(*num, QUAT_SCALAR_Q_POINT)
                                } else if index == 4 {
                                    debug!(
                                        "Quaternion processing accuracy: {}",
                                        q_point_processing(*num, 12)
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    self.quaternions.0 = status;
                    self.quaternions.1 = quat_vals[0];
                    self.quaternions.2 = quat_vals[1];
                    self.quaternions.3 = quat_vals[2];
                    self.quaternions.4 = quat_vals[3];
                }
                ReportId::GameRotationVector => {
                    let out = process_buf(data_format, data_slice);
                    let mut quat_vals = [0.0_f32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 4 {
                                    quat_vals[index] = q_point_processing(*num, QUAT_SCALAR_Q_POINT)
                                }
                            }
                            _ => {}
                        }
                    }
                    self.game_quaternions.0 = status;
                    self.game_quaternions.1 = quat_vals[0];
                    self.game_quaternions.2 = quat_vals[1];
                    self.game_quaternions.3 = quat_vals[2];
                    self.game_quaternions.4 = quat_vals[3];
                }
                ReportId::GeomagneticRotVector => {
                    let out = process_buf(data_format, data_slice);
                    let mut quat_vals = [0.0_f32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 4 {
                                    quat_vals[index] = q_point_processing(*num, QUAT_SCALAR_Q_POINT)
                                } else if index == 4 {
                                    debug!(
                                        "Quaternion processing accuracy: {}",
                                        q_point_processing(*num, 12)
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    self.geomag_quaternions.0 = status;
                    self.geomag_quaternions.1 = quat_vals[0];
                    self.geomag_quaternions.2 = quat_vals[1];
                    self.geomag_quaternions.3 = quat_vals[2];
                    self.geomag_quaternions.4 = quat_vals[3];
                }

                _ => {
                    debug!("Unimplemented")
                }
            }
        }
    }
}

pub fn parse_status(report_bytes: &[u8]) -> (Status, u16) {
    if report_bytes.len() >= 4 {
        let status_byte = report_bytes[2];
        // info!("STATUS BYTE: {:#X}", status_byte);
        let status;
        let delay_lower_byte = report_bytes[3];
        let delay_upper = status_byte >> 2;
        let delay = (delay_upper as u16) << 8 | delay_lower_byte as u16;
        match status_byte & 0b0000_0011 {
            0 => {
                status = Status::Unreliable;
            }
            1 => {
                status = Status::LowAccuracy;
            }
            2 => {
                status = Status::MediumAccuracy;
            }
            3 => {
                status = Status::HighAccuracy;
            }
            _ => {
                status = Status::Unknown;
            }
        }

        (status, delay)
    } else {
        (Status::Unknown, 0)
    }
}
