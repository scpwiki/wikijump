<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class ForumPost extends ForumPostBase
{

    public function getUser()
    {
        if ($this->getUserId() == 0) {
            return null;
        }
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getEditedUser()
    {
        if ($this->getEditedUserId() == 0) {
            return null;
        }
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getEditedUserId());
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

/*
    public function save(){
        $o = new Outdater();
        $o->forumEvent("post_save", $this);
        parent::save();
    }
*/

    public function getPreview($length = 200)
    {

        $text = $this->getText();
        $text =  preg_replace(';<table style=".*?id="toc".*?</table>;s', '', $text, 1);
        $stripped = strip_tags($text);
        $d = utf8_encode("\xFE");
        $stripped = preg_replace("/".$d."module \"([a-zA-Z0-9\/_]+?)\"(.+?)?".$d."/", '', $stripped);
        $stripped = str_replace($d, '', $stripped);
        // get last position of " "
        if (strlen8($stripped)>$length) {
            $substr = substr($stripped, 0, $length);
            $length = strrpos($substr, " ");
            $substr = trim(substr($substr, 0, $length));
            $substr .= '...';
        } else {
            $substr = $stripped;
        }
        return $substr;
    }
}
