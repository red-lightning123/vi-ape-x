mod learner_client;

use crossbeam_channel::Receiver;
use learner_client::LearnerClient;
use model::traits::ParamFetcher;
use model::BasicModel;
use packets::ActorSettings;
use replay_wrappers::RemoteReplayWrapper;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

pub enum ParamUpdaterThreadMessage {
    UpdateParams,
    Stop,
}

pub fn spawn_param_updater_thread(
    receiver: Receiver<ParamUpdaterThreadMessage>,
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
    settings: &ActorSettings,
) -> JoinHandle<()> {
    let learner_client = settings.learner_addr.map(LearnerClient::new);
    std::thread::spawn(move || loop {
        match receiver.recv().unwrap() {
            ParamUpdaterThreadMessage::UpdateParams => {
                if let Some(ref learner_client) = learner_client {
                    let params = learner_client.get_params();
                    let mut agent = agent.write().unwrap();
                    agent.set_params(params);
                }
            }
            ParamUpdaterThreadMessage::Stop => break,
        }
    })
}
