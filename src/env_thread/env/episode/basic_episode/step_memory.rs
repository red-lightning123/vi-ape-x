use super::{State, Transition};
use std::collections::VecDeque;

pub struct StepMemory {
    step_queue: VecDeque<Step>,
    n: usize,
    gamma: f64,
}

impl StepMemory {
    pub fn new(n: usize, gamma: f64) -> Self {
        Self {
            step_queue: VecDeque::new(),
            n,
            gamma,
        }
    }
    pub fn reset_to_current(&mut self) {
        *self = Self::new(self.n, self.gamma);
    }
    pub fn push(
        &mut self,
        state: State,
        score: u32,
        action: u8,
        reward: f64,
    ) -> Option<(Transition, Option<u32>)> {
        let transition = if self.step_queue.len() >= self.n {
            let (step, total_reward) = self.pop_front_step().unwrap();
            let transition = Transition {
                state: step.state,
                next_state: state.clone(),
                action: step.action,
                reward: total_reward,
                terminated: false,
            };
            Some((transition, None))
        } else {
            None
        };
        self.push_back_step(Step {
            state,
            score,
            action,
            reward,
        });
        transition
    }
    pub fn pop_terminated_transitions_into(
        &mut self,
        transition_queue: &mut VecDeque<(Transition, Option<u32>)>,
    ) {
        while let Some((step, total_reward)) = self.pop_front_step() {
            let next_state = step.state.clone();
            let transition = Transition {
                state: step.state,
                next_state,
                action: step.action,
                reward: total_reward,
                terminated: true,
            };
            let is_final = self.is_empty();
            let episode_score = if is_final { Some(step.score) } else { None };
            transition_queue.push_back((transition, episode_score));
        }
    }
    fn push_back_step(&mut self, step: Step) {
        self.step_queue.push_back(step);
    }
    fn pop_front_step(&mut self) -> Option<(Step, f64)> {
        let total_reward = self.discounted_reward_sum();
        self.step_queue.pop_front().map(|step| (step, total_reward))
    }
    fn is_empty(&self) -> bool {
        self.step_queue.is_empty()
    }
    fn discounted_reward_sum(&self) -> f64 {
        self.step_queue
            .iter()
            .map(|step| step.reward)
            .rev()
            .fold(0.0, |acc, reward| self.gamma.mul_add(acc, reward))
    }
}

struct Step {
    state: State,
    score: u32,
    action: u8,
    reward: f64,
}
