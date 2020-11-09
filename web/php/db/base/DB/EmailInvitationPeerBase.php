<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table email_invitation.
 */
class EmailInvitationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='email_invitation';
        $this->objectName='DB\\EmailInvitation';
        $this->primaryKeyName = 'invitation_id';
        $this->fieldNames = array( 'invitation_id' ,  'hash' ,  'email' ,  'name' ,  'user_id' ,  'site_id' ,  'become_member' ,  'to_contacts' ,  'message' ,  'attempts' ,  'accepted' ,  'delivered' ,  'date' );
        $this->fieldTypes = array( 'invitation_id' => 'serial',  'hash' => 'varchar(200)',  'email' => 'varchar(128)',  'name' => 'varchar(100)',  'user_id' => 'int',  'site_id' => 'int',  'become_member' => 'boolean',  'to_contacts' => 'boolean',  'message' => 'text',  'attempts' => 'int',  'accepted' => 'boolean',  'delivered' => 'boolean',  'date' => 'timestamp');
        $this->defaultValues = array( 'become_member' => 'true',  'attempts' => '1',  'accepted' => 'false',  'delivered' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\EmailInvitationPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
