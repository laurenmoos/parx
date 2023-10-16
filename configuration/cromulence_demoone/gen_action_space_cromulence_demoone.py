# we use an initial call to AllocPool() to determine the rough address of the
# heap. Since Bob's key is very distant (~0xc78000 bytes, or ~3000 pages), we
# short circuit the finding by adding in the offset to get us in a reasonable
# memory address range. We then control the range of address from this refined
# starting point using ALLOC_POOL_LEN, which defines the search space ( // 4,
# since we only search dword-aligned addresses)
ALLOC_POOL_START  = 0x5e46f18
ALLOC_POOL_OFFSET = 0x6abac90 - 0x5e46f18
ALLOC_POOL_LEN =   0x10000
# BOB_KEY_ADDR = 0x6abf0c8
import yaml

actions = []

locs  = list(range(ALLOC_POOL_START + ALLOC_POOL_OFFSET,
                  ALLOC_POOL_START + ALLOC_POOL_OFFSET + ALLOC_POOL_LEN,
                  4))

#  0: GetCrc (void* loc)
for l in locs:
    actions.append((0, l))

# 1: AllocatePool ()
actions.append((1, None))

# 2: GetAccessVariable (void* loc)
for l in locs:
    actions.append((2, l))

# 3: Demo1ValidateAccessKey (void* loc)
for l in locs:
    actions.append((3, l))


print(actions[0:10])

cromulence_actions_str = [f'{action[0]}, {action[1]}' for action in actions]

with open('cromulence_demoone_actions.yaml', 'w') as file:
    yaml.dump(cromulence_actions_str, file)
