import time
import random
import argparse
import sys


def uuid(who: int) -> str:
    return str(int(time.time())) + str(who) + str(random.randint(0, 10000000)).zfill(9)


def parse_args(argv=sys.argv):
    parser = argparse.ArgumentParser()
    parser.add_argument('--data', type=str)
    args = parser.parse_args(argv[1:])
    return args
