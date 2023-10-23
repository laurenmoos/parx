import gym
import torch
import pytorch_lightning as pl
from torch.utils.data import DataLoader
import torch.optim as optim
from reward.intrinsic_curiosity_module import ICM
from agent.recurrent_actor_critic import ActorCritic
from typing import List
from agent.replay_buffer import MiniBatch, Batch, Episode
import csv
from tdigest import TDigest
import os

import ast


def discount_rewards(rewards: List[float], discount: float) -> list:
    # computes the discounted reward given the list of rewards and a discount coefficient

    cumul_reward = []
    sum_r = 0.0

    for r in reversed(rewards):
        sum_r = (sum_r * discount) + r
        cumul_reward.append(sum_r)

    r = list(reversed(cumul_reward))
    return r


def calc_advantage(rewards: list, values: list, last_value: float, gamma: float, lam: float) -> list:
    # generalized advantage estimation
    rews = rewards + [last_value]
    vals = values + [last_value]

    delta = [rews[i] + gamma * vals[i + 1] - vals[i] for i in range(len(rews) - 1)]

    return discount_rewards(delta, gamma * lam)


'''
proximal policy optimization 

for iteration i....t
    for actor:
        run old_policy
        compute advantage estimates 
    end
    optimization surrogate for K epochs consisting of B mini-batches of sampled data
    update new_policy with old_policy
'''


