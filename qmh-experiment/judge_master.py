import yaml
import logging
import random
import os

from fake_compiler import fake_compiler
from utils import parse_args, uuid

FAKE_MASTER_UID = 2
NUM_WORKERS = 2


def send_judger_worker_job_to_k8s(binary_path, problem_id, data_id_dict, time_limit, memory_limit, priority):
    uid = uuid(FAKE_MASTER_UID)

    # create dict
    data_dict = None
    with open('./fake-file-system/template/judge-worker-job.yaml', 'r') as template:
        data_dict = yaml.load(template.read(), Loader=yaml.FullLoader)
    data_dict['data']['binary_path'] = binary_path
    data_dict['data']['problem_id'] = problem_id
    data_dict['data']['data_dict'] = str(data_id_dict)
    data_dict['data']['uid'] = uid
    data_dict['resource']['time_limit'] = time_limit
    data_dict['resource']['memory_limit'] = memory_limit
    data_dict['value'] = priority

    # send dict to k8s
    print(f'send job to k8s uuid={uid}')
    with open(f'./fake-file-system/temp/{uid}', 'w') as file:
        file.write(yaml.dump(data_dict, allow_unicode=True))
    return uid


def judge_master():
    meta_data = eval(vars(parse_args())['data'])
    problem_id = meta_data['problem_id']
    submission_id = meta_data['submission_id']
    job_id = meta_data['job_id']
    priority = random.randrange(1000)  # the bigger, the more prioritized

    logging.basicConfig(filename=f'./fake-file-system/logs/{job_id}.log', level=logging.DEBUG)
    logging.info(f'judge_master: {job_id}')
    logging.info(f'problem_id: {problem_id}')
    logging.info(f'submission_id: {submission_id}')
    logging.info(f'priority: {priority}')

    problem_data = None
    with open(f'./fake-file-system/data/problem-{problem_id}/meta.txt') as file:
        problem_data = yaml.load(file.read(), Loader=yaml.FullLoader)
    logging.info(f'problem_data: {problem_data}')

    checkpoints = [i + 1 for i in range(problem_data['checkpoints'])]
    time_limit = problem_data['timelimit']
    memory_limit = problem_data['memorylimit']

    # compile
    compile_result, binary_path = fake_compiler(f'./fake-file-system/submission/{submission_id}.cpp')
    logging.info(f'compile: {compile_result}')
    if not compile_result:
        logging.warn('result: CE')
        exit(0)

    # random scheduler
    workload = [[] for _ in range(NUM_WORKERS)]
    for task in checkpoints:
        workload[random.randrange(NUM_WORKERS)].append(task)
    logging.info(f'workload: {workload}')

    # create workers
    worker_ids = []
    for tasks in workload:
        wid = send_judger_worker_job_to_k8s(binary_path, problem_id, tasks, time_limit, memory_limit, priority)
        worker_ids.append(wid)

    # collect result
    # TODO: I actually dont know the best practice to collect k8s job, but I assure there is a way to do so.
    # Maybe through sql query?
    results = [None] * NUM_WORKERS
    while None in results:
        for worker in range(NUM_WORKERS):
            if results[worker] is not None:
                continue
            worker_id = worker_ids[worker]
            log_path = f'./fake-file-system/logs/{worker_id}.log'
            if os.path.exists(log_path):
                result = None
                with open(log_path) as file:
                    ret = file.read()
                    if 'result: AC' in ret:
                        result = 'AC'
                    elif 'result: WA' in ret:
                        result = 'WA'
                    elif 'result: TLE' in ret:
                        result = 'TLE'
                results[worker] = result

    # TODO: write result into SQL
    if 'TLE' in results:
        logging.warning('result: TLE')
    elif 'WA' in results:
        logging.warning('result: WA')
    else:
        logging.warning('result: AC')


if __name__ == '__main__':
    judge_master()
