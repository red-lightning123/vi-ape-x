use crate::MasterMessage;
use crossbeam_channel::{Receiver, SendError, Sender};
use std::thread::JoinHandle;

pub trait ThreadType {
    type Message;
    type SpawnArgs;
    fn spawn(receiver: Receiver<Self::Message>, args: Self::SpawnArgs) -> JoinHandle<()>;
    fn master_message(msg: MasterMessage) -> Self::Message;
}

pub struct Thread<T>
where
    T: ThreadType,
{
    sender: Sender<T::Message>,
    receiver: Receiver<T::Message>,
}

impl<T> Thread<T>
where
    T: ThreadType,
{
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> &Sender<T::Message> {
        &self.sender
    }

    pub fn spawn(self, args: T::SpawnArgs) -> ActiveThread<T> {
        let handle = T::spawn(self.receiver, args);
        ActiveThread {
            sender: self.sender,
            handle,
        }
    }
}

pub struct ActiveThread<T>
where
    T: ThreadType,
{
    sender: Sender<T::Message>,
    handle: JoinHandle<()>,
}

impl<T> ActiveThread<T>
where
    T: ThreadType,
{
    pub fn sender(&self) -> &Sender<T::Message> {
        &self.sender
    }

    pub fn send(&self, msg: T::Message) -> Result<(), SendError<T::Message>> {
        self.sender.send(msg)
    }

    pub fn send_master(&self, msg: MasterMessage) -> Result<(), SendError<T::Message>> {
        self.sender.send(T::master_message(msg))
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.handle.join()
    }
}
