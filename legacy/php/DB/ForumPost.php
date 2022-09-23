<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class ForumPost extends ForumPostBase
{

    public function getUser()
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }

        return User::find($this->getUserId());
    }

    public function getEditedUser()
    {
        if ($this->getEditedUserId() == User::ANONYMOUS_USER) {
            return null;
        }
        return User::find($this->getEditedUserId());
    }

    public function getEditedUserOrString()
    {
        $user = $this->getEditedUser();
        if ($user == null) {
            return $this->getEditedUserString();
        } else {
            return $user;
        }
    }

    public function getForumThread()
    {
        if (is_array($this->prefetched)) {
            if (in_array('forum_thread', $this->prefetched)) {
                if (in_array('forum_thread', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['forum_thread'];
                } else {
                    $obj = new ForumThread($this->sourceRow, $this->prefetched);
                    $obj->setNew(false);
                    $this->prefetchedObjects['forum_thread'] = $obj;
                    return $obj;
                }
            }
        }
        return ForumThreadPeer::instance()->selectByPrimaryKey($this->getThreadId());
    }

    public function getSite()
    {
        if (is_array($this->prefetched)) {
            if (in_array('site', $this->prefetched)) {
                if (in_array('site', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['site'];
                } else {
                    $obj = new Site($this->sourceRow, $this->prefetched);
                    $obj->setNew(false);
                    $this->prefetchedObjects['site'] = $obj;
                    return $obj;
                }
            }
        }
        return SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
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

    public function getOzoneUser()
    {
        return $this->getUser();
    }

    public function getRevision()
    {
        $r = ForumPostRevisionPeer::instance()->selectByPrimaryKey($this->getRevisionId());
        return $r;
    }
}
