import gym

import numpy as np
import torch
from .cannoli_streaming_client import CannoliStreamingClient
import json
import sqlite3, datetime


class CannoliEnv(gym.Env):
    """
    So once bench.py spawns the processes and CannoliStreamingClient (NAV side of socket) connects with Middleware (tracer side of socket), I believe the following operations happen for a step.
    1. CannoliEnv steps with step https://github.com/narfindustries/leetasm-private/blob/low-level-integration/navigator/envs/cannoli_env.py#L34-L55
    2. Step calls _atomic_transaction  which calls CannoliStreamingClient.try_write() with a command for the tracer. Commands for this target should be in tuples like we previously discussed. This writes data over the socket to Middleware: https://github.com/narfindustries/leetasm-private/blob/low-level-integration/navigator/envs/cannoli_streaming_client.py#L65-L74. After calling try_write(), CannoliEnv calls CannoliStreamingClient.try_read() which will wait for feedback from the tracer
    3. Middleware receives the command and proxies it to the tracer: https://github.com/narfindustries/leetasm-uefi-testbench/blob/nav-integration/gdblib/middleware.py#L25-L30
    4. The tracer handles the command and hits callbacks. This is all in the guts of the tracer. eventually, it reaches the end of the command and serializes all the events into JSON. It passes that JSON to Middleware which packages it up and sends to CannoliStreamingClient: https://github.com/narfindustries/leetasm-uefi-testbench/blob/nav-integration/gdblib/middleware.py#L56-L61
    5. CannoliStreamingClient receives and deserializes the events. It also performs invariant checking and memoization here. Right now that is stubbed out in a check function, which will later be replaced by Aaron's TLA+ code. At the end of the function, it should return whatever relevant data NAV needs. Right now this is nothing, I can change very easily once we know what NAV wants: https://github.com/narfindustries/leetasm-private/blob/low-level-integration/navigator/envs/cannoli_streaming_client.py#L98-L121
    6. This return ends atomic_transaction and CannoliEnv does the rest of the step code
    Rinse and repeat until end of episode (in which client.reset() is called)
    """

    def render(self, mode="human"):
        pass

    def __init__(self, executable_identifier, max_steps_episode: int, action_space: np.array, episodes: int,
                 epochs: int):
        super(CannoliEnv, self).__init__()
        self.max_steps_episode = max_steps_episode
        self.steps_left = max_steps_episode

        self.executable_identifier = executable_identifier
        self.client: CannoliStreamingClient = CannoliStreamingClient(executable_identifier)
        self.last_state = None

        # TODO: once checking against a specification better to keep this as an AST
        self.observation_shape = (max_steps_episode, 7)
        self.observation_space = torch.zeros(self.observation_shape)
        self.action_space = action_space
        self.episode_reward = 1.0

        self.action_sequence = []
        # THIS does not get reset
        self.memos = {}
        self.invariants_previously_seen_in_episode = 1

        self.n_episodes = episodes
        self.n_epochs = epochs

        self.epoch = 0
        self.episode = 0

        self.time = round(datetime.datetime.now().timestamp())
        self.test_name = f"test_{round(datetime.datetime.now().timestamp())}"

        self.db_events = list()
        self.con = sqlite3.connect(f"demo.db")
        self.db = self.con.cursor()

        self._create_table()

        # self.db.execute(f"INSERT INTO tests VALUES(?, ?)", (self.time, self.test_name))
        # self.db.execute(f"CREATE TABLE {self.test_name}(epoch, episode, test, data)")
        # self.con.commit()

    def _create_table(self):
        # Create the 'tests' table. Should only need to be done once per db
        # Future consideration: add distinguishing information to easily filter
        # "meaningful" tests, e.g,. max reward achieved for all epochs & episodes
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS tests (uuid, testname)"
        )

        # insert test metadata into `tests` table
        self.db.execute(f"INSERT INTO tests VALUES(?, ?)", (self.time, self.test_name))

        # Create the 'test_X' tables
        # Future consideration: add a column for `action` and `params` to easily
        # filter the table and replay interesting/failing tests
        self.db.execute(f'''
            CREATE TABLE IF NOT EXISTS {self.test_name} (
                epoch INTEGER,
                episode INTEGER,
                test INTEGER,
                data TEXT
            )''')
        self.con.commit()

    def _atomic_transaction(self, action):
        try:
            self.client.try_write(action)
            responses = self.client.try_read()
        except BaseException as e:
            print("An exception occurred in Cannoli Env:", e)
            responses = []
        return responses

    def step(self, action):
        current_step = self.max_steps_episode - self.steps_left

        self.action_sequence.append(action)

        episode_terminated = False
        responses = self._atomic_transaction(action)

        if not responses:
            print(f"Response is {responses} and likely action {action} is malformed")

        xs = []
        incremental_reward = 0
        for response in responses:
            req_crc = 0 if not response['req_crc'] else float(response['req_crc'])
            valid_key = 0 if not response['valid_key'] else float(response['valid_key'])
            crc_magic1 = 0 if not response['crc_magic1'] else float(response['crc_magic1'])
            crc_magic2 = 0 if not response['crc_magic2'] else float(response['crc_magic2'])
            ret = 0 if not response['return'] else float(response['return'])
            command = float(response['command'])
            invariant = float(response["invariants"])

            # reward that incentivizes the reuse and composition of previously discovered gadgets
            # gadgets that cause immediate failure cannot be composed

            if invariant:

                if ''.join(map(str, self.action_sequence)) in self.memos:
                    incremental_reward = 5
                    if self.invariants_previously_seen_in_episode:
                        incremental_reward *= 15 * self.invariants_previously_seen_in_episode
                else:
                    incremental_reward = current_step * 10
                    if self.invariants_previously_seen_in_episode:
                        incremental_reward *= 5 * self.invariants_previously_seen_in_episode
                self.invariants_previously_seen_in_episode += 1

            xs.append(
                {"req_crc": req_crc, "valid_key": valid_key, "crc_magic1": crc_magic1, "crc_magic2": crc_magic2,
                 "ret": ret,
                 "command": command, "invariant": invariant})

        # TODO: this is a hack
        if xs:
            x = xs[0]
            next_state = torch.tensor(
                [x['req_crc'], x['valid_key'], x['crc_magic1'], x['crc_magic2'], x['ret'], x['command'],
                 x['invariant']])
        else:
            next_state = torch.tensor([0, 0, 0, 0, 0, 0, 0])
        assert torch.is_tensor(next_state)

        # if we've seen an invariant before in the episode multi
        self.episode_reward += incremental_reward

        self.log(self.episode,
                 self.epoch,
                 self.episode_reward,
                 incremental_reward,
                 action[0],
                 action[1],
                 self.max_steps_episode - self.steps_left,
                 responses)

        self.last_state = next_state

        self.steps_left -= 1
        if not self.steps_left:
            episode_terminated = True

        self.observation_space[current_step] = self.last_state

        return self.observation_space, self.episode_reward, episode_terminated, []

    def log(self, episode, epoch, total_reward, incremental_reward, last_action, last_param, test, events):
        out = dict()
        out["episode"] = episode
        out["epoch"] = epoch
        out["total_reward"] = total_reward
        out["incremental_reward"] = incremental_reward
        out["last_action"] = last_action
        out["last_param"] = last_param
        out["test"] = test
        out["events"] = events
        self.db_events.append((epoch, episode, test, json.dumps(out)))

    def reset(self, **kwargs):
        try:
            self.client.reset()
        except BaseException as e:
            print("An exception occurred in Cannoli Env:", e)
        self.observation_space = torch.zeros(self.observation_shape)
        self.episode_reward = 0
        self.last_state = None
        self.steps_left = self.max_steps_episode
        self.action_sequence = []
        self.invariants_previously_seen_in_episode = 1

        # write to database
        self.db.executemany(f"INSERT INTO {self.test_name} VALUES(?, ?, ?, ?)", self.db_events)
        self.con.commit()
        self.db_events = list()

        self.episode += 1
        if self.episode == self.n_episodes:
            self.epoch += 1
            self.episode = 0

        return self.observation_space
