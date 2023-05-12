import os


def fake_compiler(source_code):
    if not os.path.exists(source_code):
        return False, None
    runnable_path = source_code[:-4]
    with open(runnable_path, 'w') as file:
        file.write('echo hello world')
    return True, runnable_path
