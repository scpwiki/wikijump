<?php
use DB\OzoneUserPeer;

/**
 * Sends email digest with unread notifications (if a user accepts this)
 */
class SendEmailDigestJob implements SchedulerJob
{

    public function run()
    {

        $ds = new WDDigestSender();

        // select users... all at once??? fix this!
        $c = new Criteria();
        $c->add("user_id", 0, ">");
        $c->addOrderAscending("user_id");

        $users = OzoneUserPeer::instance()->select($c);

        foreach ($users as $user) {
            try {
                $ds->handleUser($user);
            } catch (Exception $e) {
            }
        }
    }
}
