from cannoli_env import CannoliEnv

env = CannoliEnv("wrap_and_win_mips_le", 1024, None)

while True:
    env.step("1")
