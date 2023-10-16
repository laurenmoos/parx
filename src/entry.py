from argparse import ArgumentParser

from train import Navigator
import os

### entry.py kicks off the navigator. 
### Using a virtualenv run from `navigator` dir
### ie: python bin/entry.py

def create_log_directory():
    logging_dir = f"{os.getcwd()}/previous_runs//"

    if not os.path.exists(logging_dir):
        os.mkdir(logging_dir)
        path = os.path.join(logging_dir, "iteration_0/")
        os.mkdir(path)


if __name__ == "__main__":
    # TODO: check iteration number and have recovery behavior with checkpoints
    """
    previous_runs/
     ├─ iteration_1/
     ├─ iteration_2/
     │  ├─ specification
     │  ├─ model_weights
     │  ├─ logs
    """
    parser = ArgumentParser()
    # this is hacky and can be replaced
    parser.add_argument("--iteration", default=0)
    # parser.add_argument("--exec_identifier", default='spray_and_pray_mips_le')
    parser.add_argument("--exec_identifier", default='cromulence_demoone')

    args = parser.parse_args()

    create_log_directory()

    nav = Navigator(args.exec_identifier, args.iteration)

    nav.navigate()
