<?php
use DB\ContactPeer;

class PMComposeContactsListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        // get all contacts
        $c = new Criteria();
        $c->add("contact.user_id", $user->getUserId());
        $c->addJoin("target_user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $contacts = ContactPeer::instance()->select($c);

        // avatar uri
        foreach ($contacts as &$co) {
            $userId = $co->getTargetUserId();
            $co->setTemp("avatarUri", '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a16.png');
        }

        $runData->contextAdd("contacts", $contacts);
    }
}
