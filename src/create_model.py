import tensorrt
import tensorflow as tf
from tensorflow import keras

def nn2013(n_actions):
    inputs = keras.layers.Input(shape=(8, 128, 72))
    normalized = keras.layers.Lambda(lambda x : x / 255.0)(inputs)
    transpose = keras.layers.Permute((2, 3, 1))(normalized)
    conv_1 = keras.layers.Conv2D(16, 8, strides=4, activation="relu")(transpose)
    conv_2 = keras.layers.Conv2D(32, 4, strides=2, activation="relu")(conv_1)
    flattened = keras.layers.Flatten()(conv_2)
    hidden = keras.layers.Dense(256, activation="relu")(flattened)
    outputs = keras.layers.Dense(n_actions)(hidden)
    return inputs, outputs

def nn2015(n_actions):
    inputs = keras.layers.Input(shape=(8, 128, 72))
    normalized = keras.layers.Lambda(lambda x : x / 255.0)(inputs)
    transpose = keras.layers.Permute((2, 3, 1))(normalized)
    conv_1 = keras.layers.Conv2D(32, 8, strides=4, activation="relu")(transpose)
    conv_2 = keras.layers.Conv2D(64, 4, strides=2, activation="relu")(conv_1)
    conv_3 = keras.layers.Conv2D(64, 3, strides=1, activation="relu")(conv_2)
    flattened = keras.layers.Flatten()(conv_3)
    hidden = keras.layers.Dense(512, activation="relu")(flattened)
    outputs = keras.layers.Dense(n_actions)(hidden)
    return inputs, outputs

nn = nn2015

LEARNING_RATE = 0.000025

JUMP = False

if JUMP:
    n_actions = 3
else:
    n_actions = 2

class Model(tf.Module):
    def __init__(self):
        self.n_actions = n_actions
        inputs, outputs = nn(self.n_actions)
        learning_rate = LEARNING_RATE
        self.optimizer = keras.optimizers.Adam(learning_rate=learning_rate, jit_compile=False)
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
        self.optimizer.apply_gradients(zip(grads, self.model.trainable_weights))
        return loss_value
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
        gamma = 0.99
        control_new_state_qvals = self.control_model(new_states)
        target_new_state_qvals = self.target_model(new_states)
        next_actions = tf.argmax(control_new_state_qvals, axis=1)
        updated_qvals = rewards + (1 - dones) * gamma * tf.gather(target_new_state_qvals, next_actions, batch_dims=1)

        loss = self.control_model.train_on_batch(
            states,
            updated_qvals,
            actions
        )
        return loss
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

single_state = tf.zeros([8, 128, 72], dtype=tf.uint8)
states = tf.zeros([32, 8, 128, 72], dtype=tf.uint8)
next_states = tf.zeros([32, 8, 128, 72], dtype=tf.uint8)
actions = tf.zeros([32], dtype=tf.uint8)
rewards = tf.zeros([32])
dones = tf.zeros([32], dtype=tf.float32)
path = tf.constant("path")

best_action = agent.best_action.get_concrete_function(single_state)
train_pred_step = agent.train_pred_step.get_concrete_function(states, next_states, actions, rewards, dones)
copy_control_to_target = agent.copy_control_to_target.get_concrete_function()
save = agent.save.get_concrete_function(path)
load = agent.load.get_concrete_function(path)

signatures = {
    "best_action": best_action,
    "train_pred_step": train_pred_step,
    "copy_control_to_target": copy_control_to_target,
    "save": save,
    "load": load
}

tf.saved_model.save(agent, export_dir="model", signatures=signatures)
