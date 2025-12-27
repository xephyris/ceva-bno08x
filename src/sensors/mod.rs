use defmt::*;

use crate::{
    config::*,
    parsing::{get_report_format, process_buf, q_point_processing},
    register::ReportId,
};

#[derive(Debug)]
pub struct Sensors {
    pub quaternions: (f32, f32, f32, f32),
    pub acceleration: (f32, f32, f32), // Calibrated
    pub linear_accel: (f32, f32, f32),
    pub gravity: (f32, f32, f32),
    pub gyroscope: (f32, f32, f32), // Calibrated
    pub gyro_raw: (u16, u16, u16, u16, u32),
    pub magnetometer: (f32, f32, f32), // Calibrated Mag Field
}

impl Sensors {
    pub fn new() -> Sensors {
        Sensors {
            quaternions: (0.0, 0.0, 0.0, 0.0),
            acceleration: (0.0, 0.0, 0.0),
            linear_accel: (0.0, 0.0, 0.0),
            gravity: (0.0, 0.0, 0.0),
            gyroscope: (0.0, 0.0, 0.0),
            gyro_raw: (0, 0, 0, 0, 0),
            magnetometer: (0.0, 0.0, 0.0),
        }
    }

    pub fn update_data(&mut self, report_id: ReportId, buf_slice: &[u8]) {
        let data_types = get_report_format(report_id);
        if let Some((report_id, data_format)) = data_types {
            match report_id {
                ReportId::AccelerometerCalibrated => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.acceleration.0 = accel_vals[0];
                    self.acceleration.1 = accel_vals[1];
                    self.acceleration.2 = accel_vals[2];
                }
                ReportId::AccelerationLinear => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.linear_accel.0 = linear[0];
                    self.linear_accel.1 = linear[1];
                    self.linear_accel.2 = linear[2];
                }
                ReportId::Gravity => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.gravity.0 = grav[0];
                    self.gravity.1 = grav[1];
                    self.gravity.2 = grav[2];
                }
                ReportId::GyroscopeCalibrated => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.gyroscope.0 = gyro_vals[0];
                    self.gyroscope.1 = gyro_vals[1];
                    self.gyroscope.2 = gyro_vals[2];
                }
                ReportId::GyroscopeRaw => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.gyro_raw.0 = gyro_vals[0] as u16;
                    self.gyro_raw.1 = gyro_vals[1] as u16;
                    self.gyro_raw.2 = gyro_vals[2] as u16;
                    self.gyro_raw.3 = gyro_vals[3] as u16;
                    self.gyro_raw.4 = gyro_vals[4];
                }
                ReportId::MagFieldCalibrated => {
                    let out = process_buf(data_format, buf_slice);
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
                    self.magnetometer.0 = mag_vals[0];
                    self.magnetometer.1 = mag_vals[1];
                    self.magnetometer.2 = mag_vals[2];
                }
                ReportId::RotationVector => {
                    let out = process_buf(data_format, buf_slice);
                    let mut quat_vals = [0.0_f32; 4];
                    for (index, data_val) in out.iter().enumerate() {
                        match data_val {
                            crate::parsing::DataVals::I16(num) => {
                                if index < 4 {
                                    quat_vals[index] = q_point_processing(*num, QUAT_SCALAR_Q_POINT)
                                }
                            }
                            crate::parsing::DataVals::U32(accuracy) => {
                                if index == 4 {
                                    debug!("Quaternion processing accuracy: {}", accuracy);
                                }
                            }
                            _ => {}
                        }
                    }
                    self.quaternions.0 = quat_vals[0];
                    self.quaternions.1 = quat_vals[1];
                    self.quaternions.2 = quat_vals[2];
                    self.quaternions.3 = quat_vals[3];
                }

                _ => {
                    debug!("Unimplemented")
                }
            }
        }
    }
}