class RiskAwarePPO(pl.LightningModule):

    def __init__(
            self,
            path_to_specification: str,
            env,
            batch_size: int,
            episodes: int,
            steps_per_episode: int,
            nb_optim_steps: int,
            learning_rate: tuple,
            value_loss_coef: float,
            entropy_beta: float,
            clip_ratio: float,
            gamma: float,
            lam: float,
            risk_aware: bool,
            k: float,
            curious: bool,
            state_dim: int,
            latent_space: int,
            recurrent: int,
            policy_weight: float,
            reward_scale: float,
            weight: float,
            intrinsic_reward_integration: float
    ):
        super().__init__()

        self.dir = path_to_specification

        self.env = env
        self.obs_shape, self.action_shape = self.env.observation_space.shape, len(env.action_space)

        self.action_space = env.action_space
        self.obs_space = env.observation_space

        self.actor_lr, self.critic_lr = learning_rate

        self.agent = ActorCritic(state_dim=state_dim, action_dim=self.action_shape,
                                 hidden_size=latent_space, recurrent_layers=recurrent, actor_lr=self.actor_lr,
                                 critic_lr=self.critic_lr)

        self.gamma = gamma
        self.lam = lam

        self.batch_size = batch_size
        self.episodes = episodes
        self.steps_per_episode = steps_per_episode
        self.nb_optim_steps = nb_optim_steps

        self.batch_size = batch_size

        self.automatic_optimization = False

        self.optimizer_step = 0

        if risk_aware:
            self.k = k

        if curious:
            self.icm = ICM(self.obs_shape[0], self.action_shape, state_latent_size, policy_weight, reward_scale, weight)
            self.intrinsic_reward_integration = intrinsic_reward_integration

        self.value_loss_coef = value_loss_coef
        self.entropy_beta = entropy_beta
        self.clip_ratio = clip_ratio

        self.batch = MiniBatch()

        # encapsulates all the collections that compose an episode
        self.episode = Episode()

        self.epoch_rewards = []

        self.episode_step = 0
        self.avg_ep_reward = 0

        self.state = self.env.reset()

        self.td = TDigest()

    def forward(self, x: torch.Tensor):
        pi, action = self.agent.actor(x)
        value = self.agent.critic(x)

        return pi, action, value

    def _compute_episode_reward(self, rewards, values):
        last_value = values[-1][-1]

        # enrich rewards with intrinsic reward
        # intr_temp_reward = self.icm.temp_reward(self.episode.rewards, self.episode.states, self.episode.actions)
        # intr_contr_reward = self.icm.oracle_reward(self.episode.rewards, self.episode.states, self.episode.actions)
        #
        # intrinsic = 0.5 * intr_temp_reward + 0.5 * intr_contr_reward
        # agg_rewards = (1. - self.intr_weight) * self.episode.rewards + self.intr_weight * intrinsic
        agg_rewards = rewards

        qvals = discount_rewards(agg_rewards + [last_value], self.gamma)[:-1]
        adv = calc_advantage(agg_rewards, values, last_value, self.gamma, self.lam)

        assert len(qvals) == self.steps_per_episode and len(adv) == self.steps_per_episode

        return qvals, adv

    def train_batch(self) -> tuple:
        for episode_idx in range(self.episodes):
            for step in range(self.steps_per_episode):
                pi, action, log_prob, value = self.agent(self.state)

                assert action.shape[0] == self.steps_per_episode

                # for the action of the current step, take the indexed action from the action space

                next_state, reward, done, _ = self.env.step(self.action_space[action[step]])

                assert next_state.shape[0] == self.steps_per_episode

                self.episode_step += 1

                self.batch.update_experience(state=self.state[step, :], next_state=next_state[step, :],
                                             action=action[step],
                                             logp=log_prob[step])
                self.episode.update(reward=reward, value=value, state=self.state[step], action=action[step])

                self.state = next_state

                terminal = len(self.episode.rewards) == self.steps_per_episode

                if done or terminal:
                    qvals, adv = self._compute_episode_reward(self.episode.rewards, self.episode.values)

                    sum_episode_rewards = sum(self.episode.rewards)
                    self.epoch_rewards.append(sum_episode_rewards)

                    self.td.update(sum_episode_rewards)
                    top_quintile = self.td.percentile(self.k)
                    if sum_episode_rewards >= top_quintile:
                        filename = f'reproducibility_criteria/epoch:{self.current_epoch}.csv'
                        file_exists = os.path.isfile(filename)

                        # Open the file in the appropriate mode
                        mode = 'a' if file_exists else 'w'
                        with open(filename, mode) as csvfile:
                            reproducibility_criteria = csv.writer(csvfile)
                            human_read_actions = [self.action_space[int(x)] for x in self.episode.actions]
                            reproducibility_criteria.writerow(human_read_actions)
                            human_read_states = []
                            for state in self.episode.states:
                                req_crc = int(state[0])
                                valid_key = int(state[1])
                                crc_magic1 = int(state[2])
                                crc_magic2 = int(state[3])
                                ret = int(state[4])
                                command = int(state[5])
                                invariant = int(state[6])
                                human_read_states.append({"req_crc": req_crc, "valid_key": valid_key,
                                                          "crc_magic1": crc_magic1, "crc_magic2": crc_magic2,
                                                          "ret": ret,
                                                          "command": command, "invariant": invariant})

                            reproducibility_criteria.writerow(human_read_states)

                    # reset episode
                    self.episode.reset()
                    self.episode_step = 0
                    self.state = self.env.reset()

                    yield torch.stack(self.batch.states), torch.stack(self.batch.next_states), torch.stack(
                        self.batch.actions), \
                        torch.stack(self.batch.logp), adv[-1], qvals[-1]

                    self.batch.reset()

            self.avg_ep_reward = sum(self.epoch_rewards) / self.steps_per_episode

    def configure_optimizers(self) -> tuple:
        # initialize optimizer
        optimizer_actor = optim.Adam(self.agent.actor.parameters(), lr=self.actor_lr)
        optimizer_critic = optim.Adam(self.agent.critic.parameters(), lr=self.critic_lr)

        return optimizer_actor, optimizer_critic

    def optimizer_step(self, *args, **kwargs):
        for i in range(self.nb_optim_steps):
            super().optimizer_step(*args, **kwargs)

    def train_dataloader(self) -> DataLoader:
        return DataLoader(dataset=Batch(self.train_batch), batch_size=self.batch_size)

    def _assert_batch(self, state, next_state, action, old_logp, adv, qval):
        assert state.shape[0] <= self.batch_size
        assert state.shape[1] == self.steps_per_episode

        assert next_state.shape[0] <= self.batch_size
        assert state.shape[1] == self.steps_per_episode

        assert action.shape[0] <= self.batch_size and action.shape[1] == self.steps_per_episode

        assert old_logp.shape[0] <= self.batch_size and old_logp.shape[1] == self.steps_per_episode

        assert adv.shape[0] <= self.batch_size and adv.shape[1] == self.steps_per_episode

        assert qval.shape[0] <= self.batch_size

    def training_step(self, batch: tuple, batch_idx):
        state, next_state, action, old_logp, adv, qval = batch

        self._assert_batch(state, next_state, action, old_logp, adv, qval)

        adv = (adv - adv.mean()) / adv.std()

        self.log("avg_ep_reward", self.avg_ep_reward, prog_bar=True, on_step=False, on_epoch=True)

        if not self.optimizer_step:

            loss_actor = self.actor_loss(state, action, old_logp, adv)
            self.log('loss_actor_raw', loss_actor, on_step=False, on_epoch=True, prog_bar=True, logger=True)
            # intrinsic_loss = self.icm.loss(loss_actor, action, next_state, state)
            # self.log('loss_actor_curious', loss_actor, on_step=False, on_epoch=True, prog_bar=True, logger=True)
            self.optimizer_step = 1
            return loss_actor
        else:

            loss_critic = self.critic_loss(state, qval)
            self.log('loss_critic', loss_critic, on_step=False, on_epoch=True, prog_bar=False, logger=True)
            self.optimizer_step = 0
            return loss_critic

    def get_device(self, batch) -> str:
        return batch[0].device.index if self.on_gpu else 'cpu'

    def actor_loss(self, state, action, logp_old, adv) -> torch.Tensor:
        pi, _ = self.agent.actor(state)
        logp = self.agent.actor.get_log_prob(pi, action)
        ratio = torch.exp(logp - logp_old)

        # this is the PPO bit - i.e. pessimistic update of policy minimizing amount of entropy epoch over epoch
        clip_adv = torch.clamp(ratio, 1 - self.clip_ratio, 1 + self.clip_ratio) * adv
        loss_actor = -(torch.min(ratio * adv, clip_adv)).mean()

        return loss_actor

    def critic_loss(self, state, qval) -> torch.Tensor:
        value = self.agent.critic(state)
        loss_critic = (qval - value).pow(2).mean()

        assert value.shape[0] <= self.batch_size and value.shape[1] == self.steps_per_episode and len(value.shape) == 3

        return loss_critic
