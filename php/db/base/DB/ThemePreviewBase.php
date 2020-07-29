<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table theme_preview.
 */
class ThemePreviewBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='theme_preview';
        $this->peerName = 'DB\\ThemePreviewPeer';
        $this->primaryKeyName = 'theme_id';
        $this->fieldNames = array( 'theme_id' ,  'body' );

        //$this->fieldDefaultValues=
    }






    public function getThemeId()
    {
        return $this->getFieldValue('theme_id');
    }

    public function setThemeId($v1, $raw = false)
    {
        $this->setFieldValue('theme_id', $v1, $raw);
    }


    public function getBody()
    {
        return $this->getFieldValue('body');
    }

    public function setBody($v1, $raw = false)
    {
        $this->setFieldValue('body', $v1, $raw);
    }
}
