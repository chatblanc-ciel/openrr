use assert_approx_eq::assert_approx_eq;

use arci::{
    CompleteCondition, DummyJointTrajectoryClient, EachJointDiffCondition, JointTrajectoryClient,
    TotalJointDiffCondition,
};

#[test]
fn test_total_condition() {
    let client = DummyJointTrajectoryClient::new(vec!["j1".to_owned(), "j2".to_owned()]);
    let c1 = TotalJointDiffCondition::new(1.0, 0.1);
    assert!(c1.wait(&client, &[0.0, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[0.5, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[-0.5, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[-0.5, 0.8], 1.0).is_err());
    tokio_test::block_on(
        client.send_joint_positions(vec![3.0, -10.0], std::time::Duration::from_millis(100)),
    )
    .unwrap();
    assert!(c1.wait(&client, &[-0.5, 0.8], 1.0).is_err());
    assert!(c1.wait(&client, &[3.0, -10.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[3.0, -10.5], 1.0).is_ok());
}

#[test]
fn test_total_condition_accessor() {
    let mut c1 = TotalJointDiffCondition::new(1.0, 0.1);
    assert_approx_eq!(c1.allowable_error, 1.0);
    assert_approx_eq!(c1.timeout_sec, 0.1);
    c1.allowable_error = 0.1;
    c1.timeout_sec = 1.0;
    assert_approx_eq!(c1.allowable_error, 0.1);
    assert_approx_eq!(c1.timeout_sec, 1.0);
}

#[test]
fn test_total_condition_debug() {
    let c1 = TotalJointDiffCondition::new(1.0, 0.1);
    assert_eq!(
        format!("{:?}", c1),
        "TotalJointDiffCondition { allowable_error: 1.0, timeout_sec: 0.1 }"
    );
}

#[test]
fn test_total_condition_clone() {
    let c1 = TotalJointDiffCondition::new(1.0, 0.1);
    let c2 = c1.clone();
    assert_approx_eq!(c2.allowable_error, 1.0);
    assert_approx_eq!(c2.timeout_sec, 0.1);
    assert_approx_eq!(c1.allowable_error, 1.0);
    assert_approx_eq!(c1.timeout_sec, 0.1);
}

#[test]
fn test_each_condition() {
    let client = DummyJointTrajectoryClient::new(vec!["j1".to_owned(), "j2".to_owned()]);
    let c1 = EachJointDiffCondition::new(vec![1.0, 0.1], 0.1);
    assert!(c1.wait(&client, &[0.0, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[0.5, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[-0.5, 0.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[-1.5, 0.0], 1.0).is_err());
    assert!(c1.wait(&client, &[-0.5, 0.2], 1.0).is_err());
    tokio_test::block_on(
        client.send_joint_positions(vec![3.0, -10.0], std::time::Duration::from_millis(100)),
    )
    .unwrap();
    assert!(c1.wait(&client, &[3.0, 0.8], 1.0).is_err());
    assert!(c1.wait(&client, &[3.0, -9.95], 1.0).is_ok());
    assert!(c1.wait(&client, &[3.0, -10.0], 1.0).is_ok());
    assert!(c1.wait(&client, &[3.5, -10.0], 1.0).is_ok());
}

#[test]
fn test_each_condition_err() {
    let client = DummyJointTrajectoryClient::new(vec!["j1".to_owned(), "j2".to_owned()]);
    let c1 = EachJointDiffCondition::new(vec![1.0, 0.1], 0.1);
    assert!(c1.wait(&client, &[0.0, 0.0, 0.0], 1.0).is_err());
    assert!(c1.wait(&client, &[0.0], 1.0).is_err());
    assert!(c1.wait(&client, &[], 1.0).is_err());
    tokio_test::block_on(
        client.send_joint_positions(vec![3.0, -10.0], std::time::Duration::from_millis(100)),
    )
    .unwrap();
    assert!(c1.wait(&client, &[3.5, -10.0, 0.0], 1.0).is_err());
    assert!(c1.wait(&client, &[3.0], 1.0).is_err());
    assert!(c1.wait(&client, &[], 1.0).is_err());
}

#[test]
fn test_each_condition_dof() {
    let client = DummyJointTrajectoryClient::new(vec!["j1".to_owned(), "j2".to_owned()]);
    let c0 = EachJointDiffCondition::new(vec![], 0.1);
    assert!(c0.wait(&client, &[], 1.0).is_ok());
    let c1 = EachJointDiffCondition::new(vec![0.1], 0.1);
    assert!(c1.wait(&client, &[0.0], 1.0).is_ok());
    let c2 = EachJointDiffCondition::new(vec![0.1, 0.1], 0.1);
    assert!(c2.wait(&client, &[0.0, 0.0], 1.0).is_ok());
}

#[test]
#[should_panic]
fn test_each_condition_dof_err() {
    let client = DummyJointTrajectoryClient::new(vec!["j1".to_owned()]);
    let c2 = EachJointDiffCondition::new(vec![0.1, 0.1], 0.1);
    assert!(c2.wait(&client, &[0.0, 0.0], 1.0).is_ok());
}

#[test]
fn test_each_condition_accessor() {
    let mut c1 = EachJointDiffCondition::new(vec![1.0, 0.1], 0.1);
    assert_approx_eq!(c1.allowable_errors[0], 1.0);
    assert_approx_eq!(c1.allowable_errors[1], 0.1);
    assert_approx_eq!(c1.timeout_sec, 0.1);
    c1.allowable_errors = vec![0.1, 1.0];
    c1.timeout_sec = 1.0;
    assert_approx_eq!(c1.allowable_errors[0], 0.1);
    assert_approx_eq!(c1.allowable_errors[1], 1.0);
    assert_approx_eq!(c1.timeout_sec, 1.0);
}

#[test]
fn test_each_condition_debug() {
    let c1 = EachJointDiffCondition::new(vec![1.0, 0.1], 0.1);
    assert_eq!(
        format!("{:?}", c1),
        "EachJointDiffCondition { allowable_errors: [1.0, 0.1], timeout_sec: 0.1 }"
    );
}

#[test]
fn test_each_condition_clone() {
    let c1 = EachJointDiffCondition::new(vec![1.0, 0.1], 0.1);
    let c2 = c1.clone();
    assert_approx_eq!(c2.allowable_errors[0], 1.0);
    assert_approx_eq!(c2.allowable_errors[1], 0.1);
    assert_approx_eq!(c2.timeout_sec, 0.1);
    assert_approx_eq!(c1.allowable_errors[0], 1.0);
    assert_approx_eq!(c1.allowable_errors[1], 0.1);
    assert_approx_eq!(c1.timeout_sec, 0.1);
}
