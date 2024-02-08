# Judger Data

In this section we provide two approaches to connect problem packages with judger.

1. By accessing a local directory,
which is more convenient when you can't not access OJ Lab's platform data collection.
(Ex. you are using github codespaces or some other cloud IDE)
2. By setting up a connection by rclone,
this approach will be closer to the real situation.

## Configure rclone-minio.conf

Copy the example file and then edit in your own configuration.

```sh
cp -i data/rclone-minio.conf.example data/rclone-minio.conf
```

You will need to replace `YOUR_ACCESS_KEY` and `YOUR_SECRET_KEY`
with your own access key and secret key
(which can be generated in the minio web interface,
visit <http://127.0.0.1:9001> if you are using oj-lab-platform's docker-compose).

Then you can run the following command to test if the configuration is correct.

```sh
rclone --config data/rclone-minio.conf ls minio:
```

You should notice that the `minio:` is the name of the remote,
remember to replace it with your own remote name if you change it.

### Use rclone sync

By design judger will try sync the remote problem package
everytime when it got a judge task,
so that we can ensure the problem package is up-to-date.

```sh
rclone --config data/rclone-minio.conf sync minio:oj-lab-problem-package data/rclone-problem-package
```

## gitignored files

Some files are gitignored in this directory.
If you follow this README to setup rclone problem package,
this will help keeping your repo tidy.
