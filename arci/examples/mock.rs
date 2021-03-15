use arci::*;
use futures::{
    future::FutureExt,
    stream::{FuturesUnordered, StreamExt},
};
use indexmap::IndexMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use Command::*;

#[derive(Debug, Clone, PartialEq)]
enum Command {
    SpeakStart {
        name: String,
        message: String,
    },
    SpeakEnd {
        name: String,
    },
    SendPositionsStart {
        name: String,
        positions: Vec<f64>,
        duration: Duration,
    },
    SendPositionsEnd {
        name: String,
    },
}

struct RobotClient {
    speakers: IndexMap<String, Arc<DummySpeaker>>,
    joint_trajectory_clients: IndexMap<String, Arc<DummyJointTrajectoryClient>>,
    commands: Arc<Mutex<Vec<Command>>>,
}

impl RobotClient {
    fn new(
        mut speakers: IndexMap<String, Arc<DummySpeaker>>,
        mut joint_trajectory_clients: IndexMap<String, Arc<DummyJointTrajectoryClient>>,
    ) -> Self {
        let commands = Arc::new(Mutex::new(vec![]));
        for speaker in speakers.values_mut() {
            *speaker.commands.write().unwrap() = Some(commands.clone());
        }
        for joint_trajectory_client in joint_trajectory_clients.values_mut() {
            *joint_trajectory_client.commands.write().unwrap() = Some(commands.clone());
        }
        Self {
            speakers,
            joint_trajectory_clients,
            commands,
        }
    }
}

struct DummySpeaker {
    name: String,
    commands: RwLock<Option<Arc<Mutex<Vec<Command>>>>>,
}

impl DummySpeaker {
    fn new(name: String) -> Arc<Self> {
        Arc::new(Self {
            name,
            commands: RwLock::new(None),
        })
    }
}

impl Speaker for DummySpeaker {
    fn speak(&self, message: &str) {
        let commands = self.commands.read().unwrap().clone().unwrap();
        commands.lock().unwrap().push(SpeakStart {
            name: self.name.clone(),
            message: message.to_owned(),
        });
        // speak is not async, so block current thread.
        thread::sleep(Duration::from_secs_f64(message.len() as f64 * 0.1));
        commands.lock().unwrap().push(SpeakEnd {
            name: self.name.clone(),
        });
    }
}

struct DummyJointTrajectoryClient {
    name: String,
    joint_names: Vec<String>,
    positions: Arc<Mutex<Vec<Vec<f64>>>>,
    complete_condition: Box<dyn CompleteCondition>,
    commands: RwLock<Option<Arc<Mutex<Vec<Command>>>>>,
}

impl DummyJointTrajectoryClient {
    fn new(name: String, joint_names: Vec<String>) -> Arc<Self> {
        let dof = joint_names.len();
        let positions = Arc::new(Mutex::new(vec![vec![0.0; dof]]));
        Arc::new(Self {
            name,
            joint_names,
            positions,
            complete_condition: Box::new(TotalJointDiffCondition::default()),
            commands: RwLock::new(None),
        })
    }
}

#[async_trait]
impl JointTrajectoryClient for DummyJointTrajectoryClient {
    fn joint_names(&self) -> &[String] {
        &self.joint_names
    }

    fn current_joint_positions(&self) -> Result<Vec<f64>, Error> {
        Ok(self.positions.lock().unwrap().last().unwrap().clone())
    }

    async fn send_joint_positions(
        &self,
        positions: Vec<f64>,
        duration: Duration,
    ) -> Result<(), Error> {
        let self_positions = self.positions.clone();
        let commands = self.commands.read().unwrap().clone().unwrap();
        let positions_clone = positions.clone();

        let command_s = SendPositionsStart {
            name: self.name.clone(),
            duration,
            positions: positions.clone(),
        };
        let command_e = SendPositionsEnd {
            name: self.name.clone(),
        };

        thread::spawn(move || {
            commands.lock().unwrap().push(command_s);
            thread::sleep(duration);
            commands.lock().unwrap().push(command_e);
            self_positions.lock().unwrap().push(positions_clone.clone());
        });

        self.complete_condition
            .wait(self, &positions, duration.as_secs_f64())
            .await?;
        Ok(())
    }

    async fn send_joint_trajectory(
        &self,
        _full_trajectory: Vec<TrajectoryPoint>,
    ) -> Result<(), Error> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let mut handles = FuturesUnordered::new();

    let joint_trajectory_clients = indexmap::indexmap! {
        "c1".to_owned() => DummyJointTrajectoryClient::new("c1".to_owned(), vec!["j1".to_owned(), "j2".to_owned()]),
        "c2".to_owned() => DummyJointTrajectoryClient::new("c2".to_owned(), vec!["j3".to_owned(), "j4".to_owned()]),
    };
    let speakers = indexmap::indexmap! {
        "s1".to_owned() => DummySpeaker::new("s1".to_owned()),
    };
    let robot_client = RobotClient::new(speakers, joint_trajectory_clients);

    let s1 = robot_client.speakers["s1"].clone();
    handles.push(
        async move {
            tokio::task::spawn_blocking(move || {
                // 1/2/3
                s1.speak("msg");
                // 6
            })
            .await
            .unwrap()
        }
        .boxed(),
    );

    let c1 = robot_client.joint_trajectory_clients["c1"].clone();
    let c2 = robot_client.joint_trajectory_clients["c2"].clone();
    handles.push(
        async move {
            // 1/2/3
            c1.send_joint_positions(vec![1.0, -10.0], Duration::from_millis(100))
                .await
                .unwrap();
            // 4
            c1.send_joint_positions(vec![3.0, -10.0], Duration::from_millis(300))
                .await
                .unwrap();
            // 7
        }
        .boxed(),
    );
    handles.push(
        async move {
            // 1/2/3
            c2.send_joint_positions(vec![2.0, -10.0], Duration::from_millis(200))
                .await
                .unwrap();
            // 5
            c2.send_joint_positions(vec![4.0, -10.0], Duration::from_millis(400))
                .await
                .unwrap();
            // 8
        }
        .boxed(),
    );

    // 0
    while handles.next().await.is_some() {}

    let result = &*robot_client.commands.lock().unwrap();
    dbg!(result);

    // There is no guarantee which will be started first.
    assert!(result[0..3].iter().all(|r| vec![
        SpeakStart {
            name: "s1".into(),
            message: "msg".into(),
        },
        SendPositionsStart {
            name: "c1".into(),
            positions: vec![1.0, -10.0],
            duration: Duration::from_millis(100)
        },
        SendPositionsStart {
            name: "c2".into(),
            positions: vec![2.0, -10.0],
            duration: Duration::from_millis(200)
        },
    ]
    .contains(r)));

    assert_eq!(
        result[3..],
        vec![
            SendPositionsEnd { name: "c1".into() },
            SendPositionsStart {
                name: "c1".into(),
                positions: vec![3.0, -10.0],
                duration: Duration::from_millis(300)
            },
            SendPositionsEnd { name: "c2".into() },
            SendPositionsStart {
                name: "c2".into(),
                positions: vec![4.0, -10.0],
                duration: Duration::from_millis(400)
            },
            SpeakEnd { name: "s1".into() },
            SendPositionsEnd { name: "c1".into() },
            SendPositionsEnd { name: "c2".into() },
        ]
    );
}
