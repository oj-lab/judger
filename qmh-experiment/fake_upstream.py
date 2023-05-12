"""
A fake upstream is introduced to create fake submission consistantly,
so as to test the whole experiment system.
"""


import time
import random
from utils import uuid
import yaml

FAKE_UPSTREAM_UID = 1


def send_judger_master_job_to_k8s(problem_id, submission_id):
    uid = uuid(FAKE_UPSTREAM_UID)

    # create dict
    data_dict = None
    with open('./fake-file-system/template/judge-master-job.yaml', 'r') as template:
        data_dict = yaml.load(template.read(), Loader=yaml.FullLoader)
    data_dict['data']['problem_id'] = problem_id
    data_dict['data']['submission_id'] = submission_id

    # send dict to k8s
    print(f'send job to k8s uuid={uid}')
    with open(f'./fake-file-system/temp/{uid}', 'w') as file:
        file.write(yaml.dump(data_dict, allow_unicode=True))


def fake_new_judger_job():
    submission_id = 0
    while True:
        # sleep for a while
        random_sleep = random.randint(5, 10)
        print(f'sleep {random_sleep}s')
        time.sleep(random_sleep)

        # create a fake submission
        submission_id += 1
        problem_id = random.randint(1, 2)
        print(f'creating job problem_id={problem_id}, submission_id={submission_id} -b')
        with open(f'./fake-file-system/submission/{submission_id}.cpp', 'w') as fake_submission_file:
            fake_submission_file.write('print(\'Hello world\')')

        # send signal to k8s
        send_judger_master_job_to_k8s(problem_id, submission_id)


if __name__ == '__main__':
    fake_new_judger_job()
