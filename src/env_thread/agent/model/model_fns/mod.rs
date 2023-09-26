mod tensorflow_fn;
use tensorflow_fn::TensorflowFn;
use tensorflow::{ Graph, SavedModelBundle };

pub struct ModelFns {
    pub best_action : TensorflowFn<1, 1>,
    pub train_batch : TensorflowFn<5, 1>,
    pub train_batch_prioritized : TensorflowFn<9, 2>,
    pub copy_control_to_target : TensorflowFn<0, 1>,
    pub save : TensorflowFn<1, 1>,
    pub load : TensorflowFn<1, 1>
}

impl ModelFns {
    pub fn new(model_bundle : &SavedModelBundle, graph : &Graph) -> ModelFns {
        ModelFns {
            best_action : TensorflowFn::new(model_bundle, graph, "best_action", ["state"], [("output_0", 0)]),
            train_batch : TensorflowFn::new(model_bundle, graph, "train_pred_step", ["states", "new_states", "actions", "rewards", "dones"], [("output_0", 0)]),
            train_batch_prioritized : TensorflowFn::new(model_bundle, graph, "train_pred_step_prioritized", ["states", "new_states", "actions", "rewards", "dones", "probabilities", "min_probability", "replay_memory_len", "beta"], [("output_0", 0), ("output_1", 1)]),
            copy_control_to_target : TensorflowFn::new(model_bundle, graph, "copy_control_to_target", [], [("output_0", 0)]),
            save : TensorflowFn::new(model_bundle, graph, "save", ["path"], [("output_0", 0)]),
            load : TensorflowFn::new(model_bundle, graph, "load", ["path"], [("output_0", 0)])
        }
    }
}

