use nalgebra as na;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JointState {
    pub names: Vec<String>,
    pub positions: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct BasePose {
    pub position: [f64; 3],
    pub quaternion: [f64; 4],
}

impl From<na::Isometry2<f64>> for BasePose {
    fn from(nav_pose: na::Isometry2<f64>) -> Self {
        let mut position = [0.0; 3];
        position[0] = nav_pose.translation.x;
        position[1] = nav_pose.translation.y;
        let quaternion = quaternion_from_euler_angles(0.0, 0.0, nav_pose.rotation.angle());
        Self {
            position,
            quaternion,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RpcResult {
    pub is_ok: bool,
    pub reason: String,
}

pub(crate) fn get_joint_positions(base_url: &Url) -> Result<JointState, ureq::Error> {
    let re = ureq::get(base_url.join("get_joint_positions").unwrap().as_str())
        .call()?
        .into_json::<JointState>()?;
    Ok(re)
}

pub(crate) fn send_joint_positions(
    base_url: &Url,
    joint_state: JointState,
) -> Result<RpcResult, ureq::Error> {
    let re = ureq::post(base_url.join("set_joint_positions").unwrap().as_str())
        .send_json(serde_json::to_value(joint_state).unwrap())?
        .into_json()?;
    Ok(re)
}

pub(crate) fn get_robot_origin(base_url: &Url) -> Result<BasePose, ureq::Error> {
    let re = ureq::get(base_url.join("get_robot_origin").unwrap().as_str())
        .call()?
        .into_json::<BasePose>()?;
    Ok(re)
}

pub(crate) fn send_robot_origin(
    base_url: &Url,
    base_pose: BasePose,
) -> Result<RpcResult, ureq::Error> {
    let re = ureq::post(base_url.join("set_robot_origin").unwrap().as_str())
        .send_json(serde_json::to_value(base_pose).unwrap())?
        .into_json()?;
    Ok(re)
}

pub(crate) fn euler_angles_from_quaternion(q: &[f64; 4]) -> (f64, f64, f64) {
    to_nalgebra(q).euler_angles()
}

pub(crate) fn quaternion_from_euler_angles(r: f64, p: f64, y: f64) -> [f64; 4] {
    let na_q = nalgebra::UnitQuaternion::from_euler_angles(r, p, y);
    from_nalgebra(&na_q)
}

fn from_nalgebra(na_q: &nalgebra::UnitQuaternion<f64>) -> [f64; 4] {
    let mut q = [0.0; 4];
    q[0] = na_q.w;
    q[1] = na_q.i;
    q[2] = na_q.j;
    q[3] = na_q.k;
    q
}

fn to_nalgebra(q: &[f64; 4]) -> nalgebra::UnitQuaternion<f64> {
    nalgebra::UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(q[0], q[1], q[2], q[3]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    #[test]
    fn test_euler_quaternion() {
        const R: f64 = 0.5;
        const P: f64 = -0.2;
        const Y: f64 = 1.0;
        let q = quaternion_from_euler_angles(R, P, Y);
        let angles = euler_angles_from_quaternion(&q);
        assert_approx_eq!(angles.0, R);
        assert_approx_eq!(angles.1, P);
        assert_approx_eq!(angles.2, Y);
        let q2 = quaternion_from_euler_angles(angles.0, angles.1, angles.2);
        assert_approx_eq!(q[0], q2[0]);
        assert_approx_eq!(q[1], q2[1]);
        assert_approx_eq!(q[2], q2[2]);
        assert_approx_eq!(q[3], q2[3]);
    }
}
