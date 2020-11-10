<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class PageRateVote extends PageRateVoteBase
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
}
