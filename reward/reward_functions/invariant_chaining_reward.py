class InvariantChainingReward:

    def __init__(self):

        # TODO: logic for finding the unique, invariant-causing gadget

        self.memos = []

        self.n_invariants_seen = 0

    def step(self, action_sequence, current_step):
        if action_sequence in self.memos:
            incremental_reward = 5
            if self.n_invariants_seen:
                incremental_reward *= 15 * self.n_invariants_seen
            else:
                incremental_reward = current_step * 10
                if self.n_invariants_seen:
                    incremental_reward *= 5 * self.n_invariants_seen
        self.n_invariants_seen += 1
