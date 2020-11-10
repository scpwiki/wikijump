<?php
use DB\OzoneUserPeer;
use DB\MemberPeer;

class ManageSiteUserAbuseModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get
        $q = "SELECT target_user_id, count(*) AS rank " .
                "FROM user_abuse_flag " .
                "WHERE site_id='".$site->getSiteId()."' " .
                "AND site_valid = TRUE GROUP BY target_user_id ORDER BY rank DESC, target_user_id";

        $db = Database::connection();
        $res = $db->query($q);

        $all = $res->fetchAll();

        $r2 = array();

        if ($all) {
            foreach ($all as &$r) {
                // get user
                $user = OzoneUserPeer::instance()->selectByPrimaryKey($r['target_user_id']);
                if ($user) {
                    $r['user'] = $user;
                    // check if member
                    $c = new Criteria();
                    $c->add("site_id", $site->getSiteId());
                    $c->add("user_id", $user->getUserId());
                    $mem = MemberPeer::instance()->selectOne($c);
                    if ($mem) {
                        $r['member'] = $mem;
                    }
                    $r2[] = $r;
                }
            }
        }

        $runData->contextAdd("reps", $r2);
    }
}
