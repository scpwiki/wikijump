<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table member_invitation.
 */
class MemberInvitationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='member_invitation';
        $this->objectName='DB\\MemberInvitation';
        $this->primaryKeyName = 'invitation_id';
        $this->fieldNames = array( 'invitation_id' ,  'site_id' ,  'user_id' ,  'by_user_id' ,  'date' ,  'body' );
        $this->fieldTypes = array( 'invitation_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'by_user_id' => 'int',  'date' => 'timestamp',  'body' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\MemberInvitationPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
