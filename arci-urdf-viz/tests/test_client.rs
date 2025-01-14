mod web_server;

use assert_approx_eq::assert_approx_eq;
use std::sync::{Arc, Mutex};
use url::Url;

use arci::{JointTrajectoryClient, SetCompleteCondition, TotalJointDiffCondition, TrajectoryPoint};
use arci_urdf_viz::{UrdfVizWebClient, UrdfVizWebClientConfig};
use web_server::*;

#[test]
fn test_urdf_viz_web_client_config_accessor() {
    let mut config = UrdfVizWebClientConfig {
        name: "test".to_owned(),
        joint_names: vec!["j1".to_owned(), "j2".to_owned()],
        wrap_with_joint_velocity_limiter: true,
        joint_velocity_limits: vec![1.0, 2.0],
    };
    assert_eq!(config.name, "test");
    assert_eq!(config.joint_names[0], "j1");
    assert_eq!(config.joint_names[1], "j2");
    assert_eq!(config.wrap_with_joint_velocity_limiter, true);
    assert_approx_eq!(config.joint_velocity_limits[0], 1.0);
    assert_approx_eq!(config.joint_velocity_limits[1], 2.0);
    config.name = "arm".to_owned();
    config.joint_names = vec!["shoulder_pan_joint".to_owned(), "elbow_joint".to_owned()];
    config.wrap_with_joint_velocity_limiter = false;
    config.joint_velocity_limits = vec![0.0, 0.0];
    assert_eq!(config.name, "arm");
    assert_eq!(config.joint_names[0], "shoulder_pan_joint");
    assert_eq!(config.joint_names[1], "elbow_joint");
    assert_eq!(config.wrap_with_joint_velocity_limiter, false);
    assert_approx_eq!(config.joint_velocity_limits[0], 0.0);
    assert_approx_eq!(config.joint_velocity_limits[1], 0.0);
}

#[test]
fn test_urdf_viz_web_client_config_debug() {
    let config = UrdfVizWebClientConfig {
        name: "test".to_owned(),
        joint_names: vec!["j1".to_owned(), "j2".to_owned()],
        wrap_with_joint_velocity_limiter: true,
        joint_velocity_limits: vec![1.0, 2.0],
    };
    assert_eq!(
        format!("{:?}", config),
        "UrdfVizWebClientConfig { name: \"test\", joint_names: [\"j1\", \"j2\"], wrap_with_joint_velocity_limiter: true, joint_velocity_limits: [1.0, 2.0] }"
    )
}

#[test]
fn test_urdf_viz_web_client_config_clone() {
    let config1 = UrdfVizWebClientConfig {
        name: "test".to_owned(),
        joint_names: vec!["j1".to_owned(), "j2".to_owned()],
        wrap_with_joint_velocity_limiter: true,
        joint_velocity_limits: vec![1.0, 2.0],
    };
    let config2 = config1.clone();
    assert_eq!(config2.name, "test");
    assert_eq!(config2.joint_names[0], "j1");
    assert_eq!(config2.joint_names[1], "j2");
    assert_eq!(config2.wrap_with_joint_velocity_limiter, true);
    assert_approx_eq!(config2.joint_velocity_limits[0], 1.0);
    assert_approx_eq!(config2.joint_velocity_limits[1], 2.0);
}

#[test]
fn test_create_joint_trajectory_clients() {
    const DEFAULT_PORT: u16 = 7777;
    let web_server = WebServer::new(DEFAULT_PORT);
    std::thread::spawn(move || web_server.start());
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for web server to start.
    let configs = vec![
        UrdfVizWebClientConfig {
            name: "c1".to_owned(),
            joint_names: vec!["j1".to_owned(), "j2".to_owned()],
            wrap_with_joint_velocity_limiter: true,
            joint_velocity_limits: vec![1.0, 1.0],
        },
        UrdfVizWebClientConfig {
            name: "c2".to_owned(),
            joint_names: vec!["j1".to_owned(), "j2".to_owned()],
            wrap_with_joint_velocity_limiter: false,
            joint_velocity_limits: vec![],
        },
    ];
    let _clients = arci_urdf_viz::create_joint_trajectory_clients(configs, 0.1, 0.1);
}

#[test]
fn test_current_joint_positions() {
    const PORT: u16 = 7778;
    let mut web_server = WebServer::new(PORT);
    web_server.current_joint_positions = Arc::new(Mutex::new(JointNamesAndPositions {
        names: vec!["j1".to_owned(), "j2".to_owned()],
        positions: vec![1.0, -1.0],
    }));
    std::thread::spawn(move || web_server.start());
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for web server to start.
    let c = UrdfVizWebClient::try_new(Url::parse(&format!("http://127.0.0.1:{}", PORT)).unwrap())
        .unwrap();
    let v = c.current_joint_positions().unwrap();
    assert_approx_eq!(v[0], 1.0);
    assert_approx_eq!(v[1], -1.0);
}

#[test]
fn test_set_complete_condition() {
    const PORT: u16 = 7779;
    let web_server = WebServer::new(PORT);
    std::thread::spawn(move || web_server.start());
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for web server to start.
    let mut client =
        UrdfVizWebClient::try_new(Url::parse(&format!("http://127.0.0.1:{}", PORT)).unwrap())
            .unwrap();
    let cond = TotalJointDiffCondition::new(0.0, 0.1);
    client.set_complete_condition(Box::new(cond));
}

#[tokio::test]
async fn test_send_joint_positions() {
    const PORT: u16 = 7780;
    let mut web_server = WebServer::new(PORT);
    web_server.current_joint_positions = Arc::new(Mutex::new(JointNamesAndPositions {
        names: vec!["j1".to_owned()],
        positions: vec![0.0],
    }));
    std::thread::spawn(move || web_server.start());
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for web server to start.
    let mut client =
        UrdfVizWebClient::try_new(Url::parse(&format!("http://127.0.0.1:{}", PORT)).unwrap())
            .unwrap();
    client.run_send_joint_positions_thread();
    let result = client
        .send_joint_positions(vec![0.0], std::time::Duration::from_secs(1))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_joint_trajectory() {
    const PORT: u16 = 7781;
    let mut web_server = WebServer::new(PORT);
    web_server.current_joint_positions = Arc::new(Mutex::new(JointNamesAndPositions {
        names: vec!["j1".to_owned()],
        positions: vec![0.0],
    }));
    std::thread::spawn(move || web_server.start());
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for web server to start.
    let mut client =
        UrdfVizWebClient::try_new(Url::parse(&format!("http://127.0.0.1:{}", PORT)).unwrap())
            .unwrap();
    let trajectory = vec![
        TrajectoryPoint::new(vec![0.0], std::time::Duration::from_millis(100)),
        TrajectoryPoint::new(vec![0.0], std::time::Duration::from_millis(200)),
    ];
    client.run_send_joint_positions_thread();
    let result = client.send_joint_trajectory(trajectory).await;
    assert!(result.is_ok());
}
