<?php

namespace Wikidot\DB;


use Ozone\Framework\Database\Criteria;
use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class PageEditLock extends PageEditLockBase
{

    public function getConflicts()
    {
        $c = $this->getConflictsCriteria();
        $conflicts = PageEditLockPeer::instance()->select($c);
        if (count($conflicts) == 0) {
            return null;
        } else {
            return $conflicts;
        }
    }

    public function deleteConflicts()
    {
        $c = $this->getConflictsCriteria();
        PageEditLockPeer::instance()->delete($c);
    }

    public function getConflictsCriteria()
    {
        // get conflicting page locks

        // if lock for a new page...
        // anyway one should also check if a page does not exist
        // (done somewhere else)

        if ($this->getPageId() == null) {
            $c = new Criteria();
            $c->add("page_unix_name", $this->getPageUnixName());
            $c->add("site_id", $this->getSiteId());
            if ($this->getLockId() != null) {
                $c->add("lock_id", $this->getLockId(), "!=");
            }
        } else {
            // now if the page exists.

            if ($this->getMode() == "page") {
                // conflicts with any other type of lock for this page...
                $c = new Criteria();
                $c->add("page_id", $this->getPageId());
                if ($this->getLockId() != null) {
                    $c->add("lock_id", $this->getLockId(), "!=");
                }
            }

            if ($this->getMode() == "append") {
                // conflicts only with "page" mode
                $c = new Criteria();
                $c->add("page_id", $this->getPageId());
                if ($this->getLockId() != null) {
                    $c->add("lock_id", $this->getLockId(), "!=");
                }
                $c->add("mode", "page");
            }
        }
        return $c;
    }

    public function getUser()
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }
        return User::find($this->getUserId());
    }

    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }

    /**
     * Calculate number of seconds this lock will expire.
     */
    public function getExpireIn()
    {
        return $this->getDateLastAccessed()->getTimestamp() + 15 * 60 - time();
    }

    public function getStartedAgo()
    {
        return time() - $this->getDateStarted()->getTimestamp();
    }

    public function getOzoneUser()
    {
        return User::find($this->getUserId());
    }
}
