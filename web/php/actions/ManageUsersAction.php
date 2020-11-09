<?php
use DB\OzoneUserPeer;
use DB\OzoneUser;
use DB\Member;
use DB\Admin;
use DB\AdminPeer;
use DB\Moderator;
use DB\ModeratorPeer;

class ManageUsersAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        if ($runData->getTemp("site")->getSiteId() != 1) {
            throw new WDPermissionException("No permission");
        }
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function perform($r)
    {
    }

    public function saveEvent($runData)
    {
        $params = $runData->getParameterList()->asArray();

        $ids = array();
        foreach ($params as $param_key => $param_val) {
            $m = array();
            if (preg_match('/^nick_name_([new0-9]+)$/', $param_key, $m)) {
                $ids[] = $m[1];
            }
        }

        foreach ($ids as $id) {
            $nick_name = $params["nick_name_$id"];
            $password = $params["password_$id"];
            $admin = $params["admin_$id"] ? true : false;
            $mod = $params["mod_$id"] ? true : false;

            $site = $runData->getTemp('site');

            if ($nick_name) {
                if ($id = 1 * $id) {
                    $u = OzoneUserPeer::instance()->selectByPrimaryKey($id);
                } else {
                    $u = null;
                }

                $next = false;

                if (! $u) {
                    $u = new OzoneUser();
                    if (! $password) {
                        $next = true;
                    }

                    $u->save();

                    $m = new Member();
                    $m->setUserId($u->getUserId());
                    $m->setSiteId($site->getSiteId());
                    $m->save();
                }

                if (! $next) {
                    $u->setName($nick_name);
                    $u->setEmail($nick_name);
                    $u->setNickName($nick_name);
                    $u->setUnixName(WDStringUtils::toUnixName($nick_name));

                    if ($password) {
                        $u->setPassword($password);
                    }

                    $u->save();

                    if ($admin) {
                        if (! WDPermissionManager::hasPermission('manage_site', $u, $site)) {
                            $a = new Admin();
                            $a->setUserId($u->getUserId());
                            $a->setSiteId($site->getSiteId());
                            $a->save();
                        }
                    } else { // ! $admin
                        $c = new Criteria();
                        $c->add('site_id', $site->getSiteId());
                        $c->add('user_id', $u->getUserId());
                        AdminPeer::instance()->delete($c);
                    }

                    if ($mod) {
                        if (! WDPermissionManager::hasPermission('moderate_site', $u, $site)) {
                            $m = new Moderator();
                            $m->setUserId($u->getUserId());
                            $m->setSiteId($site->getSiteId());
                            $m->save();
                        }
                    } else { // ! $mod
                        $c = new Criteria();
                        $c->add('site_id', $site->getSiteId());
                        $c->add('user_id', $u->getUserId());
                        ModeratorPeer::instance()->delete($c);
                    }
                }
            }
        }
    }
}
