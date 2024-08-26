Real time monitoring for the Puerto Rico electrical grid using Genera-PR (generation) and Luma (distribution) data.

The crontab used runs on a 2 minute schedule (update frequency for genera-pr, luma is updated every 5 minutes but the time constraint is handled and discarded)

example to run the development debug version with a .env file attached and dumping logs to a file in the development directory
`*/2 * * * * DOT_ENV=/home/my_user/dev/genera-pr/.env /home/my_user/dev/genera-pr/target/debug/genera-pr >> /home/my_user/dev/genera-pr/cron_logfile.log 2>&1`

![image](https://github.com/user-attachments/assets/857cad0a-4316-40fb-b911-253433a8a909)
