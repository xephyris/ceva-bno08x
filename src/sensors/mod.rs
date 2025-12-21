use defmt::*;

use crate::{
    config::QUAT_SCALAR_Q_POINT,
    parsing::{get_report_format, get_report_length, process_buf, q_point_processing},
    register::ReportId,
};

#[derive(Debug)]
pub struct Sensors {
    pub quaternions: (f32, f32, f32, f32),
}

impl Sensors {
    pub fn new() -> Sensors {
        Sensors {
            quaternions: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn update_data(&mut self, report_id: ReportId, buf_slice: &[u8]) {
        let data_types = get_report_format(report_id);
        if let Some((report_id, data_format)) = data_types {
            match report_id {
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
