# OVMF prints this value to debug.log and is static on repeat runs
DRIVER_LOAD_LOC = 0x000067F4000
# Found by running `readpe CromulenceExample1.efi`
DRIVER_DATA_OFF =        0x4300
DRIVER_DATA_LEN =         0x1c0

# Using debug.py and `p lockbox_start`
LOCKBOX_LOC     =     0x67f9000
LOCKBOX_LEN     =        0x4000

import yaml

actions = []

data  = [0, 1, 1337, 7331, 0xdeadbeef, 0xffffffff,
         DRIVER_LOAD_LOC, LOCKBOX_LOC+LOCKBOX_LEN]

locs  = list(range(DRIVER_LOAD_LOC + DRIVER_DATA_OFF,
                  DRIVER_LOAD_LOC + DRIVER_DATA_OFF + DRIVER_DATA_LEN,
                  4))

locs += list(range(LOCKBOX_LOC, LOCKBOX_LOC + int(LOCKBOX_LEN/4), 4))

# 0: SetLockPin (uint32_t lockpin_data)
for d in data:
    actions.append((0, d, None))

# 1: WriteData (void* loc, uint32_t data)
for l in locs:
    for d in data:
        actions.append((1, l, d))

#  2: []} # GetCrc (void* loc)
for l in locs:
    actions.append((2, l, None))

print(actions[0:10])

cromulence_actions_str = [f'{action[0]}, {action[1]}, {action[2]}' for action in actions]

# with open('cromulence_actions.yaml', 'w') as file:
with open('test.yaml', 'w') as file:
    yaml.dump(cromulence_actions_str, file)
