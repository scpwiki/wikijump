#
# Regular cron jobs for the wikidot package
#
16 * * * *     wikidot /usr/lib/wikidot/bin/job.sh RemoveOldSessionsJob
18 * * * *     wikidot /usr/lib/wikidot/bin/job.sh ResetUSCounterJob
33 5 * * *     wikidot /usr/lib/wikidot/bin/job.sh SendEmailDigestJob
15 9 * * *     wikidot /usr/lib/wikidot/bin/job.sh UpdateKarmaJob
* * * * *      wikidot /usr/lib/wikidot/bin/job.sh HandleBackupRequestsJob
* * * * *      wikidot /usr/lib/wikidot/bin/job.sh UpdateLuceneIndexJob
46 * * * *     wikidot /usr/lib/wikidot/bin/job.sh OutdateDatabaseStorageJob

