<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class PrivateMessage extends PrivateMessageBase
{

    public function getFromUser()
    {
        return User::find($this->getFromUserId());
    }

    public function getToUser()
    {
        if ($this->getToUserId() !== null) {
            return User::find($this->getToUserId());
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
