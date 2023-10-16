# https://gitlab.com/ngoodger/ppo_lstm/-/blob/master/recurrent_ppo.ipynb
# this link contains an implementation of a recurrent policy but with a continuous action space
from typing import Any

import pytorch_lightning as pl
from torch import nn

class LambdaModule(nn.Module):

    def __init__(self):
        super().__init__()

    def forward(self, x):
        return x[0]


def create_lstm(state_dim, hidden_size, recurrent_layers):
    return nn.LSTM(state_dim, hidden_size, num_layers=recurrent_layers)


class RewardModel(pl.LightningModule):
    """
    This reward model predicts a relative ranking given a trace.
    Once the model is trained, this acts as an estimation for the preference of the end user.
    This allows user preferences to be incorporated iteratively every time the reward is computed.

    re. https://proceedings.neurips.cc/paper_files/paper/2017/file/d5e2c0adad503c91f91df240d0cd4e49-Paper.pdf
    """

    def __init__(self, state_dim, hidden_size, recurrent_layers, latent_size, *args: Any, **kwargs: Any):
        """
        state_dim: state here is an embedding of the entire trace vs. a single state
        hidden_size: bottleneck for the embedding
        recurrent_layers: number of recurrent layers for the lstm
        latent_size: latent size for network
        """
        super().__init__(*args, **kwargs)
        # TODO: represent a state AND transitions to incorporate more information about the trace
        self.actor_net = nn.Sequential(
            create_lstm(state_dim, hidden_size, recurrent_layers),
            LambdaModule(),
            nn.Linear(latent_size, latent_size),
            # in the last layer the model is predicting the user-assigned score (Likert ranking)
            nn.Linear(latent_size, 1)
        )
        # the output is of size 1 and is a float that can range from 0 to the number of traces
        # returned in each iteration

    def forward(self, state, **kwargs):
        return self.actor_net(state).squeeze().squeeze()
