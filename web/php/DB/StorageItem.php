<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class StorageItem extends StorageItemBase
{

    /**
     * @param $data
     * @param false $raw
     */
    public function setData($data, $raw = false)
    {
        parent::setData(serialize($data));
    }

    public function getData()
    {
        return unserialize(pg_unescape_bytea(parent::getData()));
    }
}
