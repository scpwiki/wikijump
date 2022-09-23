<?php

namespace Wikidot\Jobs;

use Exception;
use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SchedulerJob;

use Wikidot\Utils\WDDigestSender;

/**
 * Sends email digest with unread notifications (if a user accepts this)
 */
class SendEmailDigestJob implements SchedulerJob
{
    public function run()
    {
        DB::table('users')->orderBy('id')->chunk(100, function($users) {
            foreach ($users as $user) {
                try {
                    $ds = new WDDigestSender();
                    $ds->handleUser($user);
                } catch (Exception $e) {
                }
            }
        });
    }
}
