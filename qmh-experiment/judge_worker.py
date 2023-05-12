import logging
import random

from utils import parse_args


def judge_worker():
    meta_data = eval(vars(parse_args())['data'])
    job_id = meta_data['uid']
    data_dict = eval(meta_data['data_dict'])

    logging.basicConfig(filename=f'./fake-file-system/logs/{job_id}.log', level=logging.DEBUG)
    logging.info(f'judge_worker: {job_id}')

    # judge
    results = []
    for _ in data_dict:
        x = random.random()
        if x < 0.8:
            results.append('AC')
        elif x < 0.9:
            results.append('WA')
        else:
            results.append('TLE')
    logging.info(results)

    # log final result
    if 'TLE' in results:
        logging.warning('result: TLE')
    elif 'WA' in results:
        logging.warning('result: WA')
    else:
        logging.warning('result: AC')


if __name__ == '__main__':
    judge_worker()
