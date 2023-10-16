import gym

import numpy as np
import torch
import json
from cannoli_streaming_client import CannoliStreamingClient


class CannoliEnv(gym.Env):
    def render(self, mode="human"):
        pass

    def __init__(self, executable_identifier, max_steps_episode: int, action_space: np.array):
        super(CannoliEnv, self).__init__()
        self.max_steps_episode = max_steps_episode
        self.executable_identifier = executable_identifier
        self.client: CannoliStreamingClient = CannoliStreamingClient(executable_identifier)
        self.last_state = None

        # TODO: once checking against a specification better to keep this as an AST
        self.observation_space = [] * max_steps_episode
        self.observation_shape = (2, max_steps_episode)
        self.action_space = action_space
        self.episode_reward = 1.0
        self.steps_left = self.max_steps_episode

    def distance(self, oracle_next_state, next_state):
        return

    def _atomic_transaction(self, action):
        action = input()
        self.client.try_write(action)
        return self.client.try_read()

    def step(self, action):
        episode_terminated = False
        response = self._atomic_transaction(action)
        # if self.steps_left == self.max_steps_episode:
        #     print(f"Response {response}")
        # TODO: extrinsic reward is just amount of memory used multiplied by entropy
        global_heap_info = response['heap']['meta']

        bounds = global_heap_info['bounds']

        entropy, memory = global_heap_info['entropy'], bounds['end'] - bounds['start']
        next_state = torch.tensor([entropy, memory])
        self.episode_reward += entropy * memory
        self.steps_left -= 1
        if not self.steps_left:
            episode_terminated = True

        self.last_state = next_state
        self.observation_space.append(next_state)

        return next_state, self.episode_reward, episode_terminated, []



    def reset(self):
        self.client.reset()
        self.client = CannoliStreamingClient(self.executable_identifier)
        self.observation_space = []
        self.episode_reward = 0
        self.last_state = None
        self.steps_left = self.max_steps_episode

        return torch.tensor([0.0, 0])

