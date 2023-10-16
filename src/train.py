import os
from argparse import ArgumentParser
from pytorch_lightning import Trainer
from proximal_policy_optimization import RiskAwarePPO
from libs.cannoli_env import CannoliEnv
import yaml
import numpy as np
import ast
from pytorch_lightning.callbacks import ModelCheckpoint

try:
    import gym
except  ModuleNotFoundError:
    _GYM_AVAILABLE = False
else:
    _GYM_AVAILABLE = True

CONFIG_DIR = f"{os.getcwd()}" + "/configuration/cromulence_demoone/"


class Navigator:

    def __init__(self, exec_identifier, iteration_number):
        self.exec_identifier = exec_identifier

        if not _GYM_AVAILABLE:
            raise ModuleNotFoundError('This Module requires gym environment which is not installed yet.')

        self.x = None
        with open(CONFIG_DIR + self.exec_identifier + '.yaml', 'r') as file:
            self.x = yaml.safe_load(file)

        self.dir_path = os.getcwd() + "/" + f"previous_runs/iteration_{iteration_number}" + "/"

        self.ppo = self.load()

        # this logs the best performing model weights every epoch (according to the val loss)
        checkpoint_callback = ModelCheckpoint(dirpath=self.dir_path, save_top_k=1, monitor="val_loss")

        self.trainer = Trainer(accelerator=self.x['accelerator'], devices=self.x['devices'],
                               max_epochs=self.x['epochs'], callbacks=[checkpoint_callback])

    def load(self):

        with open(CONFIG_DIR + self.x['actions'], 'r') as file:
            actions = yaml.safe_load(file)
            actions = actions.split(" ")
            trans_actions = []
            for action in actions:
                action = tuple(map(int, action.split(',')))
                trans_actions.append(action)

        env = CannoliEnv(self.exec_identifier, self.x['steps'], trans_actions, self.x['episodes'], self.x['epochs'])

        return RiskAwarePPO(
            self.dir_path,
            env,
            self.x['batch_size'],
            self.x['episodes'],
            self.x['steps'],
            self.x['nb_optim_iters'],
            ast.literal_eval(self.x["learning_rate"]),
            self.x['value_loss_coef'],
            self.x['entropy_beta'],
            self.x['clip_ratio'],
            self.x['gamma'],
            self.x['lam'],
            self.x['risk_aware'],
            self.x['k'],
            self.x['curious'],
            self.x['state_dim'],
            self.x['latent_space'],
            self.x['recurrent'],
            self.x['policy_weight'],
            self.x['reward_scale'],
            self.x['weight'],
            self.x['intrinsic_reward_integration']
        )

    def navigate(self):
        self.trainer.fit(self.ppo)
