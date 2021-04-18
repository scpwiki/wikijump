<?php

namespace Ozone\Framework;

/**
 * Utility Class for providing unique strings.
 *
 */
class UniqueStrings {
    private static $tr_lastTime;
    private static $tr_lastTimeIssuedNumber = 0;

    /**
     * Returns time-based + int element string.
     */
    public static function timeBased(){
        // TODO

        $timePart = time().'';
        // TRANSACTION OR TABLE LOCK SHOULD START HERE
        $db = Database::connection();
        $db->begin();
        $c = new Criteria();
        $c->setForUpdate(true);
        $index = UniqueStringBrokerPeer::instance()->selectOne($c);
        if($index != null){
            $idx = $index->getLastIndex();
            $number = $idx + 1;
            //update index + 1
            UniqueStringBrokerPeer::instance()->increaseIndex();
        } else {
            $number = 0;
            UniqueStringBrokerPeer::instance()->init();
        }
        $db->commit();
        // TRANSACTION OR TABLE LOCK SHOULD END HERE
        return $timePart."_".$number;
    }

    public static function random_string(int $length) : string
    {
        $bytes = random_bytes($length); // Returns a string of double the requested length.
        return substr(bin2hex($bytes), 0, $length);
    }

    public static function resetCounter(){
        // sets last_index = 0
        UniqueStringBrokerPeer::instance()->reset();
    }
}
