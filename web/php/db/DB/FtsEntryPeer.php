<?php

namespace DB;

use Criteria;





/**
 * Object Model class.
 *
 */
class FtsEntryPeer extends FtsEntryPeerBase
{

    public function selectByPageId($pageId)
    {
        $c = new Criteria();
        $c->add("page_id", $pageId);
        $ie = $this->selectOne($c);
        return $ie;
    }

    public function selectByThreadId($threadId)
    {
        $c = new Criteria();
        $c->add("thread_id", $threadId);
        $ie = $this->selectOne($c);
        return $ie;
    }
}
