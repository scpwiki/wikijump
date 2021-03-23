<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class StorageItem extends StorageItemBase
{

    public function setData($data)
    {
        parent::setData(serialize($data));
    }

    public function getData()
    {
        return unserialize(pg_unescape_bytea(parent::getData()));
    }
}
