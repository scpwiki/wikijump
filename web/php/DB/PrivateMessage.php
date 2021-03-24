<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class PrivateMessage extends PrivateMessageBase
{

    public function getFromUser()
    {
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getFromUserId());
    }

    public function getToUser()
    {
        if ($this->getToUserId() !== null) {
            return OzoneUserPeer::instance()->selectByPrimaryKey($this->getToUserId());
        }
    }

    public function getPreview($length = 200)
    {

        $text = $this->getBody();

        $stripped = strip_tags($text);

        $substr = substr($stripped, 0, $length);
        if (strlen8($substr) == $length) {
            $substr = preg_replace('/\w+$/', "", $substr) . '...';
        }
        return $substr;
    }
}
