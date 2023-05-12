from time import sleep
import os
import yaml
import random

from utils import uuid


JOBS_PATH = './fake-file-system/temp'
FAKE_K8S_UID = 0


def fake_secure_run_job(job_info):
    if job_info['name'] == 'judger_master_job':
        uid = uuid(FAKE_K8S_UID)
        data = {
            'problem_id': job_info['data']['problem_id'],
            'submission_id': job_info['data']['submission_id'],
            'job_id': uid
        }
        command = f'start /b python judge_master.py --data \"{str(data)}\"'  # nohup on windows
        # command = f'nohup python judge_master.py --data \"{str(data)}\" &'
    elif job_info['name'] == 'judger_worker_job':
        data = {
            'binary_path': job_info['data']['binary_path'],
            'problem_id': job_info['data']['problem_id'],
            'data_dict': job_info['data']['data_dict'],
            'uid': job_info['data']['uid']
        }
        command = f'start /b python judge_worker.py --data \"{str(data)}\"'
    else:
        raise NotImplementedError

    # Notice: A real K8s job runner would not make a synchronous function call
    # Notice: K8s pods will deal with memory_limit, time_limit and value
    # See: https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/
    print(f'securely running job: {job_info}')
    print(f'running command: {command}')
    os.system(command)


def deal_with_yaml(config):
    yaml_path = os.path.join(JOBS_PATH, config)
    data_dict = None
    with open(yaml_path, 'r') as file:
        data_dict = yaml.load(file.read(), Loader=yaml.FullLoader)
    if data_dict['type'] == 'job':
        os.remove(yaml_path)
        fake_secure_run_job(data_dict)
    else:
        raise NotImplementedError


def fake_k8s_scheduler(configs):
    # Notice: A real K8s scheduler will arrange the jobs according to priority
    # See: https://kubernetes.io/docs/concepts/scheduling-eviction/pod-priority-preemption/
    random.shuffle(configs)
    for config in configs:
        deal_with_yaml(config)


def fake_k8s():
    while True:
        sleep(1)
        configs = os.listdir(JOBS_PATH)
        for config in configs:
            print(f'found new yaml: {config}')
        fake_k8s_scheduler(configs)


if __name__ == '__main__':
    fake_k8s()
