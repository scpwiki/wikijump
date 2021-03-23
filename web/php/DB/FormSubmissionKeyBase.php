<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table form_submission_key.
 */
class FormSubmissionKeyBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='form_submission_key';
        $this->peerName = 'DB\\FormSubmissionKeyPeer';
        $this->primaryKeyName = 'key_id';
        $this->fieldNames = array( 'key_id' ,  'date_submitted' );

        //$this->fieldDefaultValues=
    }






    public function getKeyId()
    {
        return $this->getFieldValue('key_id');
    }

    public function setKeyId($v1, $raw = false)
    {
        $this->setFieldValue('key_id', $v1, $raw);
    }


    public function getDateSubmitted()
    {
        return $this->getFieldValue('date_submitted');
    }

    public function setDateSubmitted($v1, $raw = false)
    {
        $this->setFieldValue('date_submitted', $v1, $raw);
    }
}
