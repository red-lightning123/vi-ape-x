import tensorrt
import tensorflow as tf
from tensorflow import keras

def transition_inputs():
    return keras.layers.Input(shape=(8, 72, 128))

def preprocessed_inputs(inputs):
    normalized = keras.layers.Lambda(lambda x : x / 255.0)(inputs)
    transpose = keras.layers.Permute((2, 3, 1))(normalized)
    return transpose

def nn2013_conv_features(inputs):
    conv_1 = keras.layers.Conv2D(16, 8, strides=4, activation="relu")(inputs)
    conv_2 = keras.layers.Conv2D(32, 4, strides=2, activation="relu")(conv_1)
    flattened = keras.layers.Flatten()(conv_2)
    return flattened

def nn2015_conv_features(inputs):
    conv_1 = keras.layers.Conv2D(32, 8, strides=4, activation="relu")(inputs)
    conv_2 = keras.layers.Conv2D(64, 4, strides=2, activation="relu")(conv_1)
    conv_3 = keras.layers.Conv2D(64, 3, strides=1, activation="relu")(conv_2)
    flattened = keras.layers.Flatten()(conv_3)
    return flattened

def nn2013_q_stream(conv_features, n_actions):
    hidden = keras.layers.Dense(256, activation="relu")(conv_features)
    q_values = keras.layers.Dense(n_actions)(hidden)
    return q_values

def nn2015_q_stream(conv_features, n_actions):
    hidden = keras.layers.Dense(512, activation="relu")(conv_features)
    q_values = keras.layers.Dense(n_actions)(hidden)
    return q_values

def combined_dueling_streams(value, advantages):
    advantage_avg = tf.math.reduce_mean(advantages, axis=1)
    shifted_advantages = advantages - tf.expand_dims(advantage_avg, axis=1)
    q_values = value + shifted_advantages
    return q_values

def dueling_q_stream(conv_features, n_actions):
    hidden_value = keras.layers.Dense(512, activation="relu")(conv_features)
    hidden_advantages = keras.layers.Dense(512, activation="relu")(conv_features)
    value = keras.layers.Dense(1)(hidden_value)
    advantages = keras.layers.Dense(n_actions)(hidden_advantages)
    q_values = combined_dueling_streams(value, advantages)
    return q_values

GRAD_NORM_CLIPPING = 10

# Whether to rescale the gradients by 1 / sqrt(2), as suggested in the
# dueling dqn paper. Disabled by default.
# NOTE: Most implementations of dueling dqn seem to ignore this step,
# which makes it hard to verify how it should be done. Therefore, the
# interpretation of "rescale" chosen here is not necessarily correct.
DUELING_GRAD_WEIGHTING = False

LEARNING_RATE = 0.000025
N_STEPS = 1
GAMMA = 0.99

JUMP = False

if JUMP:
    n_actions = 3
else:
    n_actions = 2

class Model(tf.Module):
    def __init__(self):
        self.n_actions = n_actions
        inputs = transition_inputs()
        preprocessed = preprocessed_inputs(inputs)
        conv_features = nn2015_conv_features(preprocessed)
        outputs = dueling_q_stream(conv_features, self.n_actions)
        learning_rate = LEARNING_RATE
        self.optimizer = keras.optimizers.Adam(learning_rate=learning_rate, jit_compile=False, clipnorm=GRAD_NORM_CLIPPING)
        self.loss = keras.losses.Huber()
        self.model = keras.Model(inputs=inputs, outputs=outputs)
    @tf.function
    def __call__(self, x):
        return self.model(x, training=False)
    @tf.function
    def train_on_batch(self, states, updated_qvals, actions):
        action_masks = tf.one_hot(actions, self.n_actions)
        with tf.GradientTape() as tape:
            predicted_qvals = self.model(states, training=True)
            relevant_qvals = tf.reduce_sum(tf.multiply(predicted_qvals, action_masks), axis=1)
            loss_value = self.loss(updated_qvals, relevant_qvals)
        grads = tape.gradient(loss_value, self.model.trainable_weights)
        if DUELING_GRAD_WEIGHTING:
            grads = grads / sqrt(2)
        self.optimizer.apply_gradients(zip(grads, self.model.trainable_weights))
        return loss_value
    @tf.function
    def train_on_batch_prioritized(self, states, updated_qvals, actions, probabilities, min_probability, replay_memory_len, beta):
        importance_sampling_weights = (replay_memory_len * probabilities)**(-beta)
        # assumes beta >= 0. the smallest probability corresponds to the largest sampling weight
        max_importance_sampling_weight = (replay_memory_len * min_probability)**(-beta)
        normalized_importance_sampling_weights = importance_sampling_weights / max_importance_sampling_weight
        action_masks = tf.one_hot(actions, self.n_actions)
        with tf.GradientTape() as tape:
            predicted_qvals = self.model(states, training=True)
            relevant_qvals = tf.reduce_sum(tf.multiply(predicted_qvals, action_masks), axis=1)
            td_errors = updated_qvals - relevant_qvals
            td_errors_expanded = tf.expand_dims(td_errors, axis=-1)
            loss_value = self.loss(td_errors_expanded, 0, sample_weight = normalized_importance_sampling_weights)
        grads = tape.gradient(loss_value, self.model.trainable_weights)
        if DUELING_GRAD_WEIGHTING:
            grads = grads / sqrt(2)
        self.optimizer.apply_gradients(zip(grads, self.model.trainable_weights))
        return loss_value, tf.abs(td_errors)
    @tf.function
    def save(self, path):
        for variable in self.model.variables:
            tf.io.write_file(tf.strings.join([path, "/model/", variable.name]), tf.io.serialize_tensor(variable))
        for variable in self.optimizer.variables:
            tf.io.write_file(tf.strings.join([path, "/optimizer/", variable.name]), tf.io.serialize_tensor(variable))
    @tf.function
    def load(self, path):
        for variable in self.model.variables:
            variable.assign(tf.io.parse_tensor(tf.io.read_file(tf.strings.join([path, "/model/", variable.name])), out_type=variable.dtype))
        for variable in self.optimizer.variables:
            variable.assign(tf.io.parse_tensor(tf.io.read_file(tf.strings.join([path, "/optimizer/", variable.name])), out_type=variable.dtype))

