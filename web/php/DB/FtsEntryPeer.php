<?php

namespace Wikidot\DB;



use Ozone\Framework\Database\Criteria;





/**
 * Object Model Class.
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
