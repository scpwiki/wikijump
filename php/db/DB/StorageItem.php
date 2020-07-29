<?php
namespace DB;

/**
 * Object Model class.
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
