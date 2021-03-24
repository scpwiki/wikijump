<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\ODate;
use Wikidot\DB\StorageItemPeer;
use Wikidot\DB\StorageItem;

class DatabaseStorage
{

    private static $instance;

    public static function instance()
    {
        if (!self::$instance) {
            self::$instance = new DatabaseStorage();
        }
        return self::$instance;
    }

    public function set($key, $value, $timeout)
    {

        // delete it if already in the database
        StorageItemPeer::instance()->deleteByPrimaryKey($key);
        if (!$value) {
            return;
        }
        $item = new StorageItem();
        $item->setItemId($key);
        $item->setData($value);
        $item->setTimeout($timeout);
        $item->setDate(new ODate());

        $item->save();
    }

    public function get($key)
    {
        $item = StorageItemPeer::instance()->selectByPrimaryKey($key);
        if ($item) {
            $timestamp = $item->getDate()->getTimestamp() + $item->getTimeout();

            if ($timestamp < time()) {
                // delete the item, it is outdated!
                StorageItemPeer::instance()->deleteByPrimaryKey($key);
            } else {
                return $item->getData();
            }
        }
        return null;
    }

    /**
     * Cleans outdated items from the database.
     */
    public function clean()
    {
        $date = new ODate();
        $c = new Criteria();
        $c->add("date + (timeout || 'sec')::interval", new ODate(), '<');
        StorageItemPeer::instance()->delete($c);
    }
}
