mod tensorflow_fn;

use tensorflow::{Graph, SavedModelBundle};
use tensorflow_fn::TensorflowFn;

pub struct ModelFns {
    pub best_action: TensorflowFn<1, 1>,
    pub train_batch: TensorflowFn<5, 2>,
    pub train_batch_prioritized: TensorflowFn<9, 3>,
    pub compute_abs_td_errors: TensorflowFn<5, 1>,
    pub copy_control_to_target: TensorflowFn<0, 1>,
    pub save: TensorflowFn<1, 1>,
    pub load: TensorflowFn<1, 1>,
    pub params: TensorflowFn<0, 1>,
    pub set_params: TensorflowFn<1, 1>,
}

impl ModelFns {
    pub fn new(model_bundle: &SavedModelBundle, graph: &Graph) -> Self {
        Self {
            best_action: TensorflowFn::new(
                model_bundle,
                graph,
                "best_action",
                ["state"],
                [("output_0", 0)],
            ),
            train_batch: TensorflowFn::new(
                model_bundle,
                graph,
                "train_pred_step",
                ["states", "new_states", "actions", "rewards", "dones"],
                [("output_0", 0), ("output_1", 1)],
            ),
            train_batch_prioritized: TensorflowFn::new(
                model_bundle,
                graph,
                "train_pred_step_prioritized",
                [
                    "states",
                    "new_states",
                    "actions",
                    "rewards",
                    "dones",
                    "probabilities",
                    "min_probability",
                    "replay_memory_len",
                    "beta",
                ],
                [("output_0", 0), ("output_1", 1), ("output_2", 2)],
            ),
            compute_abs_td_errors: TensorflowFn::new(
                model_bundle,
                graph,
                "compute_abs_td_errors",
                ["states", "new_states", "actions", "rewards", "dones"],
                [("output_0", 0)],
            ),
            copy_control_to_target: TensorflowFn::new(
                model_bundle,
                graph,
                "copy_control_to_target",
                [],
                [("output_0", 0)],
            ),
            save: TensorflowFn::new(model_bundle, graph, "save", ["path"], [("output_0", 0)]),
            load: TensorflowFn::new(model_bundle, graph, "load", ["path"], [("output_0", 0)]),
            params: TensorflowFn::new(model_bundle, graph, "get_params", [], [("output_0", 0)]),
            set_params: TensorflowFn::new(
                model_bundle,
                graph,
                "set_params",
                ["params"],
                [("output_0", 0)],
            ),
        }
    }
}