class Agent(tf.Module):
    def __init__(self):
        self.control_model = Model()
        self.target_model = Model()
        self.copy_control_to_target()
    @tf.function
    def predict_qvals(self, state):
        state = tf.expand_dims(state, 0)
        qvals = self.control_model(state)[0]
        
        return qvals
    @tf.function
    def best_action(self, state):
        return tf.argmax(self.predict_qvals(state))
    @tf.function
    def train_pred_step(self, states, new_states, actions, rewards, dones):
        gamma_pow_n = GAMMA**N_STEPS
        control_new_state_qvals = self.control_model(new_states)
        target_new_state_qvals = self.target_model(new_states)
        next_actions = tf.argmax(control_new_state_qvals, axis=1)
        updated_qvals = rewards + (1 - dones) * gamma_pow_n * tf.gather(target_new_state_qvals, next_actions, batch_dims=1)

        loss = self.control_model.train_on_batch(
            states,
            updated_qvals,
            actions
        )
        avg_target_new_state_qval = tf.math.reduce_mean(target_new_state_qvals)
        return loss, avg_target_new_state_qval
    @tf.function
    def train_pred_step_prioritized(self, states, new_states, actions, rewards, dones, probabilities, min_probability, replay_memory_len, beta):
        gamma_pow_n = GAMMA**N_STEPS
        control_new_state_qvals = self.control_model(new_states)
        target_new_state_qvals = self.target_model(new_states)
        next_actions = tf.argmax(control_new_state_qvals, axis=1)
        updated_qvals = rewards + (1 - dones) * gamma_pow_n * tf.gather(target_new_state_qvals, next_actions, batch_dims=1)

        loss, abs_td_errors = self.control_model.train_on_batch_prioritized(
            states,
            updated_qvals,
            actions,
            probabilities,
            min_probability,
            replay_memory_len,
            beta
        )
        avg_target_new_state_qval = tf.math.reduce_mean(target_new_state_qvals)
        return loss, avg_target_new_state_qval, abs_td_errors
    @tf.function
    def copy_control_to_target(self):
        '''
        copying only trainable weights means non-trainable variables like the
        optimizer state won't be copied, but this isn't a problem since the target
        model is never actually trained directly
        '''
        for i in range(0, len(self.target_model.model.trainable_weights)):
            self.target_model.model.trainable_weights[i].assign(self.control_model.model.trainable_weights[i])
        return 0 # tf function having a signature must return something
    @tf.function
    def save(self, path):
        self.control_model.save(tf.strings.join([path, "/control"]))
        self.target_model.save(tf.strings.join([path, "/target"]))
        return 0 # tf function having a signature must return something
    @tf.function
    def load(self, path):
        self.control_model.load(tf.strings.join([path, "/control"]))
        self.target_model.load(tf.strings.join([path, "/target"]))
        return 0 # tf function having a signature must return something
        
agent = Agent()

single_state = tf.zeros([8, 72, 128], dtype=tf.uint8)
states = tf.zeros([32, 8, 72, 128], dtype=tf.uint8)
next_states = tf.zeros([32, 8, 72, 128], dtype=tf.uint8)
actions = tf.zeros([32], dtype=tf.uint8)
rewards = tf.zeros([32])
dones = tf.zeros([32], dtype=tf.float32)
probabilities = tf.zeros([32], dtype=tf.float32)
min_probability = tf.zeros([], dtype=tf.float32)
replay_memory_len = tf.zeros([], dtype=tf.float32)
beta = tf.zeros([], dtype=tf.float32)
path = tf.constant("path")

best_action = agent.best_action.get_concrete_function(single_state)
train_pred_step = agent.train_pred_step.get_concrete_function(states, next_states, actions, rewards, dones)
train_pred_step_prioritized = agent.train_pred_step_prioritized.get_concrete_function(states, next_states, actions, rewards, dones, probabilities, min_probability, replay_memory_len, beta)
copy_control_to_target = agent.copy_control_to_target.get_concrete_function()
save = agent.save.get_concrete_function(path)
load = agent.load.get_concrete_function(path)

signatures = {
    "best_action": best_action,
    "train_pred_step": train_pred_step,
    "train_pred_step_prioritized": train_pred_step_prioritized,
    "copy_control_to_target": copy_control_to_target,
    "save": save,
    "load": load
}

tf.saved_model.save(agent, export_dir="model", signatures=signatures)
